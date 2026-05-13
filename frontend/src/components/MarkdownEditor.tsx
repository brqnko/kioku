import { useEffect, useRef } from "preact/hooks";
import {
  Editor,
  defaultValueCtx,
  nodeViewCtx,
  prosePluginsCtx,
  rootCtx,
} from "@milkdown/core";
import { commonmark } from "@milkdown/preset-commonmark";
import { gfm } from "@milkdown/preset-gfm";
import { listener, listenerCtx } from "@milkdown/plugin-listener";
import { nord } from "@milkdown/theme-nord";
import { Plugin } from "@milkdown/prose/state";
import { CodeBlockView } from "./CodeBlockNodeView";
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
  const rootRef = useRef<HTMLDivElement>(null);
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
    let editor: Editor | null = null;
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

    const pastePlugin = new Plugin({
      props: {
        handlePaste(view, event) {
          const handler = onImagePasteRef.current;
          if (!handler) return false;
          const items = event.clipboardData?.items;
          if (!items) return false;
          for (const item of Array.from(items)) {
            if (!item.type.startsWith("image/")) continue;
            const file = item.getAsFile();
            if (!file) continue;
            event.preventDefault();
            handler(file)
              .then((url) => {
                const node = view.state.schema.nodes.image?.create({
                  src: url,
                  alt: "",
                });
                if (!node) return;
                view.dispatch(view.state.tr.replaceSelectionWith(node));
              })
              .catch((err) => {
                console.error("image paste failed", err);
              });
            return true;
          }
          return false;
        },
      },
    });

    Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, root);
        ctx.set(defaultValueCtx, defaultValue);
        ctx.update(prosePluginsCtx, (xs) =>
          xs.concat(pastePlugin, trailingParagraphPlugin),
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
      .config(nord)
      .use(commonmark)
      .use(gfm)
      .use(listener)
      .create()
      .then((e) => {
        if (cancelled) {
          e.destroy();
        } else {
          editor = e;
        }
      })
      .catch((err) => {
        console.error("Milkdown init failed", err);
      });

    return () => {
      cancelled = true;
      editor?.destroy();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return <div ref={rootRef} class="milkdown-host" />;
}
