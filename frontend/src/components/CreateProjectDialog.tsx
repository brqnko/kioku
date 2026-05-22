import { useState, useEffect, useRef } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { kyInstance } from "../api/mutator";
import { invalidateAfterMutation } from "../utils/swrCache";
import { Dialog } from "./Dialog";
import type {
  CreateProject200,
  CreateProjectBody,
} from "../api/generated/backend.schemas";

interface CreateProjectDialogProps {
  open: boolean;
  onClose: () => void;
}

export function CreateProjectDialog({
  open,
  onClose,
}: CreateProjectDialogProps) {
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
      setError(t("createProject.errors.nameRequired"));
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      const body: CreateProjectBody = {
        name: trimmed,
        description: description.trim(),
      };
      await kyInstance
        .post("projects", { json: body })
        .json<CreateProject200>();
      await invalidateAfterMutation(mutate, { library: true, dashboard: true });
      onClose();
    } catch {
      setError(t("createProject.errors.failed"));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onClose={onClose} ariaLabel={t("createProject.title")}>
      <form onSubmit={handleSubmit} class="flex flex-col">
        <div class="p-6 border-b border-border-dark">
          <h2 class="heading-h2">{t("createProject.title")}</h2>
        </div>

        <div class="p-6 flex flex-col gap-6">
          <div class="flex flex-col gap-2">
            <label
              for="project-name"
              class="text-caption font-bold text-text-secondary"
            >
              {t("createProject.fields.name")}
            </label>
            <input
              ref={nameRef}
              id="project-name"
              type="text"
              value={name}
              onInput={(e) => setName((e.target as HTMLInputElement).value)}
              placeholder={t("createProject.placeholders.name")}
              maxLength={256}
              required
              class="input-field"
            />
          </div>

          <div class="flex flex-col gap-2">
            <label
              for="project-description"
              class="text-caption font-bold text-text-secondary"
            >
              {t("createProject.fields.description")}{" "}
              <span class="text-text-disabled font-normal">
                {t("createProject.fields.optional")}
              </span>
            </label>
            <textarea
              id="project-description"
              value={description}
              onInput={(e) =>
                setDescription((e.target as HTMLTextAreaElement).value)
              }
              placeholder={t("createProject.placeholders.description")}
              rows={3}
              maxLength={512}
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
            {t("createProject.cancel")}
          </button>
          <button type="submit" disabled={submitting} class="btn-primary">
            {submitting
              ? t("createProject.submitting")
              : t("createProject.submit")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
