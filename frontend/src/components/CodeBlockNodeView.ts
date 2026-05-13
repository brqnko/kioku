import { render, h } from "preact";
import type { Node as ProseNode } from "@milkdown/prose/model";
import type { EditorView as PMEditorView, NodeView } from "@milkdown/prose/view";
import { TextSelection } from "@milkdown/prose/state";
import {
  EditorState as CMState,
  Compartment,
  type Extension,
} from "@codemirror/state";
import {
  EditorView as CMEditorView,
  keymap as cmKeymap,
  type KeyBinding,
} from "@codemirror/view";
import { kyInstance } from "../api/mutator";
import type {
  RunCode200,
  RunCodeBody,
} from "../api/generated/backend.schemas";
import type { Compiler } from "../hooks/useCompilers";
import { pickCompiler } from "../utils/codeRunner";
import {
  createBaseExtensions,
  getCurrentTheme,
  getThemeExtension,
  loadLanguageExtension,
  onThemeChange,
  resolveLang,
  type ThemeMode,
} from "../utils/codemirror";
import { CodeBlockToolbar, CodeBlockOutput } from "./CodeBlockChrome";

export type GetCompilers = () => Compiler[];
export type SubscribeCompilers = (cb: () => void) => () => void;

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
  private outsideMouseDown: ((e: MouseEvent) => void) | null = null;
  node: ProseNode;

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
    const list = this.getCompilers();
    this.state = {
      language,
      compiler: pickCompiler(language, list),
      stdinOpen: false,
      stdin: "",
      loading: false,
      result: null,
      errorMessage: null,
      runId: 0,
      pickerOpen: false,
      outputCollapsed: false,
    };

    const themeMode = getCurrentTheme();
    const cmState = CMState.create({
      doc: node.textContent,
      extensions: [
        createBaseExtensions(),
        this.langCompartment.of([]),
        this.themeCompartment.of(getThemeExtension(themeMode)),
        cmKeymap.of(this.boundaryKeymap()),
        CMEditorView.updateListener.of((upd) => {
          if (this.updatingFromPM) return;
          if (upd.docChanged) {
            this.forwardToPM();
          }
          if (upd.selectionSet || upd.focusChanged) {
            this.forwardSelectionToPM();
          }
        }),
      ],
    });

    this.cm = new CMEditorView({
      state: cmState,
      parent: this.cmHost,
    });

    this.themeUnsub = onThemeChange((m) => this.applyTheme(m));
    this.compilersUnsub = this.subscribeCompilers?.(() =>
      this.refreshCompilers(),
    ) ?? null;
    this.applyLanguage(language);

    this.outsideMouseDown = (e) => {
      const target = e.target as Node | null;
      if (!target) return;
      if (this.dom.contains(target)) return;
      if (this.cm.hasFocus) {
        this.cm.contentDOM.blur();
      }
    };
    this.pmView.dom.addEventListener("mousedown", this.outsideMouseDown, true);

    this.renderChrome();
    this.renderOutput();
  }

  private forwardSelectionToPM(): void {
    if (!this.cm.hasFocus) return;
    const pos = this.getPos();
    if (pos === undefined) return;
    const offset = pos + 1;
    const cmSel = this.cm.state.selection.main;
    const pmState = this.pmView.state;
    const blockEnd = offset + this.node.content.size;
    const anchor = Math.min(blockEnd, offset + cmSel.anchor);
    const head = Math.min(blockEnd, offset + cmSel.head);
    let newSel: TextSelection;
    try {
      newSel = TextSelection.create(pmState.doc, anchor, head);
    } catch {
      return;
    }
    if (newSel.eq(pmState.selection)) return;
    this.pmView.dispatch(pmState.tr.setSelection(newSel));
  }

  private refreshCompilers(): void {
    if (this.destroyed) return;
    const list = this.getCompilers();
    const stillValid =
      this.state.compiler &&
      list.some((c) => c.name === this.state.compiler);
    this.setState({
      compiler: stillValid
        ? this.state.compiler
        : pickCompiler(this.state.language, list),
    });
    this.renderChrome();
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
      const hasNextBlock = $after.nodeAfter !== null;
      if (!hasNextBlock) {
        if (!paragraphType) return false;
        tr = tr.insert(after, paragraphType.create());
        cursorAt = after + 1;
      } else {
        cursorAt = after;
      }
    } else {
      const $before = pmState.doc.resolve(pos);
      const hasPrevBlock = $before.nodeBefore !== null;
      if (!hasPrevBlock) {
        if (!paragraphType) return false;
        tr = tr.insert(pos, paragraphType.create());
        cursorAt = pos + 1;
      } else {
        cursorAt = pos;
      }
    }

    tr = tr.setSelection(
      TextSelection.near(
        tr.doc.resolve(cursorAt),
        side === "before" ? -1 : 1,
      ),
    );
    tr.scrollIntoView();
    this.pmView.dispatch(tr);
    this.pmView.focus();
    return true;
  }

  private boundaryKeymap(): KeyBinding[] {
    const escape = (dir: "up" | "down" | "left" | "right"): boolean => {
      const { state } = this.cm;
      const sel = state.selection.main;
      if (!sel.empty) return false;
      const head = sel.head;
      const line = state.doc.lineAt(head);
      const atTop = line.number === 1;
      const atBottom = line.number === state.doc.lines;
      const atLineStart = head === line.from;
      const atLineEnd = head === line.to;
      const atDocStart = head === 0;
      const atDocEnd = head === state.doc.length;
      if (dir === "up" && atTop) return this.escapeBlock("before");
      if (dir === "down" && atBottom) return this.escapeBlock("after");
      if (dir === "left" && atDocStart && atLineStart)
        return this.escapeBlock("before");
      if (dir === "right" && atDocEnd && atLineEnd)
        return this.escapeBlock("after");
      return false;
    };
    return [
      { key: "ArrowUp", run: () => escape("up") },
      { key: "ArrowDown", run: () => escape("down") },
      { key: "ArrowLeft", run: () => escape("left") },
      { key: "ArrowRight", run: () => escape("right") },
      { key: "Mod-Enter", run: () => this.escapeBlock("after") },
    ];
  }

  private applyTheme(mode: ThemeMode): void {
    if (this.destroyed) return;
    this.cm.dispatch({
      effects: this.themeCompartment.reconfigure(getThemeExtension(mode)),
    });
  }

  private async applyLanguage(rawLang: string): Promise<void> {
    const key = resolveLang(rawLang);
    this.currentLangKey = key;
    let ext: Extension = [];
    if (key) {
      try {
        ext = await loadLanguageExtension(key);
      } catch {
        ext = [];
      }
    }
    if (this.destroyed || this.currentLangKey !== key) return;
    this.cm.dispatch({
      effects: this.langCompartment.reconfigure(ext),
    });
  }

  private forwardToPM(): void {
    const pos = this.getPos();
    if (pos === undefined) return;
    const newText = this.cm.state.doc.toString();
    const start = pos + 1;
    const end = start + this.node.content.size;
    const oldText = this.node.textContent;
    if (newText === oldText) return;
    const { state } = this.pmView;
    let tr = state.tr;
    if (newText.length === 0) {
      tr = tr.delete(start, end);
    } else {
      tr = tr.replaceWith(start, end, state.schema.text(newText));
    }
    tr.setMeta("addToHistory", true);
    this.pmView.dispatch(tr);
  }

  update(node: ProseNode): boolean {
    if (node.type !== this.node.type) return false;
    const newLang = (node.attrs.language as string | undefined) ?? "";
    const langChanged = newLang !== this.state.language;
    this.node = node;

    if (langChanged) {
      const list = this.getCompilers();
      this.state = {
        ...this.state,
        language: newLang,
        compiler: pickCompiler(newLang, list),
      };
      this.renderChrome();
      void this.applyLanguage(newLang);
    }

    const pmText = node.textContent;
    const cmText = this.cm.state.doc.toString();
    if (pmText !== cmText) {
      this.updatingFromPM = true;
      try {
        this.cm.dispatch({
          changes: { from: 0, to: this.cm.state.doc.length, insert: pmText },
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
    if (this.cmHost.contains(target)) return true;
    if (this.chromeRoot.contains(target)) return true;
    if (this.outputRoot.contains(target)) return true;
    return false;
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

  private renderChrome(): void {
    const allCompilers = this.getCompilers();
    render(
      h(CodeBlockToolbar, {
        language: this.state.language,
        compiler: this.state.compiler,
        allCompilers,
        stdinOpen: this.state.stdinOpen,
        stdin: this.state.stdin,
        loading: this.state.loading,
        pickerOpen: this.state.pickerOpen,
        onCompilerChange: (name) => {
          this.setState({ compiler: name });
          this.renderChrome();
        },
        onToggleStdin: () => {
          this.setState({ stdinOpen: !this.state.stdinOpen });
          this.renderChrome();
        },
        onStdinChange: (value) => {
          this.setState({ stdin: value });
          this.renderChrome();
        },
        onRun: () => {
          void this.run();
        },
        onOpenPicker: () => {
          this.setState({ pickerOpen: true });
          this.renderChrome();
        },
        onClosePicker: () => {
          this.setState({ pickerOpen: false });
          this.renderChrome();
        },
      }),
      this.chromeRoot,
    );
  }

  private renderOutput(): void {
    render(
      h(CodeBlockOutput, {
        result: this.state.result,
        errorMessage: this.state.errorMessage,
        loading: this.state.loading,
        collapsed: this.state.outputCollapsed,
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
    const myRunId = this.state.runId + 1;
    this.setState({
      runId: myRunId,
      loading: true,
      result: null,
      errorMessage: null,
      outputCollapsed: false,
    });
    this.renderChrome();
    this.renderOutput();

    const code = this.cm.state.doc.toString();
    const body: RunCodeBody = {
      code,
      compiler: this.state.compiler,
      stdin: this.state.stdinOpen && this.state.stdin ? this.state.stdin : null,
    };

    try {
      const result = await kyInstance
        .post("files/run", { json: body, signal: controller.signal })
        .json<RunCode200>();
      if (this.destroyed || myRunId !== this.state.runId) return;
      this.setState({ loading: false, result });
    } catch (err) {
      if (this.destroyed || myRunId !== this.state.runId) return;
      if ((err as { name?: string })?.name === "AbortError") return;
      this.setState({
        loading: false,
        errorMessage: err instanceof Error ? err.message : String(err),
      });
    } finally {
      if (!this.destroyed && myRunId === this.state.runId) {
        this.renderChrome();
        this.renderOutput();
      }
    }
  }
}
