import { useEffect, useRef, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { kyInstance } from "../api/mutator";
import { invalidateAfterMutation } from "../utils/swrCache";
import { Dialog } from "./Dialog";

interface RenameItemDialogProps {
  open: boolean;
  onClose: () => void;
  kind: "file" | "folder" | "project";
  id: string;
  initialName: string;
  initialDescription?: string | null;
  onSuccess: () => unknown | Promise<unknown>;
}

export function RenameItemDialog({
  open,
  onClose,
  kind,
  id,
  initialName,
  initialDescription,
  onSuccess,
}: RenameItemDialogProps) {
  const { t } = useTranslation();
  const { mutate } = useSWRConfig();
  const [name, setName] = useState(initialName);
  const [description, setDescription] = useState(initialDescription ?? "");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setName(initialName);
      setDescription(initialDescription ?? "");
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
    const initialDesc = (initialDescription ?? "").trim();
    if (!trimmedName) {
      setError(t("renameItem.errors.nameRequired"));
      return;
    }
    const nameChanged = trimmedName !== initialName;
    const descChanged = trimmedDesc !== initialDesc;
    if (!nameChanged && !descChanged) {
      onClose();
      return;
    }
    const body: { name?: string; description?: string } = {};
    if (nameChanged) body.name = trimmedName;
    if (descChanged) body.description = trimmedDesc;

    setSubmitting(true);
    setError(null);
    try {
      const path =
        kind === "file"
          ? `files/${id}`
          : kind === "project"
            ? `projects/${id}`
            : `folders/${id}`;
      await kyInstance.patch(path, { json: body });
      await Promise.all([
        onSuccess(),
        invalidateAfterMutation(mutate, {
          childListings: true,
          library: true,
          dashboard: true,
        }),
      ]);
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
      <form onSubmit={handleSubmit} class="p-4 tablet:p-6 flex flex-col gap-4">
        <h2 class="text-[18px] leading-[1.3] font-bold tracking-tight text-text-primary">
          {t("renameItem.title")}
        </h2>

        <div class="flex flex-col gap-2">
          <label
            for="rename-item-name"
            class="text-sm font-bold text-text-muted-dark"
          >
            {t("renameItem.label")}
          </label>
          <input
            ref={inputRef}
            id="rename-item-name"
            type="text"
            value={name}
            onInput={(e) => setName((e.target as HTMLInputElement).value)}
            maxLength={256}
            required
            disabled={submitting}
            class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50 disabled:opacity-50"
          />
        </div>

        <div class="flex flex-col gap-2">
          <label
            for="rename-item-description"
            class="text-sm font-bold text-text-muted-dark"
          >
            {t("renameItem.descriptionLabel")}
          </label>
          <textarea
            id="rename-item-description"
            value={description}
            onInput={(e) =>
              setDescription((e.target as HTMLTextAreaElement).value)
            }
            placeholder={t("renameItem.descriptionPlaceholder")}
            maxLength={1024}
            rows={3}
            disabled={submitting}
            class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50 disabled:opacity-50 resize-y leading-[1.5]"
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
            class="px-4 py-2 rounded-lg text-sm font-bold text-text-muted-dark hover:text-text-primary hover:bg-overlay-faint cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {t("renameItem.cancel")}
          </button>
          <button
            type="submit"
            disabled={submitting}
            class="px-4 py-2 bg-cta text-cta-fg rounded-lg text-sm font-bold hover:bg-cta-hover cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {submitting ? t("renameItem.submitting") : t("renameItem.submit")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
