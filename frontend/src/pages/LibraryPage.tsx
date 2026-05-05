import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { CreateProjectDialog } from "../components/CreateProjectDialog";
import { useLibrary } from "../hooks/useLibrary";
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

export default function LibraryPage() {
  const { t, i18n } = useTranslation();
  const { items, error, isLoading, hasMore, loadingMore, loadMore } =
    useLibrary();
  const [dialogOpen, setDialogOpen] = useState(false);

  const projects = items;

  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-64 p-8 min-h-[calc(100vh-3.5rem)] overflow-y-auto">
        <header class="flex flex-col gap-2 mb-6">
          <h1 class="text-[22px] leading-[1.27] font-bold tracking-tight">
            {t("library.title")}
          </h1>
          <p class="text-base text-text-secondary">{t("library.subtitle")}</p>
        </header>

        {error && (
          <p class="text-sm text-danger mb-4">{t("library.error")}</p>
        )}

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          <button
            type="button"
            onClick={() => setDialogOpen(true)}
            class="group flex flex-col items-center justify-center gap-2 min-h-[160px] p-6 rounded-xl border border-dashed border-border-subtle bg-transparent hover:bg-overlay-faint hover:border-text-disabled transition-all duration-200 cursor-pointer text-center"
          >
            <div class="w-10 h-10 rounded-full bg-surface-dark flex items-center justify-center">
              <span class="material-symbols-outlined text-text-secondary group-hover:text-text-primary text-[20px]">
                add
              </span>
            </div>
            <span class="text-sm text-text-secondary group-hover:text-text-primary font-medium">
              {t("library.newProject")}
            </span>
          </button>

          {isLoading && projects.length === 0 && (
            <div class="col-span-full text-sm text-text-secondary">
              {t("library.loading")}
            </div>
          )}

          {!isLoading && !error && projects.length === 0 && (
            <div class="col-span-full text-sm text-text-secondary">
              {t("library.empty")}
            </div>
          )}

          {projects.map((project: ListProjects200ItemsItem) => (
            <a
              key={project.id}
              href={`/projects/${project.id}`}
              class="group flex flex-col min-h-[160px] p-4 rounded-xl border border-border-subtle bg-surface-dark hover:border-text-disabled transition-colors duration-200 shadow-[0_2px_8px_rgba(0,0,0,0.2)] hover:shadow-[0_4px_12px_rgba(0,0,0,0.3)] no-underline text-inherit"
            >
              <div
                class="w-10 h-10 rounded-lg bg-overlay-faint border border-overlay-soft flex items-center justify-center mb-4 text-text-secondary text-lg font-bold"
                aria-hidden="true"
              >
                ?
              </div>
              <h3 class="text-base font-medium text-text-primary mb-1 line-clamp-1">
                {project.name}
              </h3>
              <p class="text-sm text-text-secondary line-clamp-2 mb-auto">
                {project.description || t("library.noDescription")}
              </p>
              <div class="mt-4 pt-2 border-t border-border-subtle flex items-center justify-between">
                <span class="text-xs text-text-disabled flex items-center gap-1">
                  <span class="material-symbols-outlined text-[14px]">
                    update
                  </span>
                  {t("library.lastUpdated", {
                    time: formatRelative(project.last_seen_at, i18n.language),
                  })}
                </span>
              </div>
            </a>
          ))}
        </div>

        {hasMore && (
          <div class="mt-6 flex justify-center">
            <button
              type="button"
              onClick={loadMore}
              disabled={loadingMore}
              class="px-4 py-2 border border-overlay-medium text-text-primary text-sm font-semibold rounded-lg hover:bg-overlay-faint transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loadingMore ? t("library.loading") : t("library.loadMore")}
            </button>
          </div>
        )}
      </main>
      <CreateProjectDialog
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
      />
    </div>
  );
}
