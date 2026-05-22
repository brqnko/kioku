import { useEffect, useRef, useState } from "preact/hooks";
import { useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { MarkdownEditor } from "../components/MarkdownEditor";
import { useFileContent, fileContentKeyFor } from "../hooks/useFile";
import { useDocumentHead } from "../hooks/useDocumentHead";
import { invalidateAfterMutation } from "../utils/swrCache";
import { fetchFileAncestors, type BreadcrumbAncestor } from "../hooks/useFolder";
import { kyInstance } from "../api/mutator";
import { uploadFile } from "../api/upload";
import { formatSize } from "../utils/file";
import type {
  GetFileContent200,
  UpdateFileTextBody,
} from "../api/generated/backend.schemas";

type SaveState = "idle" | "saving" | "saved" | "error";
const AUTOSAVE_DELAY_MS = 1500;

function formatDate(iso: string, locale: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleDateString(locale, {
    year: "numeric",
    month: "long",
    day: "numeric",
  });
}

export default function FilePage() {
  const { t, i18n } = useTranslation();
  useDocumentHead({ title: "File — kioku", robots: "noindex,nofollow" });
  const route = useRoute();
  const { mutate } = useSWRConfig();
  const fileId = route.params.fileId;

  const { data, error, isLoading } = useFileContent(fileId);
  const file = data?.file;
  const content = data?.content;

  const [ancestors, setAncestors] = useState<BreadcrumbAncestor[]>([]);
  const [draft, setDraft] = useState<string | null>(null);
  const [savedText, setSavedText] = useState<string | null>(null);
  const [saveState, setSaveState] = useState<SaveState>("idle");
  const initRef = useRef<string | null>(null);

  const [copiedField, setCopiedField] = useState<"created" | "updated" | null>(null);

  const copyDate = (field: "created" | "updated", iso: string) => {
    navigator.clipboard.writeText(formatDate(iso, i18n.language)).catch(() => {});
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 1500);
  };

  const [editingName, setEditingName] = useState(false);
  const [nameDraft, setNameDraft] = useState("");
  const [editingDesc, setEditingDesc] = useState(false);
  const [descDraft, setDescDraft] = useState("");
  const nameInputRef = useRef<HTMLInputElement>(null);
  const descTextareaRef = useRef<HTMLTextAreaElement>(null);

  const patchMeta = async (
    patch: { name?: string; description?: string },
  ): Promise<void> => {
    if (!fileId) return;
    await kyInstance.patch(`files/${fileId}`, { json: patch });
    await Promise.all([
      mutate(fileContentKeyFor(fileId)),
      invalidateAfterMutation(mutate, {
        childListings: true,
        library: true,
        dashboard: true,
      }),
    ]);
  };

  const startEditName = () => {
    if (!file) return;
    setNameDraft(file.name);
    setEditingName(true);
    queueMicrotask(() => {
      nameInputRef.current?.focus();
      nameInputRef.current?.select();
    });
  };

  const commitName = async () => {
    if (!file) return setEditingName(false);
    const trimmed = nameDraft.trim();
    setEditingName(false);
    if (!trimmed || trimmed === file.name.trim()) return;
    try {
      await patchMeta({ name: trimmed });
    } catch {
      /* ignore — user can retry */
    }
  };

  const startEditDesc = () => {
    if (!file) return;
    setDescDraft(file.description ?? "");
    setEditingDesc(true);
    queueMicrotask(() => {
      descTextareaRef.current?.focus();
    });
  };

  const commitDesc = async () => {
    if (!file) return setEditingDesc(false);
    const trimmed = descDraft.trim();
    setEditingDesc(false);
    if (trimmed === (file.description ?? "").trim()) return;
    try {
      await patchMeta({ description: trimmed });
    } catch {
      /* ignore */
    }
  };

  useEffect(() => {
    if (!fileId) return;
    let cancelled = false;
    fetchFileAncestors(fileId)
      .then((chain) => {
        if (!cancelled) setAncestors(chain);
      })
      .catch(() => {
        if (!cancelled) setAncestors([]);
      });
    return () => {
      cancelled = true;
    };
  }, [fileId]);

  const isText = content?.kind === "text";

  // Initialize draft once, when text content first loads
  useEffect(() => {
    if (!isText || initRef.current === fileId) return;
    if (content?.kind === "text") {
      initRef.current = fileId ?? null;
      setDraft(content.content);
      setSavedText(content.content);
      setSaveState("idle");
    }
  }, [isText, content, fileId]);

  // Debounced auto-save
  useEffect(() => {
    if (!fileId || draft === null || savedText === null) return;
    if (draft === savedText) {
      setSaveState((s) => (s === "saving" ? s : "idle"));
      return;
    }

    const handle = window.setTimeout(async () => {
      setSaveState("saving");
      try {
        const body: UpdateFileTextBody = { text: draft };
        await kyInstance.put(`files/${fileId}/text`, { json: body });
        setSavedText(draft);
        setSaveState("saved");
        const key = fileContentKeyFor(fileId);
        await mutate(
          key,
          (prev: GetFileContent200 | undefined) =>
            prev
              ? { ...prev, content: { kind: "text" as const, content: draft } }
              : prev,
          { revalidate: false },
        );
      } catch {
        setSaveState("error");
      }
    }, AUTOSAVE_DELAY_MS);

    return () => window.clearTimeout(handle);
  }, [draft, savedText, fileId, mutate]);

  // Auto-clear "saved" indicator after a couple seconds
  useEffect(() => {
    if (saveState !== "saved") return;
    const handle = window.setTimeout(() => setSaveState("idle"), 2000);
    return () => window.clearTimeout(handle);
  }, [saveState]);


  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-[var(--sidebar-width)] h-[calc(100vh-3.5rem)] overflow-y-auto transition-[margin-left] duration-200 ease-in-out">
        <div class="w-full max-w-4xl mx-auto px-4 py-8 tablet:px-12 tablet:py-16">
          {error && (
            <p class="text-sm text-danger mb-4">{t("file.errors.load")}</p>
          )}

          {isLoading && !file && (
            <p class="text-sm text-text-secondary">{t("file.loading")}</p>
          )}

          {file && (
            <header class="mb-8 tablet:mb-12">
              <nav class="flex items-center gap-2 text-text-secondary text-sm font-medium flex-wrap mb-6">
                <a
                  href="/library"
                  class="hover:text-text-primary no-underline text-inherit"
                >
                  {t("project.breadcrumb.library")}
                </a>
                {ancestors.map((a) => (
                  <span
                    key={`${a.kind}-${a.id}`}
                    class="flex items-center gap-2"
                  >
                    <span class="material-symbols-outlined text-[16px]">
                      chevron_right
                    </span>
                    <a
                      href={
                        a.kind === "project"
                          ? `/projects/${a.id}`
                          : `/folders/${a.id}`
                      }
                      class="hover:text-text-primary no-underline text-inherit"
                    >
                      {a.name}
                    </a>
                  </span>
                ))}
                <span class="material-symbols-outlined text-[16px]">
                  chevron_right
                </span>
                <span class="text-text-primary">{file.name}</span>
              </nav>

              <div class="flex flex-col tablet:flex-row tablet:items-start tablet:justify-between gap-2 tablet:gap-4 mb-6">
                {editingName ? (
                  <input
                    ref={nameInputRef}
                    type="text"
                    value={nameDraft}
                    onInput={(e) =>
                      setNameDraft((e.target as HTMLInputElement).value)
                    }
                    onBlur={() => void commitName()}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        void commitName();
                      } else if (e.key === "Escape") {
                        e.preventDefault();
                        setEditingName(false);
                      }
                    }}
                    maxLength={256}
                    class="flex-1 min-w-0 font-bold tracking-tight text-text-primary text-[32px] leading-[1.15] tablet:text-[54px] tablet:leading-[1.04] bg-transparent border-none outline-none focus:ring-0 p-0 -ml-1 px-1 rounded -mr-2"
                  />
                ) : (
                  <h1
                    onClick={startEditName}
                    role="button"
                    tabIndex={0}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        startEditName();
                      }
                    }}
                    class="flex-1 min-w-0 font-bold tracking-tight text-text-primary text-[32px] leading-[1.15] tablet:text-[54px] tablet:leading-[1.04] cursor-text rounded -ml-1 px-1 hover:bg-overlay-faint"
                  >
                    {file.name}
                  </h1>
                )}
                {isText && (
                  <div
                    class="shrink-0 self-end text-sm text-text-secondary flex items-center gap-2"
                    aria-live="polite"
                  >
                    {saveState === "saving" && (
                      <>
                        <span class="material-symbols-outlined text-[16px] animate-spin">
                          progress_activity
                        </span>
                        {t("file.autosave.saving")}
                      </>
                    )}
                    {saveState === "saved" && (
                      <>
                        <span class="material-symbols-outlined text-[16px] text-success">
                          check
                        </span>
                        {t("file.autosave.saved")}
                      </>
                    )}
                    {saveState === "error" && (
                      <span class="text-danger">
                        {t("file.errors.save")}
                      </span>
                    )}
                  </div>
                )}
              </div>

              <div class="flex flex-col gap-2 text-sm text-text-secondary">
                <div class="flex items-center gap-4">
                  <span class="flex items-center gap-2 w-24 tablet:w-32">
                    <span class="material-symbols-outlined text-[16px]">
                      calendar_today
                    </span>
                    {t("file.createdAt")}
                  </span>
                  <span class="text-text-primary">
                    {formatDate(file.uploaded_at, i18n.language)}
                  </span>
                  <button
                    type="button"
                    onClick={() => copyDate("created", file.uploaded_at)}
                    aria-label={t("file.copy", { defaultValue: "Copy" })}
                    class="icon-button"
                  >
                    <span class="material-symbols-outlined text-[14px]">
                      {copiedField === "created" ? "check" : "content_copy"}
                    </span>
                  </button>
                </div>
                <div class="flex items-center gap-4">
                  <span class="flex items-center gap-2 w-24 tablet:w-32">
                    <span class="material-symbols-outlined text-[16px]">
                      update
                    </span>
                    {t("file.updatedAt")}
                  </span>
                  <span class="text-text-primary">
                    {formatDate(file.changed_at, i18n.language)}
                  </span>
                  <button
                    type="button"
                    onClick={() => copyDate("updated", file.changed_at)}
                    aria-label={t("file.copy", { defaultValue: "Copy" })}
                    class="icon-button"
                  >
                    <span class="material-symbols-outlined text-[14px]">
                      {copiedField === "updated" ? "check" : "content_copy"}
                    </span>
                  </button>
                </div>
                <div class="flex items-center gap-4">
                  <span class="flex items-center gap-2 w-24 tablet:w-32">
                    <span class="material-symbols-outlined text-[16px]">
                      database
                    </span>
                    {t("file.size")}
                  </span>
                  <span class="text-text-primary">
                    {formatSize(file.file_size)}
                  </span>
                </div>
                <div class="flex items-start gap-4">
                  <span class="flex items-center gap-2 w-24 tablet:w-32 shrink-0 mt-0.5">
                    <span class="material-symbols-outlined text-[16px]">
                      notes
                    </span>
                    {t("file.description.label")}
                  </span>
                  {editingDesc ? (
                    <textarea
                      ref={descTextareaRef}
                      value={descDraft}
                      onInput={(e) =>
                        setDescDraft((e.target as HTMLTextAreaElement).value)
                      }
                      onBlur={() => void commitDesc()}
                      onKeyDown={(e) => {
                        if (e.key === "Escape") {
                          e.preventDefault();
                          setEditingDesc(false);
                        }
                      }}
                      maxLength={1024}
                      rows={2}
                      class="flex-1 min-w-0 -ml-1 px-1 py-0.5 text-sm text-text-primary bg-transparent border-none outline-none focus:ring-1 focus:ring-accent-blue/40 rounded resize-y leading-[1.5]"
                    />
                  ) : (
                    <button
                      type="button"
                      onClick={startEditDesc}
                      class="flex-1 min-w-0 -ml-1 px-1 py-0.5 rounded text-left hover:bg-overlay-faint cursor-text"
                      aria-label={t("file.description.edit")}
                    >
                      {file.description ? (
                        <span class="text-text-primary whitespace-pre-wrap break-words">
                          {file.description}
                        </span>
                      ) : (
                        <span class="text-text-disabled">
                          {t("file.description.empty")}
                        </span>
                      )}
                    </button>
                  )}
                </div>
              </div>

            </header>
          )}

          {isText && draft !== null && file && (
            <div class="rounded-lg border border-border-subtle bg-surface-container-low overflow-hidden">
              <MarkdownEditor
                key={fileId}
                defaultValue={draft}
                onChange={setDraft}
                onImagePaste={async (pasted) => {
                  const ext = pasted.type.split("/")[1] || "png";
                  const id = crypto.randomUUID().replace(/-/g, "");
                  const named = new File([pasted], `image-${id}.${ext}`, {
                    type: pasted.type,
                  });
                  const result = await uploadFile({
                    file: named,
                    parentId: file.parent_id,
                    parentKind: file.parent_kind,
                  });
                  return `/api/files/${result.id}/raw`;
                }}
              />
            </div>
          )}

          {content?.kind === "url" &&
            (file?.name.toLowerCase().endsWith(".pdf") ? (
              <div class="rounded-[12px] border border-border-subtle overflow-hidden bg-surface-container-low">
                <iframe
                  src={content.url}
                  title={file?.name ?? "PDF"}
                  class="w-full h-[calc(100vh-12rem)] block"
                />
              </div>
            ) : (
              <div class="flex flex-col items-start gap-4">
                <p class="text-base text-text-secondary">
                  {t("file.binary.notice")}
                </p>
                <a
                  href={content.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  class="btn-primary no-underline"
                >
                  <span class="material-symbols-outlined text-[20px]">
                    open_in_new
                  </span>
                  {t("file.binary.open")}
                </a>
              </div>
            ))}
        </div>
      </main>
    </div>
  );
}
