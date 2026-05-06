import { useEffect, useRef, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { kyInstance } from "../api/mutator";
import type { UpdatePodcastBody } from "../api/generated/backend.schemas";
import { Dialog } from "./Dialog";

interface EditPodcastDialogProps {
  open: boolean;
  onClose: () => void;
  projectId: string;
  podcastId: string;
  initialName: string;
  initialDescription: string;
  onSuccess: () => unknown | Promise<unknown>;
}

export function EditPodcastDialog({
  open,
  onClose,
  projectId,
  podcastId,
  initialName,
  initialDescription,
  onSuccess,
}: EditPodcastDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState(initialName);
  const [description, setDescription] = useState(initialDescription);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setName(initialName);
      setDescription(initialDescription);
      setError(null);
      setSubmitting(false);
      queueMicrotask(() => {
        inputRef.current?.focus();
        inputRef.current?.select();
      });
    }
  }, [open, initialName, initialDescription]);

  const handleClose = () => {
    if (!submitting) onClose();
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    const trimmedName = name.trim();
    const trimmedDesc = description.trim();
    if (!trimmedName) {
      setError(t("renameItem.errors.nameRequired"));
      return;
    }
    if (trimmedName === initialName && trimmedDesc === initialDescription.trim()) {
      onClose();
      return;
    }
    const body: UpdatePodcastBody = {};
    if (trimmedName !== initialName) body.name = trimmedName;
    if (trimmedDesc !== initialDescription.trim()) body.description = trimmedDesc;

    setSubmitting(true);
    setError(null);
    try {
      await kyInstance.patch(`projects/${projectId}/podcasts/${podcastId}`, { json: body });
      await onSuccess();
      onClose();
    } catch {
      setError(t("renameItem.errors.failed"));
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onClose={handleClose} ariaLabel={t("renameItem.title")} maxWidth="max-w-[480px]">
      <form onSubmit={handleSubmit} class="p-6 flex flex-col gap-4">
        <h2 class="text-[18px] leading-[1.3] font-bold tracking-tight text-text-primary">
          {t("renameItem.title")}
        </h2>

        <div class="flex flex-col gap-2">
          <label
            for="edit-podcast-name"
            class="text-sm font-bold text-text-muted-dark"
          >
            {t("renameItem.label")}
          </label>
          <input
            ref={inputRef}
            id="edit-podcast-name"
            type="text"
            value={name}
            onInput={(e) => setName((e.target as HTMLInputElement).value)}
            maxLength={256}
            required
            disabled={submitting}
            class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50 transition-all disabled:opacity-50"
          />
        </div>

        <div class="flex flex-col gap-2">
          <label
            for="edit-podcast-description"
            class="text-sm font-bold text-text-muted-dark"
          >
            {t("renameItem.descriptionLabel")}
          </label>
          <textarea
            id="edit-podcast-description"
            value={description}
            onInput={(e) => setDescription((e.target as HTMLTextAreaElement).value)}
            placeholder={t("renameItem.descriptionPlaceholder")}
            maxLength={1024}
            rows={3}
            disabled={submitting}
            class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50 transition-all disabled:opacity-50 resize-y leading-[1.5]"
          />
        </div>

        {error && (
          <div class="px-3 py-2 rounded-lg bg-danger/10 border border-danger/30 text-danger text-sm">
            {error}
          </div>
        )}

        <div class="flex items-center justify-end gap-3 mt-2">
          <button
            type="button"
            onClick={handleClose}
            disabled={submitting}
            class="px-4 py-2 rounded-lg text-sm font-bold text-text-muted-dark hover:text-text-primary hover:bg-overlay-faint transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {t("renameItem.cancel")}
          </button>
          <button
            type="submit"
            disabled={submitting}
            class="px-4 py-2 bg-cta text-cta-fg rounded-lg text-sm font-bold hover:bg-cta-hover transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {submitting ? t("renameItem.submitting") : t("renameItem.submit")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
