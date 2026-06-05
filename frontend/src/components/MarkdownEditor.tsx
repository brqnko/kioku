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
import type { Node as ProseNode } from "@milkdown/prose/model";
import { listener, listenerCtx } from "@milkdown/plugin-listener";
import { splitBlock } from "@milkdown/prose/commands";
import { keymap } from "@milkdown/prose/keymap";
import { Plugin, TextSelection } from "@milkdown/prose/state";
import type { EditorView } from "@milkdown/prose/view";
import { callCommand } from "@milkdown/utils";
import { useTranslation } from "react-i18next";
import { useCompilers, type Compiler } from "../hooks/useCompilers";
import { CodeBlockView } from "./CodeBlockNodeView";
import { ImageLightbox } from "./ImageLightbox";

interface MarkdownEditorProps {
  defaultValue: string;
  onChange: (markdown: string) => void;
  onImagePaste?: (file: File) => Promise<string>;
}

type ToolbarAction =
  | { kind: "bold" }
  | { kind: "italic" }
  | { kind: "strikethrough" }
  | { kind: "inlineCode" }
  | { kind: "heading"; level: 1 | 2 | 3 }
  | { kind: "paragraph" }
  | { kind: "bulletList" }
  | { kind: "orderedList" }
  | { kind: "blockquote" }
  | { kind: "codeBlock" };

interface ToolbarButtonProps {
  label: string;
  icon: string;
  disabled?: boolean;
  onClick: () => void;
}

function ToolbarButton({ label, icon, disabled, onClick }: ToolbarButtonProps) {
  return (
    <button
      type="button"
      class="icon-button"
      aria-label={label}
      title={label}
      disabled={disabled}
      onMouseDown={(event) => event.preventDefault()}
      onClick={onClick}
    >
      <span class="material-symbols-outlined text-[20px]">{icon}</span>
    </button>
  );
}

interface TopLevelBlock {
  index: number;
  node: ProseNode;
  pos: number;
  rect: DOMRect;
}

function elementFromTarget(target: EventTarget | null): Element | null {
  if (!target) return null;
  if (target instanceof Element) return target;
  if (target instanceof Node) return target.parentElement;
  return null;
}

function collectTopLevelBlocks(view: EditorView): TopLevelBlock[] {
  const blocks: TopLevelBlock[] = [];
  view.state.doc.forEach((node, offset, index) => {
    const dom = view.nodeDOM(offset);
    if (!(dom instanceof Element)) return;
    blocks.push({
      index,
      node,
      pos: offset,
      rect: dom.getBoundingClientRect(),
    });
  });
  return blocks;
}

function dispatchTextSelection(
  view: EditorView,
  pos: number,
  event: MouseEvent,
): boolean {
  event.preventDefault();
  const selection = TextSelection.create(view.state.doc, pos);
  view.dispatch(view.state.tr.setSelection(selection).scrollIntoView());
  view.focus();
  return true;
}

function textblockStart(block: TopLevelBlock): number {
  return block.pos + 1;
}

function textblockEnd(block: TopLevelBlock): number {
  return block.pos + 1 + block.node.content.size;
}

function placeBoundaryCursor(
  view: EditorView,
  blocks: TopLevelBlock[],
  index: number,
  side: "before" | "after",
  event: MouseEvent,
): boolean {
  const paragraphType = view.state.schema.nodes.paragraph;
  if (!paragraphType) return false;

  const block = blocks[index];
  if (!block) return false;

  event.preventDefault();
  let tr = view.state.tr;
  let cursorPos: number;

  if (side === "before") {
    const prev = blocks[index - 1];
    if (prev?.node.isTextblock) {
      cursorPos = textblockEnd(prev);
    } else if (block.node.isTextblock) {
      cursorPos = textblockStart(block);
    } else {
      tr = tr.insert(block.pos, paragraphType.create());
      cursorPos = block.pos + 1;
    }
  } else {
    const next = blocks[index + 1];
    if (next?.node.isTextblock) {
      cursorPos = textblockStart(next);
    } else if (block.node.isTextblock) {
      cursorPos = textblockEnd(block);
    } else {
      const insertAt = block.pos + block.node.nodeSize;
      tr = tr.insert(insertAt, paragraphType.create());
      cursorPos = insertAt + 1;
    }
  }

  tr = tr.setSelection(TextSelection.create(tr.doc, cursorPos)).scrollIntoView();
  view.dispatch(tr);
  view.focus();
  return true;
}

