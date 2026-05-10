import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { RowActionMenu } from "./RowActionMenu";
import { Dialog } from "./Dialog";
import { kyInstance } from "../api/mutator";
import type { ListProjects200ItemsItem } from "../api/generated/backend.schemas";

const RELATIVE_THRESHOLDS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 60 * 60 * 24 * 365],
  ["month", 60 * 60 * 24 * 30],
  ["day", 60 * 60 * 24],
  ["hour", 60 * 60],
  ["minute", 60],
];

function formatRelative(iso: string, locale: string) {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const diffSec = Math.round((then - Date.now()) / 1000);
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  for (const [unit, sec] of RELATIVE_THRESHOLDS) {
    if (Math.abs(diffSec) >= sec) {
      return rtf.format(Math.round(diffSec / sec), unit);
    }
  }
  return rtf.format(diffSec, "second");
}

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
      onRefresh();
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
      onRefresh();
      setDeleteOpen(false);
    } catch {
      // keep dialog open
    } finally {
      setDeleteSubmitting(false);
    }
  };

  return (
    <>
      <div class="group relative flex flex-col min-h-[160px] rounded-xl border border-border-subtle bg-surface-dark hover:border-text-disabled shadow-[0_2px_8px_rgba(0,0,0,0.2)]">
        {/* Full-card link — sits behind content */}
        <a
          href={href}
          class="absolute inset-0 rounded-xl"
          aria-label={project.name}
        />

        {/* Card content — pointer-events-none so the link behind catches clicks */}
        <div class="relative pointer-events-none flex flex-col flex-1 p-4">
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
          <h2 class="text-[22px] font-bold leading-tight">
            {t("renameItem.title")}
          </h2>
          <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium text-text-secondary">
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
                class="w-full h-9 bg-surface-dark border border-border-subtle rounded-md px-3 text-sm text-text-primary placeholder:text-text-disabled focus:outline-none focus:border-accent-blue"
              />
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium text-text-secondary">
                {t("renameItem.descriptionLabel")}
              </label>
              <input
                type="text"
                value={renameDesc}
                onInput={(e) =>
                  setRenameDesc((e.target as HTMLInputElement).value)
                }
                placeholder={t("renameItem.descriptionPlaceholder")}
                class="w-full h-9 bg-surface-dark border border-border-subtle rounded-md px-3 text-sm text-text-primary placeholder:text-text-disabled focus:outline-none focus:border-accent-blue"
              />
            </div>
          </div>
          <div class="flex justify-end gap-3">
            <button
              type="button"
              onClick={() => setRenameOpen(false)}
              class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary border border-border-subtle rounded-lg hover:bg-overlay-faint cursor-pointer bg-transparent"
            >
              {t("renameItem.cancel")}
            </button>
            <button
              type="button"
              onClick={handleRenameSubmit}
              disabled={renameSubmitting || !renameInput.trim()}
              class="px-4 py-2 text-sm font-bold bg-cta text-cta-fg rounded-lg hover:bg-cta-hover cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
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
          <h2 class="text-[22px] font-bold leading-tight">
            {t("deleteItem.title")}
          </h2>
          <p class="text-sm text-text-secondary">
            {t("deleteItem.body", { name: project.name })}
          </p>
          <div class="flex justify-end gap-3">
            <button
              type="button"
              onClick={() => setDeleteOpen(false)}
              class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary border border-border-subtle rounded-lg hover:bg-overlay-faint cursor-pointer bg-transparent"
            >
              {t("deleteItem.cancel")}
            </button>
            <button
              type="button"
              onClick={handleDeleteConfirm}
              disabled={deleteSubmitting}
              class="px-4 py-2 text-sm font-bold bg-danger/10 text-danger hover:bg-danger/20 rounded-lg cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
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
