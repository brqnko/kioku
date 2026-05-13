import { useEffect, useRef, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { Dialog } from "./Dialog";
import { createTextFile, uploadFile } from "../api/upload";
import { invalidateAfterMutation } from "../utils/swrCache";

interface UploadDialogProps {
  open: boolean;
  onClose: () => void;
  parentId: string;
  parentKind: "project" | "folder";
  onSuccess: () => unknown | Promise<unknown>;
}

type Mode = "file" | "text";

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
  const [file, setFile] = useState<File | null>(null);
  const [name, setName] = useState("");
  const [text, setText] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setMode("file");
      setFile(null);
      setName("");
      setText("");
      setError(null);
      setSubmitting(false);
    }
  }, [open]);

  const handleFileChange = (e: Event) => {
    const input = e.target as HTMLInputElement;
    setFile(input.files?.[0] ?? null);
  };

  const translateError = (err: unknown): string => {
    if (err instanceof Error && err.message === "file_too_large") {
      return t("upload.errors.tooLarge");
    }
    return t("upload.errors.failed");
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError(null);

    if (mode === "file") {
      if (!file) {
        setError(t("upload.errors.fileRequired"));
        return;
      }
      setSubmitting(true);
      try {
        await uploadFile({ file, parentId, parentKind });
        await Promise.all([
          onSuccess(),
          invalidateAfterMutation(mutate, { library: true, dashboard: true }),
        ]);
        onClose();
      } catch (err) {
        setError(translateError(err));
      } finally {
        setSubmitting(false);
      }
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
      setError(translateError(err));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} ariaLabel={t("upload.title")}>
      <form onSubmit={handleSubmit} class="flex flex-col">
        <div class="p-6 border-b border-border-dark">
          <h2 class="text-[22px] leading-[1.27] font-bold tracking-tight text-text-primary">
            {t("upload.title")}
          </h2>
        </div>

        <div class="px-6 pt-4">
          <div class="inline-flex bg-surface-container-high rounded-lg p-1 gap-1">
            <button
              type="button"
              onClick={() => setMode("file")}
              class={`px-4 py-1.5 rounded-md text-sm font-medium cursor-pointer ${
                mode === "file"
                  ? "bg-overlay-soft text-text-primary"
                  : "text-text-muted-dark hover:text-text-primary"
              }`}
            >
              {t("upload.tabs.file")}
            </button>
            <button
              type="button"
              onClick={() => setMode("text")}
              class={`px-4 py-1.5 rounded-md text-sm font-medium cursor-pointer ${
                mode === "text"
                  ? "bg-overlay-soft text-text-primary"
                  : "text-text-muted-dark hover:text-text-primary"
              }`}
            >
              {t("upload.tabs.text")}
            </button>
          </div>
        </div>

        <div class="p-6 flex flex-col gap-4">
          {mode === "file" && (
            <div class="flex flex-col gap-2">
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                class="w-full border border-dashed border-border-dark rounded-lg px-4 py-8 text-center hover:border-overlay-strong hover:bg-overlay-faint cursor-pointer flex flex-col items-center gap-2"
              >
                <span class="material-symbols-outlined text-text-muted-dark text-[28px]">
                  upload_file
                </span>
                {file ? (
                  <span class="text-sm text-text-primary">{file.name}</span>
                ) : (
                  <span class="text-sm text-text-muted-dark">
                    {t("upload.file.cta")}
                  </span>
                )}
                <span class="text-xs text-text-disabled">
                  {t("upload.file.hint")}
                </span>
              </button>
              <input
                ref={fileInputRef}
                type="file"
                class="hidden"
                onChange={handleFileChange}
              />
            </div>
          )}

          {mode === "text" && (
            <>
              <div class="flex flex-col gap-2">
                <label
                  for="upload-text-name"
                  class="text-sm font-bold text-text-muted-dark"
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
                  class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50"
                />
              </div>
              <div class="flex flex-col gap-2">
                <label
                  for="upload-text-body"
                  class="text-sm font-bold text-text-muted-dark"
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
                  class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50 resize-y font-mono"
                />
              </div>
            </>
          )}

          {error && <p class="text-sm text-danger">{error}</p>}
        </div>

        <div class="p-6 bg-surface-container-low/50 flex items-center justify-end gap-4 border-t border-border-dark">
          <button
            type="button"
            onClick={onClose}
            disabled={submitting}
            class="px-6 py-2.5 rounded-lg text-sm font-bold text-text-muted-dark hover:text-text-primary hover:bg-overlay-faint cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {t("upload.cancel")}
          </button>
          <button
            type="submit"
            disabled={submitting}
            class="px-6 py-2.5 bg-cta text-cta-fg rounded-lg text-sm font-bold hover:bg-cta-hover shadow-sm cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {submitting ? t("upload.submitting") : t("upload.submit")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
