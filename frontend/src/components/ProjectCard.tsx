import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { RowActionMenu } from "./RowActionMenu";
import { Dialog } from "./Dialog";
import { kyInstance } from "../api/mutator";
import { formatRelative } from "../utils/datetime";
import { invalidateAfterMutation } from "../utils/swrCache";
import type { ListProjects200ItemsItem } from "../api/generated/backend.schemas";

interface ProjectCardProps {
  project: ListProjects200ItemsItem;
  href: string;
  noDescriptionKey: string;
  lastUpdatedKey: string;
  onRefresh: () => void;
}

export function ProjectCard({
  project,
  href,
  noDescriptionKey,
  lastUpdatedKey,
  onRefresh,
}: ProjectCardProps) {
  const { t, i18n } = useTranslation();
  const { mutate } = useSWRConfig();

  const [renameOpen, setRenameOpen] = useState(false);
  const [renameInput, setRenameInput] = useState("");
  const [renameDesc, setRenameDesc] = useState("");
  const [renameSubmitting, setRenameSubmitting] = useState(false);

  const [deleteOpen, setDeleteOpen] = useState(false);
  const [deleteSubmitting, setDeleteSubmitting] = useState(false);

  const handleRenameOpen = () => {
    setRenameInput(project.name);
    setRenameDesc(project.description ?? "");
    setRenameOpen(true);
  };

  const handleRenameSubmit = async () => {
    if (!renameInput.trim() || renameSubmitting) return;
    setRenameSubmitting(true);
    try {
      await kyInstance.patch(`projects/${project.id}`, {
        json: {
          name: renameInput.trim(),
          description: renameDesc.trim() || null,
        },
      });
      await Promise.all([
        onRefresh(),
        invalidateAfterMutation(mutate, {
          childListings: true,
          library: true,
          dashboard: true,
        }),
      ]);
      setRenameOpen(false);
    } catch {
      // keep dialog open
    } finally {
      setRenameSubmitting(false);
    }
  };

  const handleDeleteConfirm = async () => {
    if (deleteSubmitting) return;
    setDeleteSubmitting(true);
    try {
      await kyInstance.delete(`projects/${project.id}`);
      await Promise.all([
        onRefresh(),
        invalidateAfterMutation(mutate, {
          childListings: true,
          library: true,
          dashboard: true,
        }),
      ]);
      setDeleteOpen(false);
    } catch {
      // keep dialog open
    } finally {
      setDeleteSubmitting(false);
    }
  };

  return (
    <>
      <div class="group relative flex flex-col min-h-[160px] rounded-[12px] border border-border-subtle bg-surface-dark hover:border-text-disabled shadow-[0_1px_3px_rgba(0,0,0,0.1)]">
        {/* Full-card link — sits behind content */}
        <a
          href={href}
          class="absolute inset-0 rounded-[12px]"
          aria-label={project.name}
        />

        {/* Card content — pointer-events-none so the link behind catches clicks */}
        <div class="relative pointer-events-none flex flex-col flex-1 p-6">
          <h3 class="text-base font-medium text-text-primary mb-1 line-clamp-1 pr-6">
            {project.name}
          </h3>
          <p class="text-sm text-text-secondary line-clamp-2 mb-auto">
            {project.description || t(noDescriptionKey)}
          </p>
          <div class="mt-4 pt-2 border-t border-border-subtle flex items-center">
            <span class="text-xs text-text-disabled flex items-center gap-1">
              <span class="material-symbols-outlined text-[14px]">update</span>
              {t(lastUpdatedKey, {
                time: formatRelative(project.last_seen_at, i18n.language),
              })}
            </span>
          </div>
        </div>

        {/* Menu button — pointer-events-auto to sit above the link */}
        <div class="absolute top-2 right-2 pointer-events-auto z-10">
          <RowActionMenu
            icon="more_vert"
            ariaLabel={t("renameItem.menu") + " / " + t("deleteItem.menu")}
            onEdit={handleRenameOpen}
            onDelete={() => setDeleteOpen(true)}
          />
        </div>
      </div>

      {/* Rename dialog */}
      <Dialog
        open={renameOpen}
        onClose={() => setRenameOpen(false)}
        ariaLabel={t("renameItem.title")}
        maxWidth="max-w-[420px]"
      >
        <div class="p-6 flex flex-col gap-5">
          <h2 class="heading-h2">{t("renameItem.title")}</h2>
          <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-1.5">
              <label class="text-caption font-medium text-text-secondary">
                {t("renameItem.label")}
              </label>
              <input
                type="text"
                value={renameInput}
                onInput={(e) =>
                  setRenameInput((e.target as HTMLInputElement).value)
                }
                onKeyDown={(e) => e.key === "Enter" && handleRenameSubmit()}
                autofocus
                class="input-field"
              />
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-caption font-medium text-text-secondary">
                {t("renameItem.descriptionLabel")}
              </label>
              <input
                type="text"
                value={renameDesc}
                onInput={(e) =>
                  setRenameDesc((e.target as HTMLInputElement).value)
                }
                placeholder={t("renameItem.descriptionPlaceholder")}
                class="input-field"
              />
            </div>
          </div>
          <div class="flex justify-end gap-3">
            <button
              type="button"
              onClick={() => setRenameOpen(false)}
              class="btn-secondary"
            >
              {t("renameItem.cancel")}
            </button>
            <button
              type="button"
              onClick={handleRenameSubmit}
              disabled={renameSubmitting || !renameInput.trim()}
              class="btn-primary"
            >
              {renameSubmitting
                ? t("renameItem.submitting")
                : t("renameItem.submit")}
            </button>
          </div>
        </div>
      </Dialog>

      {/* Delete dialog */}
      <Dialog
        open={deleteOpen}
        onClose={() => setDeleteOpen(false)}
        ariaLabel={t("deleteItem.title")}
        maxWidth="max-w-[420px]"
      >
        <div class="p-6 flex flex-col gap-5">
          <h2 class="heading-h2">{t("deleteItem.title")}</h2>
          <p class="text-body text-text-secondary">
            {t("deleteItem.body", { name: project.name })}
          </p>
          <div class="flex justify-end gap-3">
            <button
              type="button"
              onClick={() => setDeleteOpen(false)}
              class="btn-secondary"
            >
              {t("deleteItem.cancel")}
            </button>
            <button
              type="button"
              onClick={handleDeleteConfirm}
              disabled={deleteSubmitting}
              class="btn-danger"
            >
              {deleteSubmitting
                ? t("deleteItem.submitting")
                : t("deleteItem.submit")}
            </button>
          </div>
        </div>
      </Dialog>
    </>
  );
}
