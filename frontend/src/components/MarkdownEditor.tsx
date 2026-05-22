import { useEffect, useRef, useState } from "preact/hooks";
import {
  Editor,
  defaultValueCtx,
  editorViewCtx,
  nodeViewCtx,
  prosePluginsCtx,
  rootCtx,
} from "@milkdown/core";
import {
  commonmark,
  createCodeBlockCommand,
  toggleEmphasisCommand,
  toggleInlineCodeCommand,
  toggleStrongCommand,
  turnIntoTextCommand,
  wrapInBlockquoteCommand,
  wrapInBulletListCommand,
  wrapInHeadingCommand,
  wrapInOrderedListCommand,
} from "@milkdown/preset-commonmark";
import { gfm, toggleStrikethroughCommand } from "@milkdown/preset-gfm";
import { listener, listenerCtx } from "@milkdown/plugin-listener";
import { Plugin } from "@milkdown/prose/state";
import type { EditorView } from "@milkdown/prose/view";
import { callCommand } from "@milkdown/utils";
import { useTranslation } from "react-i18next";
import { CodeBlockView } from "./CodeBlockNodeView";
import {
  MarkdownEditorToolbar,
  type ToolbarAction,
} from "./MarkdownEditorToolbar";
import { useCompilers, type Compiler } from "../hooks/useCompilers";

interface MarkdownEditorProps {
  defaultValue: string;
  onChange: (markdown: string) => void;
  onImagePaste?: (file: File) => Promise<string>;
}

