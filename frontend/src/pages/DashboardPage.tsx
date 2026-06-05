import { useState } from "preact/hooks";
import { useLocation } from "preact-iso";
import { useTranslation } from "react-i18next";
import { AppLayout } from "../components/AppLayout";
import { CreateProjectTile } from "../components/CreateProjectTile";
import { MarkdownView } from "../components/MarkdownView";
import { PageHeader } from "../components/PageHeader";
import { ProjectCard } from "../components/ProjectCard";
import { StateMessage } from "../components/StateMessage";
import { useDashboard } from "../hooks/useDashboard";
import { useDocumentHead } from "../hooks/useDocumentHead";
import { fetchFileAncestors } from "../hooks/useFolder";
import { useLibrary } from "../hooks/useLibrary";
import { formatRelative } from "../utils/datetime";
import type {
  GetDashboard200RecentSeenFilesItem,
  ListProjects200ItemsItem,
} from "../api/generated/backend.schemas";

function fileIcon(name: string): { icon: string; tone: "danger" | "info" } {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  if (ext === "pdf") return { icon: "picture_as_pdf", tone: "danger" };
  return { icon: "description", tone: "info" };
}

const toneClass = {
  danger: "bg-danger/10 text-danger",
  info: "bg-accent-blue/10 text-accent-blue",
} as const;

export default function DashboardPage() {
  const { t, i18n } = useTranslation();
  const { route: navigate } = useLocation();
  useDocumentHead({ title: "Workspace — kioku", robots: "noindex,nofollow" });
  const { data, error, isLoading } = useDashboard();
  const [openingFileId, setOpeningFileId] = useState<string | null>(null);
  const [recentOpenError, setRecentOpenError] = useState(false);
  const {
    items: projects,
    error: libraryError,
    isLoading: libraryLoading,
    hasMore,
    loadingMore,
    loadMore,
    mutate,
  } = useLibrary();

  const openRecentFile = async (file: GetDashboard200RecentSeenFilesItem) => {
    setOpeningFileId(file.id);
    setRecentOpenError(false);
    try {
      const ancestors = await fetchFileAncestors(file.id);
      const project = ancestors.find((ancestor) => ancestor.kind === "project");
      if (!project) throw new Error("project ancestor not found");
      const folderIds = ancestors
        .filter((ancestor) => ancestor.kind === "folder")
        .map((ancestor) => ancestor.id);
      const search = new URLSearchParams({ file: file.id });
      if (folderIds.length > 0) search.set("folders", folderIds.join(","));
      navigate(`/projects/${project.id}?${search.toString()}`);
    } catch {
      setRecentOpenError(true);
    } finally {
      setOpeningFileId(null);
    }
  };

  return (
    <AppLayout className="flex flex-col gap-8">
      <PageHeader
        title={t("workspace.title")}
      />

      <div class="grid grid-cols-12 gap-4 auto-rows-min">
        <section class="col-span-12 bg-surface-dark rounded-[12px] border border-border-subtle p-6">
          <div class="flex items-start justify-between gap-4 mb-4">
            <div class="flex items-center gap-3 min-w-0">
              <div class="w-9 h-9 rounded-lg bg-overlay-faint border border-border-subtle flex items-center justify-center shrink-0">
                <span class="material-symbols-outlined text-text-primary text-[20px]">
                  auto_awesome
                </span>
              </div>
              <h2 class="heading-h2 truncate">{t("dashboard.summary.title")}</h2>
            </div>
            {data?.ai_learning_summary_updated_at && (
              <span class="text-xs text-text-disabled shrink-0">
                {formatRelative(
                  data.ai_learning_summary_updated_at,
                  i18n.language,
                )}
              </span>
            )}
          </div>

          {isLoading && <StateMessage>{t("dashboard.loading")}</StateMessage>}
          {error && (
            <StateMessage tone="danger">{t("dashboard.error")}</StateMessage>
          )}
          {data && !data.ai_learning_summary && (
            <StateMessage>{t("dashboard.summary.empty")}</StateMessage>
          )}
          {data?.ai_learning_summary && (
            <MarkdownView
              source={data.ai_learning_summary}
              className="markdown-body text-sm text-text-muted-dark leading-relaxed"
            />
          )}
        </section>

        <section class="col-span-12 mt-2">
          <div class="section-heading">
            <h2>
              <span class="material-symbols-outlined text-text-secondary text-[18px]">
                history
              </span>
              {t("dashboard.recent.title")}
            </h2>
          </div>

          {isLoading && <StateMessage>{t("dashboard.loading")}</StateMessage>}
          {error && (
            <StateMessage tone="danger">{t("dashboard.error")}</StateMessage>
          )}
          {recentOpenError && (
            <StateMessage tone="danger" className="mb-4">
              {t("dashboard.error")}
            </StateMessage>
          )}
          {data?.recent_seen_files?.length === 0 && (
            <StateMessage>{t("dashboard.recent.empty")}</StateMessage>
          )}
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
            {data?.recent_seen_files?.map(
              (file: GetDashboard200RecentSeenFilesItem) => {
                const { icon, tone } = fileIcon(file.name);
                const isOpening = openingFileId === file.id;
                return (
                  <button
                    type="button"
                    key={file.id}
                    onClick={() => void openRecentFile(file)}
                    disabled={openingFileId !== null}
                    class="bg-surface-dark border border-border-subtle rounded-[8px] p-4 flex items-center gap-3 group text-inherit min-w-0 text-left hover:bg-overlay-faint hover:border-overlay-medium disabled:cursor-wait disabled:opacity-70"
                  >
                    <div
                      class={`w-10 h-10 rounded-lg flex items-center justify-center shrink-0 ${toneClass[tone]}`}
                    >
                      {isOpening ? (
                        <span class="material-symbols-outlined text-[20px] animate-spin">
                          progress_activity
                        </span>
                      ) : (
                        <span class="material-symbols-outlined text-[20px]">
                          {icon}
                        </span>
                      )}
                    </div>
                    <div class="min-w-0">
                      <h3 class="text-sm font-bold truncate group-hover:text-accent-blue">
                        {file.name}
                      </h3>
                      <p class="text-xs text-text-disabled">
                        {formatRelative(file.changed_at, i18n.language)}
                      </p>
                    </div>
                  </button>
                );
              },
            )}
          </div>
        </section>

        <section class="col-span-12 mt-2">
          <div class="section-heading">
            <h2>
              <span class="material-symbols-outlined text-text-secondary text-[18px]">
                workspaces
              </span>
              {t("workspace.projects.title")}
            </h2>
          </div>

          {libraryError && (
            <StateMessage tone="danger" className="mb-4">
              {t("library.error")}
            </StateMessage>
          )}

          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            <CreateProjectTile labelKey="library.newProject" />

            {libraryLoading && projects.length === 0 && (
              <StateMessage className="col-span-full">
                {t("library.loading")}
              </StateMessage>
            )}

            {!libraryLoading && !libraryError && projects.length === 0 && (
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
        </section>
      </div>
    </AppLayout>
  );
}
