import { useRef } from "preact/hooks";
import { useTranslation } from "react-i18next";
import type { JSX } from "preact";

export type ToolbarAction =
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

interface MarkdownEditorToolbarProps {
  ready: boolean;
  onAction: (action: ToolbarAction) => void;
  onInsertLink: () => void;
  onPickImage: (file: File) => void;
}

const isMac =
  typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform);
const mod = isMac ? "⌘" : "Ctrl";

interface ButtonSpec {
  icon: string;
  labelKey: string;
  shortcut?: string;
  action: ToolbarAction;
}

const inlineButtons: ButtonSpec[] = [
  { icon: "format_bold", labelKey: "editor.toolbar.bold", shortcut: `${mod}+B`, action: { kind: "bold" } },
  { icon: "format_italic", labelKey: "editor.toolbar.italic", shortcut: `${mod}+I`, action: { kind: "italic" } },
  { icon: "format_strikethrough", labelKey: "editor.toolbar.strikethrough", action: { kind: "strikethrough" } },
  { icon: "code", labelKey: "editor.toolbar.inlineCode", action: { kind: "inlineCode" } },
];

const headingButtons: ButtonSpec[] = [
  { icon: "format_h1", labelKey: "editor.toolbar.h1", action: { kind: "heading", level: 1 } },
  { icon: "format_h2", labelKey: "editor.toolbar.h2", action: { kind: "heading", level: 2 } },
  { icon: "format_h3", labelKey: "editor.toolbar.h3", action: { kind: "heading", level: 3 } },
  { icon: "format_paragraph", labelKey: "editor.toolbar.paragraph", action: { kind: "paragraph" } },
];

const listButtons: ButtonSpec[] = [
  { icon: "format_list_bulleted", labelKey: "editor.toolbar.bulletList", action: { kind: "bulletList" } },
  { icon: "format_list_numbered", labelKey: "editor.toolbar.orderedList", action: { kind: "orderedList" } },
];

const blockButtons: ButtonSpec[] = [
  { icon: "format_quote", labelKey: "editor.toolbar.blockquote", action: { kind: "blockquote" } },
  { icon: "code_blocks", labelKey: "editor.toolbar.codeBlock", action: { kind: "codeBlock" } },
];

export function MarkdownEditorToolbar({
  ready,
  onAction,
  onInsertLink,
  onPickImage,
}: MarkdownEditorToolbarProps) {
  const { t } = useTranslation();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleMouseDown = (e: JSX.TargetedMouseEvent<HTMLElement>) => {
    e.preventDefault();
  };

  const renderButton = (b: ButtonSpec) => {
    const label = t(b.labelKey);
    const title = b.shortcut ? `${label} (${b.shortcut})` : label;
    return (
      <button
        key={b.labelKey}
        type="button"
        class="icon-button"
        title={title}
        aria-label={label}
        disabled={!ready}
        onMouseDown={handleMouseDown}
        onClick={() => onAction(b.action)}
      >
        <span class="material-symbols-outlined">{b.icon}</span>
      </button>
    );
  };

  const linkLabel = t("editor.toolbar.link");
  const imageLabel = t("editor.toolbar.image");

  return (
    <div class="md-toolbar" role="toolbar" aria-label={t("editor.toolbar.label")}>
      <div class="md-toolbar-group">{inlineButtons.map(renderButton)}</div>
      <div class="md-toolbar-sep" aria-hidden="true" />
      <div class="md-toolbar-group">{headingButtons.map(renderButton)}</div>
      <div class="md-toolbar-sep" aria-hidden="true" />
      <div class="md-toolbar-group">{listButtons.map(renderButton)}</div>
      <div class="md-toolbar-sep" aria-hidden="true" />
      <div class="md-toolbar-group">{blockButtons.map(renderButton)}</div>
      <div class="md-toolbar-sep" aria-hidden="true" />
      <div class="md-toolbar-group">
        <button
          type="button"
          class="icon-button"
          title={linkLabel}
          aria-label={linkLabel}
          disabled={!ready}
          onMouseDown={handleMouseDown}
          onClick={onInsertLink}
        >
          <span class="material-symbols-outlined">link</span>
        </button>
        <button
          type="button"
          class="icon-button"
          title={imageLabel}
          aria-label={imageLabel}
          disabled={!ready}
          onMouseDown={handleMouseDown}
          onClick={() => fileInputRef.current?.click()}
        >
          <span class="material-symbols-outlined">image</span>
        </button>
        <input
          ref={fileInputRef}
          type="file"
          accept="image/*"
          class="md-toolbar-file-input"
          onChange={(e) => {
            const input = e.currentTarget;
            const file = input.files?.[0];
            if (file) onPickImage(file);
            input.value = "";
          }}
        />
      </div>
    </div>
  );
}
