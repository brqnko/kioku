import { useTranslation } from "react-i18next";
import type { RunCode200 } from "../api/generated/backend.schemas";
import type { Compiler } from "../hooks/useCompilers";
import { CompilerPicker } from "./CompilerPicker";

export interface ToolbarProps {
  language: string;
  compiler: string | null;
  allCompilers: Compiler[];
  stdinOpen: boolean;
  stdin: string;
  loading: boolean;
  pickerOpen: boolean;
  onCompilerChange: (name: string) => void;
  onToggleStdin: () => void;
  onStdinChange: (value: string) => void;
  onRun: () => void;
  onOpenPicker: () => void;
  onClosePicker: () => void;
}

export function CodeBlockToolbar({
  language,
  compiler,
  allCompilers,
  stdinOpen,
  stdin,
  loading,
  pickerOpen,
  onCompilerChange,
  onToggleStdin,
  onStdinChange,
  onRun,
  onOpenPicker,
  onClosePicker,
}: ToolbarProps) {
  const { t } = useTranslation();
  const selected = compiler
    ? allCompilers.find((item) => item.name === compiler)
    : null;
  const compilerLabel = selected
    ? selected.display_name || selected.name
    : allCompilers.length === 0
      ? t("codeBlock.compilerLoading")
      : t("codeBlock.selectCompiler");
  const runDisabled = loading || !compiler;

  return (
    <div class="code-block-toolbar">
      <div class="code-block-header">
        <div class="code-block-identity">
          <button
            type="button"
            class="code-block-compiler-button"
            disabled={loading || allCompilers.length === 0}
            onClick={onOpenPicker}
          >
            <span class="material-symbols-outlined text-[15px]">terminal</span>
            <span class="truncate">{compilerLabel}</span>
            <span class="material-symbols-outlined text-[15px]">
              expand_more
            </span>
          </button>
        </div>
        <div class="code-block-actions">
          <button
            type="button"
            class={`code-block-icon-button ${stdinOpen ? "is-active" : ""}`}
            aria-pressed={stdinOpen}
            aria-label={t("codeBlock.stdin")}
            title={t("codeBlock.stdin")}
            onClick={onToggleStdin}
          >
            <span class="material-symbols-outlined text-[18px]">input</span>
          </button>
          <button
            type="button"
            class="code-block-run-button"
            disabled={runDisabled}
            title={!compiler ? t("codeBlock.selectCompilerHint") : undefined}
            onClick={onRun}
          >
            <span
              class={`material-symbols-outlined text-[18px] ${
                loading ? "animate-spin" : ""
              }`}
            >
              {loading ? "progress_activity" : "play_arrow"}
            </span>
            <span>{loading ? t("codeBlock.running") : t("codeBlock.run")}</span>
          </button>
        </div>
      </div>
      {stdinOpen && (
        <div class="code-block-stdin">
          <textarea
            value={stdin}
            maxLength={32 * 1024}
            rows={3}
            placeholder={t("codeBlock.stdinPlaceholder")}
            onInput={(event) =>
              onStdinChange((event.target as HTMLTextAreaElement).value)
            }
          />
        </div>
      )}
      <CompilerPicker
        open={pickerOpen}
        onClose={onClosePicker}
        compilers={allCompilers}
        selected={compiler}
        preferredLanguage={language}
        onSelect={onCompilerChange}
      />
    </div>
  );
}

export interface OutputProps {
  result: RunCode200 | null;
  errorMessage: string | null;
  collapsed: boolean;
  onToggleCollapsed: () => void;
}

function nonEmpty(value: string | null | undefined): string | null {
  return value && value.length > 0 ? value : null;
}

function resultSections(result: RunCode200 | null) {
  if (!result) return [];
  return [
    {
      body: nonEmpty(result.compiler_error),
      labelKey: "codeBlock.outputs.compileErrors",
      tone: "error" as const,
    },
    {
      body: nonEmpty(result.compiler_output),
      labelKey: "codeBlock.outputs.compilerOutput",
      tone: "default" as const,
    },
    {
      body: nonEmpty(result.compiler_message),
      labelKey: "codeBlock.outputs.compilerMessage",
      tone: "default" as const,
    },
    {
      body: nonEmpty(result.program_output),
      labelKey: "codeBlock.outputs.stdout",
      tone: "default" as const,
    },
    {
      body: nonEmpty(result.program_error),
      labelKey: "codeBlock.outputs.stderr",
      tone: "error" as const,
    },
    {
      body: nonEmpty(result.program_message),
      labelKey: "codeBlock.outputs.programMessage",
      tone: "default" as const,
    },
  ].filter((section) => section.body);
}

export function CodeBlockOutput({
  result,
  errorMessage,
  collapsed,
  onToggleCollapsed,
}: OutputProps) {
  const { t } = useTranslation();
  const sections = resultSections(result);
  const hasBody = Boolean(errorMessage || sections.length > 0);

  if (!result && !errorMessage) return null;

  return (
    <div class="code-block-result">
      <div class="code-block-result-header">
        <div class="code-block-result-status">
          {errorMessage ? (
            <span class="code-block-result-badge is-error">
              {t("codeBlock.requestFailed")}
            </span>
          ) : null}
        </div>
        <button
          type="button"
          class="code-block-result-toggle"
          disabled={!hasBody}
          aria-expanded={!collapsed}
          title={collapsed ? t("codeBlock.expand") : t("codeBlock.collapse")}
          onClick={onToggleCollapsed}
        >
          <span class="material-symbols-outlined text-[19px]">
            {collapsed ? "expand_more" : "expand_less"}
          </span>
        </button>
      </div>
      {!collapsed && (
        <div class="code-block-result-body">
          {errorMessage && (
            <pre class="code-block-result-pre is-error">{errorMessage}</pre>
          )}
          {sections.map((section) => (
            <OutputSection
              key={section.labelKey}
              label={t(section.labelKey)}
              body={section.body!}
              tone={section.tone}
            />
          ))}
          {!errorMessage && sections.length === 0 && (
            <div class="code-block-result-empty">{t("codeBlock.noOutput")}</div>
          )}
        </div>
      )}
    </div>
  );
}

function OutputSection({
  label,
  body,
  tone,
}: {
  label: string;
  body: string;
  tone: "default" | "error";
}) {
  return (
    <section class="code-block-result-section">
      <div class="code-block-result-label">{label}</div>
      <pre
        class={`code-block-result-pre ${tone === "error" ? "is-error" : ""}`}
      >
        {body}
      </pre>
    </section>
  );
}
