import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { CreateProjectTile } from "../components/CreateProjectTile";
import { useLibrary } from "../hooks/useLibrary";
import { useDocumentHead } from "../hooks/useDocumentHead";
import { formatRelative } from "../utils/datetime";
import type { ListProjects200ItemsItem } from "../api/generated/backend.schemas";

export default function PodcastPage() {
  const { t, i18n } = useTranslation();
  useDocumentHead({ title: "Podcast — kioku", robots: "noindex,nofollow" });
  const { items, error, isLoading, hasMore, loadingMore, loadMore } =
    useLibrary();

  const projects = items;

  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-[var(--sidebar-width)] p-4 tablet:p-8 h-[calc(100vh-3.5rem)] overflow-y-auto transition-[margin-left] duration-200 ease-in-out">
        <header class="flex flex-col gap-2 mb-6">
          <h1 class="heading-h2">{t("podcast.selectProject.title")}</h1>
          <p class="text-body text-text-secondary">
            {t("podcast.selectProject.subtitle")}
          </p>
        </header>

        {error && (
          <p class="text-sm text-danger mb-4">{t("podcast.errors.load")}</p>
        )}

        {isLoading && projects.length === 0 && (
          <p class="text-sm text-text-secondary">{t("podcast.loading")}</p>
        )}

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          <CreateProjectTile />

          {!isLoading && !error && projects.length === 0 && (
            <div class="col-span-full text-sm text-text-secondary">
              {t("podcast.selectProject.empty")}
            </div>
          )}

          {projects.map((project: ListProjects200ItemsItem) => (
            <a
              key={project.id}
              href={`/projects/${project.id}/podcasts`}
              class="flex flex-col min-h-[160px] p-6 rounded-[12px] border border-border-subtle bg-surface-dark hover:border-text-disabled shadow-[0_1px_3px_rgba(0,0,0,0.1)] no-underline text-inherit"
            >
              <h3 class="text-base font-medium text-text-primary mb-1 line-clamp-1">
                {project.name}
              </h3>
              <p class="text-sm text-text-secondary line-clamp-2 mb-auto">
                {project.description ||
                  t("podcast.selectProject.noDescription")}
              </p>
              <div class="mt-4 pt-2 border-t border-border-subtle flex items-center justify-between">
                <span class="text-xs text-text-disabled flex items-center gap-1">
                  <span class="material-symbols-outlined text-[14px]">
                    update
                  </span>
                  {t("podcast.selectProject.lastUpdated", {
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
              class="btn-secondary"
            >
              {loadingMore ? t("podcast.loading") : t("podcast.loadMore")}
            </button>
          </div>
        )}
      </main>
    </div>
  );
}
