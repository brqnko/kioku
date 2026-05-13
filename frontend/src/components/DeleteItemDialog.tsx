import { useEffect, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { kyInstance } from "../api/mutator";
import { invalidateAfterMutation } from "../utils/swrCache";
import { Dialog } from "./Dialog";

interface DeleteItemDialogProps {
  open: boolean;
  onClose: () => void;
  kind?: "file" | "folder";
  id: string;
  name: string;
  onSuccess: () => unknown | Promise<unknown>;
  customPath?: string;
}

export function DeleteItemDialog({
  open,
  onClose,
  kind,
  id,
  name,
  onSuccess,
  customPath,
}: DeleteItemDialogProps) {
  const { t } = useTranslation();
  const { mutate } = useSWRConfig();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setSubmitting(false);
      setError(null);
    }
  }, [open]);

  const handleDelete = async () => {
    setSubmitting(true);
    setError(null);
    try {
      const path = customPath ?? (kind === "file" ? `files/${id}` : `folders/${id}`);
      await kyInstance.delete(path);
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
      setError(t("deleteItem.errors.failed"));
      setSubmitting(false);
    }
  };

  const handleClose = () => {
    if (!submitting) onClose();
  };

  return (
    <Dialog
      open={open}
      onClose={handleClose}
      ariaLabel={t("deleteItem.title")}
      maxWidth="max-w-[440px]"
    >
      <div class="p-6 flex flex-col gap-4">
        <h2 class="heading-h2">{t("deleteItem.title")}</h2>
        <p class="text-body text-text-secondary">
          {t("deleteItem.body", { name })}
        </p>

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
            {t("deleteItem.cancel")}
          </button>
          <button
            type="button"
            onClick={handleDelete}
            disabled={submitting}
            class="btn-danger"
          >
            {submitting ? t("deleteItem.submitting") : t("deleteItem.submit")}
          </button>
        </div>
      </div>
    </Dialog>
  );
}
