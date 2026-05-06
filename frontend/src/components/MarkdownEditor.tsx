import { useEffect, useRef } from "preact/hooks";
import { Editor, defaultValueCtx, rootCtx } from "@milkdown/core";
import { commonmark } from "@milkdown/preset-commonmark";
import { gfm } from "@milkdown/preset-gfm";
import { listener, listenerCtx } from "@milkdown/plugin-listener";
import { nord } from "@milkdown/theme-nord";

interface MarkdownEditorProps {
  defaultValue: string;
  onChange: (markdown: string) => void;
}

export function MarkdownEditor({
  defaultValue,
  onChange,
}: MarkdownEditorProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    let editor: Editor | null = null;
    let cancelled = false;

    Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, root);
        ctx.set(defaultValueCtx, defaultValue);
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
