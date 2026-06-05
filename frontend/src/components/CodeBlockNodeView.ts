import { render, h } from "preact";
import type { Node as ProseNode } from "@milkdown/prose/model";
import { TextSelection } from "@milkdown/prose/state";
import type {
  EditorView as PMEditorView,
  NodeView,
} from "@milkdown/prose/view";
import {
  Compartment,
  EditorState as CMState,
  type Extension,
} from "@codemirror/state";
import {
  EditorView as CMEditorView,
  keymap as cmKeymap,
  type KeyBinding,
} from "@codemirror/view";
import type { RunCode200, RunCodeBody } from "../api/generated/backend.schemas";
import { kyInstance } from "../api/mutator";
import type { Compiler } from "../hooks/useCompilers";
import { normalizeLanguageKey, pickCompiler } from "../utils/codeRunner";
import {
  createBaseExtensions,
  getCurrentTheme,
  getThemeExtension,
  loadLanguageExtension,
  onThemeChange,
  resolveLang,
  type ThemeMode,
} from "../utils/codemirror";
import { CodeBlockOutput, CodeBlockToolbar } from "./CodeBlockChrome";

export type GetCompilers = () => Compiler[];
export type SubscribeCompilers = (callback: () => void) => () => void;

interface LocalState {
  language: string;
  compiler: string | null;
  stdinOpen: boolean;
  stdin: string;
  loading: boolean;
  result: RunCode200 | null;
  errorMessage: string | null;
  runId: number;
  pickerOpen: boolean;
  outputCollapsed: boolean;
}

export class CodeBlockView implements NodeView {
  dom: HTMLElement;
  node: ProseNode;
  private chromeRoot: HTMLElement;
  private cmHost: HTMLElement;
  private outputRoot: HTMLElement;
  private state: LocalState;
  private destroyed = false;
  private abort: AbortController | null = null;
  private cm: CMEditorView;
  private langCompartment = new Compartment();
  private themeCompartment = new Compartment();
  private themeUnsub: (() => void) | null = null;
  private compilersUnsub: (() => void) | null = null;
  private updatingFromPM = false;
  private currentLangKey = "";
  private outsideMouseDown: ((event: MouseEvent) => void) | null = null;

  constructor(
    node: ProseNode,
    private pmView: PMEditorView,
    private getPos: () => number | undefined,
    private getCompilers: GetCompilers,
    private subscribeCompilers?: SubscribeCompilers,
  ) {
    this.node = node;

    this.dom = document.createElement("div");
    this.dom.className = "code-block-wrapper";

    this.chromeRoot = document.createElement("div");
    this.chromeRoot.className = "code-block-chrome";
    this.chromeRoot.contentEditable = "false";
    this.dom.appendChild(this.chromeRoot);

    this.cmHost = document.createElement("div");
    this.cmHost.className = "code-block-cm";
    this.dom.appendChild(this.cmHost);

    this.outputRoot = document.createElement("div");
    this.outputRoot.className = "code-block-output";
    this.outputRoot.contentEditable = "false";
    this.dom.appendChild(this.outputRoot);

    const language = (node.attrs.language as string | undefined) ?? "";
    const compilers = this.getCompilers();
    this.state = {
      language,
      compiler: pickCompiler(language, compilers),
      stdinOpen: false,
      stdin: "",
      loading: false,
      result: null,
      errorMessage: null,
      runId: 0,
      pickerOpen: false,
      outputCollapsed: true,
    };

    const themeMode = getCurrentTheme();
    const cmState = CMState.create({
      doc: node.textContent,
      extensions: [
        createBaseExtensions(),
        this.langCompartment.of([]),
        this.themeCompartment.of(getThemeExtension(themeMode)),
        cmKeymap.of(this.boundaryKeymap()),
        CMEditorView.updateListener.of((update) => {
          if (this.updatingFromPM) return;
          if (update.docChanged) {
            this.forwardToPM();
          }
          if (update.selectionSet || update.focusChanged) {
            this.forwardSelectionToPM();
          }
        }),
      ],
    });

    this.cm = new CMEditorView({
      parent: this.cmHost,
      state: cmState,
    });

    this.themeUnsub = onThemeChange((mode) => this.applyTheme(mode));
    this.compilersUnsub =
      this.subscribeCompilers?.(() => this.refreshCompilers()) ?? null;
    void this.applyLanguage(language);

    this.outsideMouseDown = (event) => {
      const target = event.target as Node | null;
      if (!target || this.dom.contains(target)) return;
      if (this.cm.hasFocus) {
        this.cm.contentDOM.blur();
      }
    };
    this.pmView.dom.addEventListener("mousedown", this.outsideMouseDown, true);

    this.renderChrome();
    this.renderOutput();
  }

