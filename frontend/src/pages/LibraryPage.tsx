import { useTranslation } from "react-i18next";
import { AppLayout } from "../components/AppLayout";
import { CreateProjectTile } from "../components/CreateProjectTile";
import { PageHeader } from "../components/PageHeader";
import { ProjectCard } from "../components/ProjectCard";
import { StateMessage } from "../components/StateMessage";
import { useLibrary } from "../hooks/useLibrary";
import { useDocumentHead } from "../hooks/useDocumentHead";
import type { ListProjects200ItemsItem } from "../api/generated/backend.schemas";

export default function LibraryPage() {
  const { t } = useTranslation();
  useDocumentHead({ title: "Library — kioku", robots: "noindex,nofollow" });
  const { items, error, isLoading, hasMore, loadingMore, loadMore, mutate } =
    useLibrary();

  const projects = items;

  return (
    <AppLayout>
      <PageHeader
        title={t("library.title")}
        description={t("library.subtitle")}
      />

        {error && (
          <StateMessage tone="danger" className="mb-4">
            {t("library.error")}
          </StateMessage>
        )}

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          <CreateProjectTile labelKey="library.newProject" />

          {isLoading && projects.length === 0 && (
            <StateMessage className="col-span-full">
              {t("library.loading")}
            </StateMessage>
          )}

          {!isLoading && !error && projects.length === 0 && (
            <StateMessage className="col-span-full">
              {t("library.empty")}
            </StateMessage>
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
              class="btn-secondary"
            >
              {loadingMore ? t("library.loading") : t("library.loadMore")}
            </button>
          </div>
        )}
    </AppLayout>
  );
}
