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
    const baselineName = initialName.trim();
    const baselineDesc = initialDescription.trim();
    if (!trimmedName) {
      setError(t("renameItem.errors.nameRequired"));
      return;
    }
    if (trimmedName === baselineName && trimmedDesc === baselineDesc) {
      onClose();
      return;
    }
    const body: UpdatePodcastBody = {};
    if (trimmedName !== baselineName) body.name = trimmedName;
    if (trimmedDesc !== baselineDesc) body.description = trimmedDesc;

    setSubmitting(true);
    setError(null);
    try {
      await kyInstance.patch(`projects/${projectId}/podcasts/${podcastId}`, {
        json: body,
      });
      await onSuccess();
      onClose();
    } catch {
      setError(t("renameItem.errors.failed"));
      setSubmitting(false);
    }
  };

  return (
    <Dialog
      open={open}
      onClose={handleClose}
      ariaLabel={t("renameItem.title")}
      maxWidth="max-w-[480px]"
    >
      <form onSubmit={handleSubmit} class="p-6 flex flex-col gap-4">
        <h2 class="heading-h2">{t("renameItem.title")}</h2>

        <div class="flex flex-col gap-2">
          <label
            for="edit-podcast-name"
            class="text-caption font-bold text-text-secondary"
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
            class="input-field"
          />
        </div>

        <div class="flex flex-col gap-2">
          <label
            for="edit-podcast-description"
            class="text-caption font-bold text-text-secondary"
          >
            {t("renameItem.descriptionLabel")}
          </label>
          <textarea
            id="edit-podcast-description"
            value={description}
            onInput={(e) =>
              setDescription((e.target as HTMLTextAreaElement).value)
            }
            placeholder={t("renameItem.descriptionPlaceholder")}
            maxLength={1024}
            rows={3}
            disabled={submitting}
            class="textarea-field"
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
            class="btn-ghost"
          >
            {t("renameItem.cancel")}
          </button>
          <button type="submit" disabled={submitting} class="btn-primary">
            {submitting ? t("renameItem.submitting") : t("renameItem.submit")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
