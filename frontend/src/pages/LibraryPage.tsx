import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { CreateProjectTile } from "../components/CreateProjectTile";
import { ProjectCard } from "../components/ProjectCard";
import { useLibrary } from "../hooks/useLibrary";
import type { ListProjects200ItemsItem } from "../api/generated/backend.schemas";

export default function LibraryPage() {
  const { t } = useTranslation();
  const { items, error, isLoading, hasMore, loadingMore, loadMore, mutate } =
    useLibrary();

  const projects = items;

  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-[var(--sidebar-width)] p-4 tablet:p-8 min-h-[calc(100vh-3.5rem)] overflow-y-auto transition-[margin-left] duration-200 ease-in-out">
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
          <CreateProjectTile labelKey="library.newProject" />

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
            <ProjectCard
              key={project.id}
              project={project}
              href={`/projects/${project.id}`}
              noDescriptionKey="library.noDescription"
              lastUpdatedKey="library.lastUpdated"
              onRefresh={() => mutate()}
            />
          ))}
        </div>

        {hasMore && (
          <div class="mt-6 flex justify-center">
            <button
              type="button"
              onClick={loadMore}
              disabled={loadingMore}
              class="px-4 py-2 border border-overlay-medium text-text-primary text-sm font-semibold rounded-lg hover:bg-overlay-faint cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loadingMore ? t("library.loading") : t("library.loadMore")}
            </button>
          </div>
        )}
      </main>
    </div>
  );
}