export function MarkdownEditor({
  defaultValue,
  onChange,
  onImagePaste,
}: MarkdownEditorProps) {
  const { t } = useTranslation();
  const rootRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<Editor | null>(null);
  const [ready, setReady] = useState(false);

  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const onImagePasteRef = useRef(onImagePaste);
  onImagePasteRef.current = onImagePaste;

  const { data: compilers } = useCompilers();
  const compilersRef = useRef<Compiler[]>([]);
  const compilerSubsRef = useRef(new Set<() => void>());
  compilersRef.current = compilers ?? [];

  useEffect(() => {
    for (const cb of compilerSubsRef.current) {
      try {
        cb();
      } catch {
        // ignore
      }
    }
  }, [compilers]);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    let cancelled = false;

    const trailingParagraphPlugin = new Plugin({
      appendTransaction(_trs, _oldState, newState) {
        const doc = newState.doc;
        const last = doc.lastChild;
        if (!last) return null;
        if (last.type.name !== "code_block") return null;
        const paragraphType = newState.schema.nodes.paragraph;
        if (!paragraphType) return null;
        return newState.tr.insert(doc.content.size, paragraphType.create());
      },
    });

    const insertImageAt = (
      view: EditorView,
      file: File,
      atPos: number | null,
    ): Promise<number | null> => {
      const handler = onImagePasteRef.current;
      if (!handler) return Promise.resolve(null);
      return handler(file)
        .then((url) => {
          const node = view.state.schema.nodes.image?.create({
            src: url,
            alt: "",
          });
          if (!node) return null;
          if (atPos == null) {
            view.dispatch(view.state.tr.replaceSelectionWith(node));
            return null;
          }
          view.dispatch(view.state.tr.insert(atPos, node));
          return atPos + node.nodeSize;
        })
        .catch((err) => {
          console.error("image insert failed", err);
          return atPos;
        });
    };

    const pastePlugin = new Plugin({
      props: {
        handlePaste(view, event) {
          if (!onImagePasteRef.current) return false;
          const items = event.clipboardData?.items;
          if (!items) return false;
          for (const item of Array.from(items)) {
            if (!item.type.startsWith("image/")) continue;
            const file = item.getAsFile();
            if (!file) continue;
            event.preventDefault();
            insertImageAt(view, file, null);
            return true;
          }
          return false;
        },
      },
    });

    const dropPlugin = new Plugin({
      props: {
        handleDrop(view, event) {
          if (!onImagePasteRef.current) return false;
          const dt = event.dataTransfer;
          const files = dt
            ? Array.from(dt.files).filter((f) => f.type.startsWith("image/"))
            : [];
          if (files.length === 0) return false;
          event.preventDefault();

          let pos =
            view.posAtCoords({ left: event.clientX, top: event.clientY })
              ?.pos ?? view.state.selection.from;
          const $pos = view.state.doc.resolve(pos);
          if ($pos.parent.type.name === "code_block") {
            pos = $pos.after($pos.depth);
          }

          (async () => {
            let insertPos: number | null = pos;
            for (const file of files) {
              insertPos = await insertImageAt(view, file, insertPos);
              if (insertPos == null) break;
            }
          })();
          return true;
        },
      },
    });

    Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, root);
        ctx.set(defaultValueCtx, defaultValue);
        ctx.update(prosePluginsCtx, (xs) =>
          xs.concat(pastePlugin, dropPlugin, trailingParagraphPlugin),
        );
        ctx.update(nodeViewCtx, (xs) =>
          xs.concat([
            [
              "code_block",
              (node, view, getPos) =>
                new CodeBlockView(
                  node,
                  view,
                  getPos,
                  () => compilersRef.current,
                  (cb) => {
                    compilerSubsRef.current.add(cb);
                    return () => {
                      compilerSubsRef.current.delete(cb);
                    };
                  },
                ),
            ],
          ]),
        );
        ctx.get(listenerCtx).markdownUpdated((_, markdown) => {
          onChangeRef.current(markdown);
        });
      })
      .use(commonmark)
      .use(gfm)
      .use(listener)
      .create()
      .then((e) => {
        if (cancelled) {
          e.destroy();
        } else {
          editorRef.current = e;
          setReady(true);
        }
      })
      .catch((err) => {
        console.error("Milkdown init failed", err);
      });

    return () => {
      cancelled = true;
      editorRef.current?.destroy();
      editorRef.current = null;
      setReady(false);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const focusView = () => {
    const editor = editorRef.current;
    if (!editor) return;
    editor.action((ctx) => ctx.get(editorViewCtx).focus());
  };

  const runAction = (action: ToolbarAction) => {
    const editor = editorRef.current;
    if (!editor) return;
    focusView();
    switch (action.kind) {
      case "bold":
        editor.action(callCommand(toggleStrongCommand.key));
        break;
      case "italic":
        editor.action(callCommand(toggleEmphasisCommand.key));
        break;
      case "strikethrough":
        editor.action(callCommand(toggleStrikethroughCommand.key));
        break;
      case "inlineCode":
        editor.action(callCommand(toggleInlineCodeCommand.key));
        break;
      case "heading":
        editor.action(callCommand(wrapInHeadingCommand.key, action.level));
        break;
      case "paragraph":
        editor.action(callCommand(turnIntoTextCommand.key));
        break;
      case "bulletList":
        editor.action(callCommand(wrapInBulletListCommand.key));
        break;
      case "orderedList":
        editor.action(callCommand(wrapInOrderedListCommand.key));
        break;
      case "blockquote":
        editor.action(callCommand(wrapInBlockquoteCommand.key));
        break;
      case "codeBlock":
        editor.action(callCommand(createCodeBlockCommand.key));
        break;
    }
  };

  const handleInsertLink = () => {
    const editor = editorRef.current;
    if (!editor) return;
    const raw = window.prompt(t("editor.toolbar.linkPrompt"));
    const href = raw?.trim();
    if (!href) return;
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const { state } = view;
      const linkMark = state.schema.marks.link;
      if (!linkMark) return;
      const mark = linkMark.create({ href });
      const { from, to, empty } = state.selection;
      const tr = empty
        ? state.tr.replaceSelectionWith(state.schema.text(href, [mark]), false)
        : state.tr.addMark(from, to, mark);
      view.dispatch(tr);
      view.focus();
    });
  };

  const handlePickImage = (file: File) => {
    const editor = editorRef.current;
    const handler = onImagePasteRef.current;
    if (!editor || !handler) return;
    handler(file)
      .then((url) => {
        editor.action((ctx) => {
          const view = ctx.get(editorViewCtx);
          const node = view.state.schema.nodes.image?.create({
            src: url,
            alt: "",
          });
          if (!node) return;
          view.dispatch(view.state.tr.replaceSelectionWith(node));
          view.focus();
        });
      })
      .catch((err) => {
        console.error("image pick failed", err);
      });
  };

  return (
    <div class="md-editor">
      <MarkdownEditorToolbar
        ready={ready}
        onAction={runAction}
        onInsertLink={handleInsertLink}
        onPickImage={handlePickImage}
      />
      <div ref={rootRef} class="milkdown-host" />
    </div>
  );
}
