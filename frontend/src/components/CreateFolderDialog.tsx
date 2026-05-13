import { useEffect, useRef, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { kyInstance } from "../api/mutator";
import { invalidateAfterMutation } from "../utils/swrCache";
import { Dialog } from "./Dialog";
import {
  CreateFolderBodyParentKind,
  type CreateFolder200,
  type CreateFolderBody,
} from "../api/generated/backend.schemas";

interface CreateFolderDialogProps {
  open: boolean;
  onClose: () => void;
  parentId: string;
  parentKind: "project" | "folder";
  onSuccess: () => unknown | Promise<unknown>;
}

export function CreateFolderDialog({
  open,
  onClose,
  parentId,
  parentKind,
  onSuccess,
}: CreateFolderDialogProps) {
  const { t } = useTranslation();
  const { mutate } = useSWRConfig();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const nameRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) {
      setName("");
      setDescription("");
      setError(null);
      setSubmitting(false);
      queueMicrotask(() => nameRef.current?.focus());
    }
  }, [open]);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) {
      setError(t("createFolder.errors.nameRequired"));
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const body: CreateFolderBody = {
        name: trimmed,
        description: description.trim(),
        parent_id: parentId,
        parent_kind:
          parentKind === "project"
            ? CreateFolderBodyParentKind.project
            : CreateFolderBodyParentKind.folder,
      };
      await kyInstance.post("folders", { json: body }).json<CreateFolder200>();
      await Promise.all([
        onSuccess(),
        invalidateAfterMutation(mutate, { library: true, dashboard: true }),
      ]);
      onClose();
    } catch {
      setError(t("createFolder.errors.failed"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} ariaLabel={t("createFolder.title")}>
      <form onSubmit={handleSubmit} class="flex flex-col">
        <div class="p-6 border-b border-border-dark">
          <h2 class="heading-h2">{t("createFolder.title")}</h2>
        </div>

        <div class="p-6 flex flex-col gap-6">
          <div class="flex flex-col gap-2">
            <label
              for="folder-name"
              class="text-caption font-bold text-text-secondary"
            >
              {t("createFolder.fields.name")}
            </label>
            <input
              ref={nameRef}
              id="folder-name"
              type="text"
              value={name}
              onInput={(e) => setName((e.target as HTMLInputElement).value)}
              placeholder={t("createFolder.placeholders.name")}
              maxLength={256}
              required
              class="input-field"
            />
          </div>

          <div class="flex flex-col gap-2">
            <label
              for="folder-description"
              class="text-caption font-bold text-text-secondary"
            >
              {t("createFolder.fields.description")}{" "}
              <span class="text-text-disabled font-normal">
                {t("createFolder.fields.optional")}
              </span>
            </label>
            <textarea
              id="folder-description"
              value={description}
              onInput={(e) =>
                setDescription((e.target as HTMLTextAreaElement).value)
              }
              placeholder={t("createFolder.placeholders.description")}
              rows={3}
              maxLength={1024}
              class="textarea-field"
            />
          </div>

          {error && <p class="text-sm text-danger">{error}</p>}
        </div>

        <div class="p-6 bg-surface-container-low/50 flex items-center justify-end gap-3 border-t border-border-dark">
          <button
            type="button"
            onClick={onClose}
            disabled={submitting}
            class="btn-ghost"
          >
            {t("createFolder.cancel")}
          </button>
          <button type="submit" disabled={submitting} class="btn-primary">
            {submitting
              ? t("createFolder.submitting")
              : t("createFolder.submit")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
