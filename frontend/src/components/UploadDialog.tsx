import { useEffect, useRef, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { Dialog } from "./Dialog";
import { createTextFile, uploadFile } from "../api/upload";
import { invalidateAfterMutation } from "../utils/swrCache";
import { pushNotification } from "../notifications/store";

interface UploadDialogProps {
  open: boolean;
  onClose: () => void;
  parentId: string;
  parentKind: "project" | "folder";
  onSuccess: () => unknown | Promise<unknown>;
}

type Mode = "file" | "text";

type UploadStatus = "pending" | "uploading" | "done" | "failed";

interface UploadItem {
  id: string;
  file: File;
  status: UploadStatus;
  errorKey?: "tooLarge" | "failed";
}

const CONCURRENCY = 3;

function makeItemId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}

function filesToItems(files: FileList | File[] | null | undefined): UploadItem[] {
  if (!files) return [];
  const arr = Array.from(files);
  return arr.map((file) => ({ id: makeItemId(), file, status: "pending" }));
}

async function runWithConcurrency(
  tasks: Array<() => Promise<void>>,
  limit: number,
): Promise<void> {
  let idx = 0;
  const workers = Array.from(
    { length: Math.min(limit, tasks.length) },
    async () => {
      while (idx < tasks.length) {
        const i = idx++;
        await tasks[i]();
      }
    },
  );
  await Promise.all(workers);
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function UploadDialog({
  open,
  onClose,
  parentId,
  parentKind,
  onSuccess,
}: UploadDialogProps) {
  const { t } = useTranslation();
  const { mutate } = useSWRConfig();
  const [mode, setMode] = useState<Mode>("file");
  const [items, setItems] = useState<UploadItem[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const [name, setName] = useState("");
  const [text, setText] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setMode("file");
      setItems([]);
      setIsDragging(false);
      setName("");
      setText("");
      setError(null);
      setSubmitting(false);
    }
  }, [open]);

  const errorKeyFor = (err: unknown): "tooLarge" | "failed" =>
    err instanceof Error && err.message === "file_too_large"
      ? "tooLarge"
      : "failed";

  const translateTopError = (err: unknown): string =>
    err instanceof Error && err.message === "file_too_large"
      ? t("upload.errors.tooLarge")
      : t("upload.errors.failed");

  const addFiles = (files: FileList | File[] | null | undefined) => {
    const next = filesToItems(files);
    if (next.length === 0) return;
    setItems((prev) => [...prev, ...next]);
  };

  const handleFileChange = (e: Event) => {
    const input = e.target as HTMLInputElement;
    addFiles(input.files);
    input.value = "";
  };

  const handleRemove = (id: string) => {
    setItems((prev) =>
      prev.filter((it) => !(it.id === id && it.status === "pending")),
    );
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    if (!isDragging) setIsDragging(true);
  };
  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };
  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    if (submitting) return;
    addFiles(e.dataTransfer?.files);
  };

  const handleFileSubmit = async () => {
    const targets = items.filter((it) => it.status === "pending");
    if (targets.length === 0) {
      setError(t("upload.errors.fileRequired"));
      return;
    }
    setError(null);
    setSubmitting(true);

    const updateItem = (id: string, patch: Partial<UploadItem>) => {
      setItems((prev) =>
        prev.map((it) => (it.id === id ? { ...it, ...patch } : it)),
      );
    };

    const results = new Map<string, "done" | "failed">();
    const tasks: Array<() => Promise<void>> = targets.map((item) => async () => {
      updateItem(item.id, { status: "uploading", errorKey: undefined });
      try {
        await uploadFile({ file: item.file, parentId, parentKind });
        results.set(item.id, "done");
        updateItem(item.id, { status: "done" });
      } catch (err) {
        results.set(item.id, "failed");
        updateItem(item.id, { status: "failed", errorKey: errorKeyFor(err) });
      }
    });

    await runWithConcurrency(tasks, CONCURRENCY);

    setSubmitting(false);

    const total = targets.length;
    let ok = 0;
    let ng = 0;
    for (const v of results.values()) {
      if (v === "done") ok++;
      else ng++;
    }

    if (ok > 0) {
      await Promise.all([
        onSuccess(),
        invalidateAfterMutation(mutate, { library: true, dashboard: true }),
      ]);
    }

    if (ng === 0) {
      pushNotification({
        kind: "success",
        message: t("upload.summary.allOk", { count: ok }),
      });
      onClose();
    } else if (ok === 0) {
      pushNotification({
        kind: "error",
        message: t("upload.summary.allFailed", { count: total }),
      });
    } else {
      pushNotification({
        kind: "warning",
        message: t("upload.summary.partial", { ok, ng }),
      });
    }
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError(null);

    if (mode === "file") {
      await handleFileSubmit();
      return;
    }

    const trimmedName = name.trim();
    if (!trimmedName) {
      setError(t("upload.errors.nameRequired"));
      return;
    }
    setSubmitting(true);
    try {
      await createTextFile({
        name: trimmedName,
        text,
        parentId,
        parentKind,
      });
      await Promise.all([
        onSuccess(),
        invalidateAfterMutation(mutate, { library: true, dashboard: true }),
      ]);
      onClose();
    } catch (err) {
      setError(translateTopError(err));
    } finally {
      setSubmitting(false);
    }
  };

  const pendingCount = items.filter((it) => it.status === "pending").length;
  const hasItems = items.length > 0;
  const submitLabel = submitting
    ? t("upload.submitting")
    : mode === "file" && hasItems
      ? t("upload.submitMultiple", { count: pendingCount || items.length })
      : t("upload.submit");
  const submitDisabled =
    submitting || (mode === "file" && pendingCount === 0);

  return (
    <Dialog open={open} onClose={onClose} ariaLabel={t("upload.title")}>
      <form onSubmit={handleSubmit} class="flex flex-col">
        <div class="p-6 border-b border-border-dark">
          <h2 class="heading-h2">{t("upload.title")}</h2>
        </div>

        <div class="px-6 pt-4">
          <div class="inline-flex bg-surface-container-high rounded-lg p-1 gap-1">
            <button
              type="button"
              onClick={() => setMode("file")}
              class={`px-4 py-1.5 rounded-md text-sm font-medium cursor-pointer border-none ${
                mode === "file"
                  ? "bg-overlay-soft text-text-primary"
                  : "bg-transparent text-text-secondary hover:text-text-primary"
              }`}
            >
              {t("upload.tabs.file")}
            </button>
            <button
              type="button"
              onClick={() => setMode("text")}
              class={`px-4 py-1.5 rounded-md text-sm font-medium cursor-pointer border-none ${
                mode === "text"
                  ? "bg-overlay-soft text-text-primary"
                  : "bg-transparent text-text-secondary hover:text-text-primary"
              }`}
            >
              {t("upload.tabs.text")}
            </button>
          </div>
        </div>

        <div class="p-6 flex flex-col gap-4">
          {mode === "file" && (
            <div class="flex flex-col gap-3">
              <div
                onClick={() => fileInputRef.current?.click()}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onDrop={handleDrop}
                class={`w-full border border-dashed rounded-lg px-4 py-6 text-center cursor-pointer flex flex-col items-center gap-2 ${
                  isDragging
                    ? "border-overlay-strong bg-overlay-faint"
                    : "border-border-dark hover:border-overlay-strong hover:bg-overlay-faint"
                }`}
              >
                <span class="material-symbols-outlined text-text-secondary text-[28px]">
                  upload_file
                </span>
                <span class="text-sm text-text-secondary">
                  {isDragging
                    ? t("upload.file.dropHere")
                    : hasItems
                      ? t("upload.file.ctaAdd")
                      : t("upload.file.cta")}
                </span>
                <span class="text-xs text-text-disabled">
                  {t("upload.file.hint")}
                </span>
              </div>
              <input
                ref={fileInputRef}
                type="file"
                multiple
                class="hidden"
                onChange={handleFileChange}
              />

              {hasItems && (
                <ul class="flex flex-col gap-1 max-h-64 overflow-y-auto">
                  {items.map((it) => (
                    <li
                      key={it.id}
                      class="flex items-center gap-3 px-3 py-2 rounded-md bg-surface-container-low/60"
                    >
                      <span class="material-symbols-outlined text-text-secondary text-[20px]">
                        description
                      </span>
                      <div class="flex-1 min-w-0">
                        <div class="text-sm text-text-primary truncate">
                          {it.file.name}
                        </div>
                        <div class="text-xs text-text-disabled">
                          {formatBytes(it.file.size)}
                        </div>
                      </div>
                      <StatusChip item={it} />
                      {it.status === "pending" && (
                        <button
                          type="button"
                          onClick={() => handleRemove(it.id)}
                          aria-label={t("upload.file.remove")}
                          class="bg-transparent border-none text-text-secondary hover:text-text-primary cursor-pointer p-1"
                        >
                          <span class="material-symbols-outlined text-[18px]">
                            close
                          </span>
                        </button>
                      )}
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}

          {mode === "text" && (
            <>
              <div class="flex flex-col gap-2">
                <label
                  for="upload-text-name"
                  class="text-caption font-bold text-text-secondary"
                >
                  {t("upload.text.name")}
                </label>
                <input
                  id="upload-text-name"
                  type="text"
                  value={name}
                  onInput={(e) =>
                    setName((e.target as HTMLInputElement).value)
                  }
                  placeholder={t("upload.text.namePlaceholder")}
                  maxLength={256}
                  required
                  class="input-field"
                />
              </div>
              <div class="flex flex-col gap-2">
                <label
                  for="upload-text-body"
                  class="text-caption font-bold text-text-secondary"
                >
                  {t("upload.text.body")}
                </label>
                <textarea
                  id="upload-text-body"
                  value={text}
                  onInput={(e) =>
                    setText((e.target as HTMLTextAreaElement).value)
                  }
                  placeholder={t("upload.text.bodyPlaceholder")}
                  rows={8}
                  class="textarea-field font-mono"
                />
              </div>
            </>
          )}

          {error && <p class="text-sm text-danger">{error}</p>}
        </div>

        <div class="p-6 bg-surface-container-low/50 flex items-center justify-end gap-3 border-t border-border-dark">
          <button
            type="button"
            onClick={onClose}
            disabled={submitting}
            class="btn-ghost"
          >
            {t("upload.cancel")}
          </button>
          <button type="submit" disabled={submitDisabled} class="btn-primary">
            {submitLabel}
          </button>
        </div>
      </form>
    </Dialog>
  );
}

function StatusChip({ item }: { item: UploadItem }) {
  const { t } = useTranslation();
  const base =
    "text-xs px-2 py-0.5 rounded-full flex items-center gap-1 whitespace-nowrap";
  if (item.status === "pending") {
    return (
      <span class={`${base} bg-surface-container-high text-text-secondary`}>
        {t("upload.status.pending")}
      </span>
    );
  }
  if (item.status === "uploading") {
    return (
      <span class={`${base} bg-surface-container-high text-text-primary`}>
        <span class="material-symbols-outlined text-[14px] animate-spin">
          progress_activity
        </span>
        {t("upload.status.uploading")}
      </span>
    );
  }
  if (item.status === "done") {
    return (
      <span class={`${base} bg-success/15 text-success`}>
        <span class="material-symbols-outlined text-[14px]">check</span>
        {t("upload.status.done")}
      </span>
    );
  }
  return (
    <span class={`${base} bg-danger/15 text-danger`}>
      <span class="material-symbols-outlined text-[14px]">error</span>
      {t(`upload.errors.${item.errorKey ?? "failed"}`)}
    </span>
  );
}