  update(node: ProseNode): boolean {
    if (node.type !== this.node.type) return false;
    const newLanguage = (node.attrs.language as string | undefined) ?? "";
    const languageChanged = newLanguage !== this.state.language;
    this.node = node;

    if (languageChanged) {
      const compilers = this.getCompilers();
      const selected = this.state.compiler
        ? compilers.find((compiler) => compiler.name === this.state.compiler)
        : null;
      const selectedMatchesLanguage =
        selected &&
        normalizeLanguageKey(selected.language) ===
          normalizeLanguageKey(newLanguage);
      this.setState({
        language: newLanguage,
        compiler: selectedMatchesLanguage
          ? this.state.compiler
          : pickCompiler(newLanguage, compilers),
      });
      this.renderChrome();
      void this.applyLanguage(newLanguage);
    }

    const pmText = node.textContent;
    const cmText = this.cm.state.doc.toString();
    if (pmText !== cmText) {
      this.updatingFromPM = true;
      try {
        this.cm.dispatch({
          changes: { from: 0, insert: pmText, to: this.cm.state.doc.length },
        });
      } finally {
        this.updatingFromPM = false;
      }
    }

    return true;
  }

  setSelection(anchor: number, head: number): void {
    if (this.destroyed) return;
    this.cm.focus();
    const max = this.cm.state.doc.length;
    this.cm.dispatch({
      selection: { anchor: Math.min(anchor, max), head: Math.min(head, max) },
    });
  }

  stopEvent(event: Event): boolean {
    const target = event.target as Node | null;
    if (!target) return false;
    return (
      this.cmHost.contains(target) ||
      this.chromeRoot.contains(target) ||
      this.outputRoot.contains(target)
    );
  }

  ignoreMutation(): boolean {
    return true;
  }

  ignoreMutations(): boolean {
    return true;
  }

  destroy(): void {
    this.destroyed = true;
    this.themeUnsub?.();
    this.themeUnsub = null;
    this.compilersUnsub?.();
    this.compilersUnsub = null;
    if (this.outsideMouseDown) {
      this.pmView.dom.removeEventListener(
        "mousedown",
        this.outsideMouseDown,
        true,
      );
      this.outsideMouseDown = null;
    }
    this.abort?.abort();
    this.cm.destroy();
    render(null, this.chromeRoot);
    render(null, this.outputRoot);
  }

  private setState(patch: Partial<LocalState>): void {
    this.state = { ...this.state, ...patch };
  }

  private forwardSelectionToPM(): void {
    if (!this.cm.hasFocus) return;
    const pos = this.getPos();
    if (pos === undefined) return;
    const offset = pos + 1;
    const cmSelection = this.cm.state.selection.main;
    const pmState = this.pmView.state;
    const blockEnd = offset + this.node.content.size;
    const anchor = Math.min(blockEnd, offset + cmSelection.anchor);
    const head = Math.min(blockEnd, offset + cmSelection.head);
    let selection: TextSelection;
    try {
      selection = TextSelection.create(pmState.doc, anchor, head);
    } catch {
      return;
    }
    if (selection.eq(pmState.selection)) return;
    this.pmView.dispatch(pmState.tr.setSelection(selection));
  }