function shouldIgnoreBoundaryClick(target: Element): boolean {
  return Boolean(
    target.closest(
      [
        ".code-block-cm",
        ".code-block-chrome",
        ".code-block-output",
        "button",
        "input",
        "textarea",
        "select",
        "a",
      ].join(","),
    ),
  );
}

export function MarkdownEditor({
  defaultValue,
  onChange,
  onImagePaste,
}: MarkdownEditorProps) {
  const { t } = useTranslation();
  const rootRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<Editor | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [ready, setReady] = useState(false);
  const [expandedImage, setExpandedImage] = useState<{
    alt: string;
    src: string;
  } | null>(null);

  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const onImagePasteRef = useRef(onImagePaste);
  onImagePasteRef.current = onImagePaste;
  const { data: compilers } = useCompilers();
  const compilersRef = useRef<Compiler[]>([]);
  const compilerSubsRef = useRef(new Set<() => void>());
  compilersRef.current = compilers ?? [];

  useEffect(() => {
    for (const callback of compilerSubsRef.current) {
      callback();
    }
  }, [compilers]);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    let cancelled = false;

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
          const files = Array.from(event.dataTransfer?.files ?? []).filter(
            (f) => f.type.startsWith("image/"),
          );
          if (files.length === 0) return false;
          event.preventDefault();

          let pos =
            view.posAtCoords({ left: event.clientX, top: event.clientY })
              ?.pos ?? view.state.selection.from;
          const $pos = view.state.doc.resolve(pos);
          if ($pos.parent.type.name === "code_block") {
            pos = $pos.after($pos.depth);
          }

          void (async () => {
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

    const trailingParagraphPlugin = new Plugin({
      appendTransaction(_trs, _oldState, newState) {
        const doc = newState.doc;
        const paragraphType = newState.schema.nodes.paragraph;
        if (!paragraphType) return null;

        const inserts: number[] = [];
        let pos = 0;
        for (let index = 0; index < doc.childCount; index += 1) {
          const node = doc.child(index);
          const next = index + 1 < doc.childCount ? doc.child(index + 1) : null;
          if (node.type.name === "code_block") {
            if (index === 0) inserts.push(pos);
            if (!next || next.type.name === "code_block") {
              inserts.push(pos + node.nodeSize);
            }
          }
          pos += node.nodeSize;
        }

        if (inserts.length === 0) return null;
        let tr = newState.tr;
        let offset = 0;
        for (const insertPos of inserts) {
          const paragraph = paragraphType.create();
          tr = tr.insert(insertPos + offset, paragraph);
          offset += paragraph.nodeSize;
        }
        tr.setMeta("addToHistory", false);
        return tr;
      },
    });

    const clickBoundaryPlugin = new Plugin({
      props: {
        handleDOMEvents: {
          mousedown(view, rawEvent) {
            if (!(rawEvent instanceof MouseEvent)) return false;
            if (rawEvent.button !== 0 || rawEvent.defaultPrevented) {
              return false;
            }

            const target = elementFromTarget(rawEvent.target);
            if (!target || !view.dom.contains(target)) return false;
            if (shouldIgnoreBoundaryClick(target)) return false;

            const blocks = collectTopLevelBlocks(view);
            if (blocks.length === 0) return false;

            const emptyParagraph = target.closest("p");
            if (
              emptyParagraph &&
              view.dom.contains(emptyParagraph) &&
              emptyParagraph.textContent?.trim() === ""
            ) {
              const block = blocks.find((item) => {
                const dom = view.nodeDOM(item.pos);
                return dom === emptyParagraph;
              });
              if (block?.node.type.name === "paragraph") {
                return dispatchTextSelection(
                  view,
                  textblockStart(block),
                  rawEvent,
                );
              }
            }

            const clickedCodeWrapper = target.closest(".code-block-wrapper");
            if (clickedCodeWrapper && view.dom.contains(clickedCodeWrapper)) {
              const block = blocks.find((item) => {
                const dom = view.nodeDOM(item.pos);
                return dom === clickedCodeWrapper;
              });
              if (block?.node.type.name === "code_block") {
                const side =
                  rawEvent.clientY < block.rect.top + block.rect.height / 2
                    ? "before"
                    : "after";
                return placeBoundaryCursor(
                  view,
                  blocks,
                  block.index,
                  side,
                  rawEvent,
                );
              }
            }

            if (target !== view.dom) return false;

            for (const block of blocks) {
              if (rawEvent.clientY < block.rect.top) {
                return placeBoundaryCursor(
                  view,
                  blocks,
                  block.index,
                  "before",
                  rawEvent,
                );
              }

              if (
                rawEvent.clientY >= block.rect.top &&
                rawEvent.clientY <= block.rect.bottom
              ) {
                const side =
                  rawEvent.clientY < block.rect.top + block.rect.height / 2
                    ? "before"
                    : "after";
                return placeBoundaryCursor(
                  view,
                  blocks,
                  block.index,
                  side,
                  rawEvent,
                );
              }
            }

            return placeBoundaryCursor(
              view,
              blocks,
              blocks.length - 1,
              "after",
              rawEvent,
            );
          },
        },
      },
    });

    const imageLightboxPlugin = new Plugin({
      props: {
        handleDOMEvents: {
          click(view, rawEvent) {
            if (!(rawEvent instanceof MouseEvent)) return false;
            const target = elementFromTarget(rawEvent.target);
            const image = target?.closest("img");
            if (!(image instanceof HTMLImageElement)) return false;
            if (!view.dom.contains(image)) return false;
            if (!image.currentSrc && !image.src) return false;

            rawEvent.preventDefault();
            rawEvent.stopPropagation();
            setExpandedImage({
              alt: image.alt,
              src: image.currentSrc || image.src,
            });
            return true;
          },
        },
      },
    });

    const headingEnterPlugin = keymap({
      Enter(state, dispatch, view) {
        const paragraphType = state.schema.nodes.paragraph;
        if (!paragraphType) return false;
        const { selection } = state;
        const { $from } = selection;
        if ($from.parent.type.name !== "heading") return false;

        let nextTr = state.tr;
        const didSplit = splitBlock(
          state,
          (tr) => {
            nextTr = tr;
          },
          view,
        );
        if (!didSplit) return false;

        const { $from: nextFrom } = nextTr.selection;
        if (nextFrom.parent.type.name === "heading") {
          nextTr = nextTr.setNodeMarkup(nextFrom.before(), paragraphType);
        }
        dispatch?.(nextTr.scrollIntoView());
        return true;
      },
    });

    Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, root);
        ctx.set(defaultValueCtx, defaultValue);
        ctx.update(prosePluginsCtx, (xs) =>
          xs.concat(
            pastePlugin,
            dropPlugin,
            trailingParagraphPlugin,
            imageLightboxPlugin,
            clickBoundaryPlugin,
            headingEnterPlugin,
          ),
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
                  (callback) => {
                    compilerSubsRef.current.add(callback);
                    return () => {
                      compilerSubsRef.current.delete(callback);
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
      .then((editor) => {
        if (cancelled) {
          editor.destroy();
          return;
        }
        editorRef.current = editor;
        setReady(true);
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
  }, []);

  const focusView = () => {
    editorRef.current?.action((ctx) => ctx.get(editorViewCtx).focus());
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

  const insertLink = () => {
    const editor = editorRef.current;
    if (!editor) return;
    const href = window.prompt(t("editor.toolbar.linkPrompt"))?.trim();
    if (!href) return;
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const linkMark = view.state.schema.marks.link;
      if (!linkMark) return;
      const mark = linkMark.create({ href });
      const { from, to, empty } = view.state.selection;
      const tr = empty
        ? view.state.tr.replaceSelectionWith(
            view.state.schema.text(href, [mark]),
            false,
          )
        : view.state.tr.addMark(from, to, mark);
      view.dispatch(tr);
      view.focus();
    });
  };

  const insertImage = (file: File) => {
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
    <>
      <div class="md-editor">
        <div class="md-toolbar" aria-label={t("editor.toolbar.label")}>
        <div class="md-toolbar-group">
          <ToolbarButton
            label={t("editor.toolbar.bold")}
            icon="format_bold"
            disabled={!ready}
            onClick={() => runAction({ kind: "bold" })}
          />
          <ToolbarButton
            label={t("editor.toolbar.italic")}
            icon="format_italic"
            disabled={!ready}
            onClick={() => runAction({ kind: "italic" })}
          />
          <ToolbarButton
            label={t("editor.toolbar.strikethrough")}
            icon="strikethrough_s"
            disabled={!ready}
            onClick={() => runAction({ kind: "strikethrough" })}
          />
          <ToolbarButton
            label={t("editor.toolbar.inlineCode")}
            icon="code"
            disabled={!ready}
            onClick={() => runAction({ kind: "inlineCode" })}
          />
        </div>
        <span class="md-toolbar-sep" aria-hidden />
        <div class="md-toolbar-group">
          <ToolbarButton
            label={t("editor.toolbar.h1")}
            icon="format_h1"
            disabled={!ready}
            onClick={() => runAction({ kind: "heading", level: 1 })}
          />
          <ToolbarButton
            label={t("editor.toolbar.h2")}
            icon="format_h2"
            disabled={!ready}
            onClick={() => runAction({ kind: "heading", level: 2 })}
          />
          <ToolbarButton
            label={t("editor.toolbar.paragraph")}
            icon="notes"
            disabled={!ready}
            onClick={() => runAction({ kind: "paragraph" })}
          />
        </div>
        <span class="md-toolbar-sep" aria-hidden />
        <div class="md-toolbar-group">
          <ToolbarButton
            label={t("editor.toolbar.bulletList")}
            icon="format_list_bulleted"
            disabled={!ready}
            onClick={() => runAction({ kind: "bulletList" })}
          />
          <ToolbarButton
            label={t("editor.toolbar.orderedList")}
            icon="format_list_numbered"
            disabled={!ready}
            onClick={() => runAction({ kind: "orderedList" })}
          />
          <ToolbarButton
            label={t("editor.toolbar.blockquote")}
            icon="format_quote"
            disabled={!ready}
            onClick={() => runAction({ kind: "blockquote" })}
          />
          <ToolbarButton
            label={t("editor.toolbar.codeBlock")}
            icon="data_object"
            disabled={!ready}
            onClick={() => runAction({ kind: "codeBlock" })}
          />
        </div>
        <span class="md-toolbar-sep" aria-hidden />
        <div class="md-toolbar-group">
          <ToolbarButton
            label={t("editor.toolbar.link")}
            icon="link"
            disabled={!ready}
            onClick={insertLink}
          />
          <ToolbarButton
            label={t("editor.toolbar.image")}
            icon="image"
            disabled={!ready || !onImagePaste}
            onClick={() => fileInputRef.current?.click()}
          />
          <input
            ref={fileInputRef}
            type="file"
            accept="image/*"
            class="md-toolbar-file-input"
            onChange={(event) => {
              const input = event.currentTarget;
              const file = input.files?.[0];
              input.value = "";
              if (file) insertImage(file);
            }}
          />
        </div>
        </div>
        <div ref={rootRef} class="milkdown-host" />
      </div>

      {expandedImage && (
        <ImageLightbox
          src={expandedImage.src}
          alt={expandedImage.alt}
          onClose={() => setExpandedImage(null)}
        />
      )}
    </>
  );
}
