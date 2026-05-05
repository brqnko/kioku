import { useState, useEffect, useRef } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { kyInstance } from "../api/mutator";
import { Dialog } from "./Dialog";
import type {
  CreateProject200,
  CreateProjectBody,
} from "../api/generated/backend.schemas";

interface CreateProjectDialogProps {
  open: boolean;
  onClose: () => void;
}

export function CreateProjectDialog({ open, onClose }: CreateProjectDialogProps) {
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
      await Promise.all([
        mutate("users/me/dashboard"),
        mutate(
          (key) => Array.isArray(key) && key[0] === "library:page",
        ),
      ]);
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
          <h2 class="text-[22px] leading-[1.27] font-bold tracking-tight text-text-primary">
            {t("createProject.title")}
          </h2>
        </div>

        <div class="p-6 flex flex-col gap-6">
          <div class="flex flex-col gap-2">
            <label
              for="project-name"
              class="text-sm font-bold text-text-muted-dark"
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
              class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50 transition-all"
            />
          </div>

          <div class="flex flex-col gap-2">
            <label
              for="project-description"
              class="text-sm font-bold text-text-muted-dark"
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
              class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50 transition-all resize-none"
            />
          </div>

          {error && <p class="text-sm text-danger">{error}</p>}
        </div>

        <div class="p-6 bg-surface-container-low/50 flex items-center justify-end gap-4 border-t border-border-dark">
          <button
            type="button"
            onClick={onClose}
            disabled={submitting}
            class="px-6 py-2.5 rounded-lg text-sm font-bold text-text-muted-dark hover:text-text-primary hover:bg-overlay-faint transition-all cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {t("createProject.cancel")}
          </button>
          <button
            type="submit"
            disabled={submitting}
            class="px-6 py-2.5 bg-cta text-cta-fg rounded-lg text-sm font-bold hover:bg-cta-hover transition-colors shadow-sm cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {submitting ? t("createProject.submitting") : t("createProject.submit")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