  private forwardToPM(): void {
    const pos = this.getPos();
    if (pos === undefined) return;
    const newText = this.cm.state.doc.toString();
    const oldText = this.node.textContent;
    if (newText === oldText) return;

    const start = pos + 1;
    const end = start + this.node.content.size;
    const { state } = this.pmView;
    const tr =
      newText.length === 0
        ? state.tr.delete(start, end)
        : state.tr.replaceWith(start, end, state.schema.text(newText));
    tr.setMeta("addToHistory", true);
    this.pmView.dispatch(tr);
  }

  private refreshCompilers(): void {
    if (this.destroyed) return;
    const compilers = this.getCompilers();
    const stillValid =
      this.state.compiler &&
      compilers.some((compiler) => compiler.name === this.state.compiler);
    this.setState({
      compiler: stillValid
        ? this.state.compiler
        : pickCompiler(this.state.language, compilers),
    });
    this.renderChrome();
  }

  private setBlockLanguage(language: string): void {
    if (language === this.state.language) return;
    const pos = this.getPos();
    if (pos === undefined) {
      this.setState({ language });
      void this.applyLanguage(language);
      this.renderChrome();
      return;
    }
    const attrs = { ...this.node.attrs, language };
    const tr = this.pmView.state.tr.setNodeMarkup(pos, undefined, attrs);
    this.pmView.dispatch(tr);
  }

  private handleCompilerChange(name: string): void {
    const compiler = this.getCompilers().find((item) => item.name === name);
    const nextLanguage = compiler?.language || this.state.language;
    this.setState({ compiler: name, pickerOpen: false });
    if (nextLanguage) {
      this.setBlockLanguage(nextLanguage);
    } else {
      this.renderChrome();
    }
  }

  private escapeBlock(side: "before" | "after"): boolean {
    const pos = this.getPos();
    if (pos === undefined) return false;
    const pmState = this.pmView.state;
    const paragraphType = pmState.schema.nodes.paragraph;
    let tr = pmState.tr;
    let cursorAt: number;

    if (side === "after") {
      const after = pos + this.node.nodeSize;
      const $after = pmState.doc.resolve(after);
      const nextBlock = $after.nodeAfter;
      if (!nextBlock || nextBlock.type.name === "code_block") {
        if (!paragraphType) return false;
        tr = tr.insert(after, paragraphType.create());
        cursorAt = after + 1;
      } else {
        cursorAt = after;
      }
    } else {
      const $before = pmState.doc.resolve(pos);
      const prevBlock = $before.nodeBefore;
      if (!prevBlock || prevBlock.type.name === "code_block") {
        if (!paragraphType) return false;
        tr = tr.insert(pos, paragraphType.create());
        cursorAt = pos + 1;
      } else {
        cursorAt = pos;
      }
    }

    tr = tr.setSelection(
      TextSelection.near(tr.doc.resolve(cursorAt), side === "before" ? -1 : 1),
    );
    tr.scrollIntoView();
    this.pmView.dispatch(tr);
    this.pmView.focus();
    return true;
  }

