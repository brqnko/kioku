import { useTranslation } from "react-i18next";
import type { Compiler } from "../hooks/useCompilers";
import type { RunCode200 } from "../api/generated/backend.schemas";
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
  const runDisabled = !compiler || loading;
  const selected = compiler
    ? allCompilers.find((c) => c.name === compiler)
    : null;
  const compilerLabel = selected
    ? `${selected.display_name || selected.name}${
        selected.version ? ` (${selected.version})` : ""
      }`
    : t("codeBlock.selectCompiler");
  return (
    <div class="flex flex-col gap-2 mb-2">
      <div class="flex items-center gap-2 flex-wrap">
        <span class="text-xs px-2 py-0.5 rounded bg-overlay-faint text-text-secondary">
          {language || "plain"}
        </span>
        <button
          type="button"
          onClick={onOpenPicker}
          disabled={loading || allCompilers.length === 0}
          class="text-xs px-2 py-1 rounded border border-border-subtle bg-surface-container-low text-text-primary hover:bg-overlay-faint disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5 cursor-pointer"
        >
          <span class="material-symbols-outlined text-[14px] text-text-secondary">
            terminal
          </span>
          <span class="truncate max-w-[260px]">{compilerLabel}</span>
          <span class="material-symbols-outlined text-[14px] text-text-disabled">
            unfold_more
          </span>
        </button>
        <label class="text-xs flex items-center gap-1 text-text-secondary cursor-pointer">
          <input
            type="checkbox"
            checked={stdinOpen}
            onChange={onToggleStdin}
          />
          {t("codeBlock.stdin")}
        </label>
        <button
          type="button"
          class="ml-auto text-xs px-3 py-1 rounded bg-accent-blue hover:brightness-110 text-white disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer border-none"
          disabled={runDisabled}
          title={!compiler ? t("codeBlock.selectCompilerHint") : ""}
          onClick={onRun}
        >
          {loading ? t("codeBlock.running") : t("codeBlock.run")}
        </button>
      </div>
      {stdinOpen && (
        <textarea
          class="text-xs font-mono w-full p-2 rounded border border-border-subtle bg-surface-container-low text-text-primary"
          rows={3}
          placeholder={t("codeBlock.stdinPlaceholder")}
          value={stdin}
          maxLength={32 * 1024}
          onInput={(e) => onStdinChange((e.target as HTMLTextAreaElement).value)}
        />
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
  loading: boolean;
  collapsed: boolean;
  onToggleCollapsed: () => void;
}

function nonEmpty(s: string | null | undefined): string | null {
  return s && s.length > 0 ? s : null;
}

export function CodeBlockOutput({
  result,
  errorMessage,
  loading,
  collapsed,
  onToggleCollapsed,
}: OutputProps) {
  const { t } = useTranslation();
  if (!result && !loading && !errorMessage) return null;

  const compileErr = result ? nonEmpty(result.compiler_error) : null;
  const stdout = result ? nonEmpty(result.program_output) : null;
  const stderr = result ? nonEmpty(result.program_error) : null;
  const status = result?.status ?? "";
  const signal = result ? nonEmpty(result.signal) : null;
  const hasBody = Boolean(errorMessage || compileErr || stdout || stderr);

  return (
    <div class="mt-2 rounded border border-border-subtle bg-surface-container-low p-2 text-xs">
      <div class={`flex items-center gap-2 ${collapsed || !hasBody ? "" : "mb-1"}`}>
        {loading ? (
          <span class="text-text-secondary">{t("codeBlock.running")}</span>
        ) : errorMessage ? (
          <span class="text-danger">{t("codeBlock.requestFailed")}</span>
        ) : (
          <>
            <span
              class={`px-2 py-0.5 rounded ${
                status === "0"
                  ? "bg-green-600/20 text-green-700 dark:text-green-300"
                  : "bg-amber-600/20 text-amber-700 dark:text-amber-300"
              }`}
            >
              {t("codeBlock.exit")} {status || "?"}
            </span>
            {signal && (
              <span class="px-2 py-0.5 rounded bg-red-600/20 text-red-700 dark:text-red-300">
                {t("codeBlock.signal")} {signal}
              </span>
            )}
          </>
        )}
        <button
          type="button"
          class="ml-auto text-text-secondary hover:text-text-primary cursor-pointer flex items-center disabled:opacity-50 disabled:cursor-not-allowed"
          onClick={onToggleCollapsed}
          disabled={!hasBody}
          aria-expanded={!collapsed}
          title={collapsed ? t("codeBlock.expand") : t("codeBlock.collapse")}
        >
          <span class="material-symbols-outlined text-[18px]">
            {collapsed ? "expand_more" : "expand_less"}
          </span>
        </button>
      </div>
      {!collapsed && (
        <>
          {errorMessage && (
            <pre class="whitespace-pre-wrap font-mono text-danger">{errorMessage}</pre>
          )}
          {compileErr && (
            <Section
              label={t("codeBlock.outputs.compileErrors")}
              body={compileErr}
              tone="error"
            />
          )}
          {stdout && (
            <Section label={t("codeBlock.outputs.stdout")} body={stdout} />
          )}
          {stderr && (
            <Section
              label={t("codeBlock.outputs.stderr")}
              body={stderr}
              tone="error"
            />
          )}
        </>
      )}
    </div>
  );
}

function Section({
  label,
  body,
  tone,
}: {
  label: string;
  body: string;
  tone?: "error";
}) {
  return (
    <div class="mt-1">
      <div class="text-[10px] uppercase tracking-wider text-text-secondary mb-0.5">
        {label}
      </div>
      <pre
        class={`whitespace-pre-wrap font-mono p-2 rounded bg-surface-container-lowest ${
          tone === "error" ? "text-danger" : "text-text-primary"
        }`}
      >
        {body}
      </pre>
    </div>
  );
}