  private boundaryKeymap(): KeyBinding[] {
    const escape = (direction: "up" | "down" | "left" | "right"): boolean => {
      const { state } = this.cm;
      const selection = state.selection.main;
      if (!selection.empty) return false;
      const head = selection.head;
      const line = state.doc.lineAt(head);
      const atTop = line.number === 1;
      const atBottom = line.number === state.doc.lines;
      const atLineStart = head === line.from;
      const atLineEnd = head === line.to;
      const atDocStart = head === 0;
      const atDocEnd = head === state.doc.length;
      if (direction === "up" && atTop) return this.escapeBlock("before");
      if (direction === "down" && atBottom) return this.escapeBlock("after");
      if (direction === "left" && atDocStart && atLineStart) {
        return this.escapeBlock("before");
      }
      if (direction === "right" && atDocEnd && atLineEnd) {
        return this.escapeBlock("after");
      }
      return false;
    };

    return [
      { key: "ArrowUp", run: () => escape("up") },
      { key: "ArrowDown", run: () => escape("down") },
      { key: "ArrowLeft", run: () => escape("left") },
      { key: "ArrowRight", run: () => escape("right") },
      { key: "Mod-Enter", run: () => this.escapeBlock("after") },
      { key: "Shift-Mod-Enter", run: () => this.escapeBlock("before") },
    ];
  }

  private applyTheme(mode: ThemeMode): void {
    if (this.destroyed) return;
    this.cm.dispatch({
      effects: this.themeCompartment.reconfigure(getThemeExtension(mode)),
    });
  }

  private async applyLanguage(rawLanguage: string): Promise<void> {
    const key = resolveLang(rawLanguage);
    this.currentLangKey = key;
    let extension: Extension = [];
    if (key) {
      extension = await loadLanguageExtension(key);
    }
    if (this.destroyed || this.currentLangKey !== key) return;
    this.cm.dispatch({
      effects: this.langCompartment.reconfigure(extension),
    });
  }

  private renderChrome(): void {
    render(
      h(CodeBlockToolbar, {
        allCompilers: this.getCompilers(),
        compiler: this.state.compiler,
        language: this.state.language,
        loading: this.state.loading,
        pickerOpen: this.state.pickerOpen,
        stdin: this.state.stdin,
        stdinOpen: this.state.stdinOpen,
        onClosePicker: () => {
          this.setState({ pickerOpen: false });
          this.renderChrome();
        },
        onCompilerChange: (name) => this.handleCompilerChange(name),
        onOpenPicker: () => {
          this.setState({ pickerOpen: true });
          this.renderChrome();
        },
        onRun: () => {
          void this.run();
        },
        onStdinChange: (value) => {
          this.setState({ stdin: value });
          this.renderChrome();
        },
        onToggleStdin: () => {
          this.setState({ stdinOpen: !this.state.stdinOpen });
          this.renderChrome();
        },
      }),
      this.chromeRoot,
    );
  }

  private renderOutput(): void {
    render(
      h(CodeBlockOutput, {
        collapsed: this.state.outputCollapsed,
        errorMessage: this.state.errorMessage,
        result: this.state.result,
        onToggleCollapsed: () => {
          this.setState({ outputCollapsed: !this.state.outputCollapsed });
          this.renderOutput();
        },
      }),
      this.outputRoot,
    );
  }

  private async run(): Promise<void> {
    if (!this.state.compiler) return;
    this.abort?.abort();
    const controller = new AbortController();
    this.abort = controller;
    const runId = this.state.runId + 1;
    this.setState({
      errorMessage: null,
      loading: true,
      outputCollapsed: false,
      result: null,
      runId,
    });
    this.renderChrome();
    this.renderOutput();

    const body: RunCodeBody = {
      code: this.cm.state.doc.toString(),
      compiler: this.state.compiler,
      stdin: this.state.stdinOpen && this.state.stdin ? this.state.stdin : null,
    };

    try {
      const result = await kyInstance
        .post("files/run", { json: body, signal: controller.signal })
        .json<RunCode200>();
      if (this.destroyed || runId !== this.state.runId) return;
      this.setState({ loading: false, result });
    } catch (error) {
      if (this.destroyed || runId !== this.state.runId) return;
      if ((error as { name?: string })?.name === "AbortError") return;
      this.setState({
        errorMessage: error instanceof Error ? error.message : String(error),
        loading: false,
      });
    } finally {
      if (!this.destroyed && runId === this.state.runId) {
        this.renderChrome();
        this.renderOutput();
      }
    }
  }
}
