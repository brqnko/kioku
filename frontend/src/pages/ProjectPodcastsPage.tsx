import { useRoute } from "preact-iso";
import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { RowActionMenu } from "../components/RowActionMenu";
import { EditPodcastDialog } from "../components/EditPodcastDialog";
import { DeleteItemDialog } from "../components/DeleteItemDialog";
import { useProject } from "../hooks/useProject";
import { usePodcasts } from "../hooks/usePodcasts";
import { useDocumentHead } from "../hooks/useDocumentHead";
import type { ListPodcasts200ItemsItem } from "../api/generated/backend.schemas";
import { ListPodcasts200ItemsItemStatus } from "../api/generated/backend.schemas";

function formatPodcastDate(iso: string, locale: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleDateString(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}

export default function ProjectPodcastsPage() {
  const { t, i18n } = useTranslation();
  useDocumentHead({ title: "Project podcasts — kioku", robots: "noindex,nofollow" });
  const route = useRoute();
  const projectId = route.params.projectId;

  const { data: project, error: projectError } = useProject(projectId);
  const {
    items,
    error: listError,
    isLoading,
    hasMore,
    loadingMore,
    loadMore,
    refresh,
  } = usePodcasts(projectId);

  const [editTarget, setEditTarget] = useState<ListPodcasts200ItemsItem | null>(null);
  const [editOpen, setEditOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<ListPodcasts200ItemsItem | null>(null);
  const [deleteOpen, setDeleteOpen] = useState(false);

  const openEdit = (podcast: ListPodcasts200ItemsItem) => {
    setEditTarget(podcast);
    setEditOpen(true);
  };

  const openDelete = (podcast: ListPodcasts200ItemsItem) => {
    setDeleteTarget(podcast);
    setDeleteOpen(true);
  };

  const detailHref = (id: string) =>
    `/projects/${projectId}/podcasts/${id}`;

  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-[var(--sidebar-width)] p-4 tablet:p-8 h-[calc(100vh-3.5rem)] overflow-y-auto transition-[margin-left] duration-200 ease-in-out">
        <div class="max-w-[1200px] mx-auto">
          <nav class="flex items-center gap-2 text-text-secondary text-sm font-medium flex-wrap mb-6">
            <a
              href="/library"
              class="hover:text-text-primary no-underline text-inherit"
            >
              {t("project.breadcrumb.library")}
            </a>
            <span class="material-symbols-outlined text-[16px]">
              chevron_right
            </span>
            <a
              href={`/projects/${projectId}`}
              class="hover:text-text-primary no-underline text-inherit"
            >
              {project?.name ?? (projectError ? "—" : "...")}
            </a>
            <span class="material-symbols-outlined text-[16px]">
              chevron_right
            </span>
            <span class="text-text-primary">{t("podcast.list.crumb")}</span>
          </nav>

          <header class="flex items-end justify-between gap-6 flex-wrap mb-8 tablet:mb-12">
            <div class="flex flex-col gap-2 max-w-2xl">
              <h1 class="heading-h2">{t("podcast.list.title")}</h1>
              <p class="text-body text-text-secondary">
                {t("podcast.list.subtitle")}
              </p>
            </div>
            <a
              href={`/projects/${projectId}/podcasts/new`}
              class="btn-primary no-underline"
            >
              <span class="material-symbols-outlined text-[20px]">
                add_circle
              </span>
              {t("podcast.list.create.cta")}
            </a>
          </header>

          {projectError && (
            <p class="text-sm text-danger mb-4">
              {t("project.errors.load")}
            </p>
          )}
          {listError && (
            <p class="text-sm text-danger mb-4">
              {t("podcast.errors.load")}
            </p>
          )}

          {isLoading && items.length === 0 && (
            <p class="text-sm text-text-secondary">{t("podcast.loading")}</p>
          )}

          {!isLoading && !listError && items.length === 0 && (
            <div class="flex flex-col items-center gap-3 py-16 text-center">
              <div class="w-12 h-12 rounded-full bg-surface-dark flex items-center justify-center">
                <span class="material-symbols-outlined text-text-secondary text-[24px]">
                  podcasts
                </span>
              </div>
              <p class="text-sm text-text-secondary max-w-md">
                {t("podcast.list.empty")}
              </p>
            </div>
          )}

          {items.length > 0 && (
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {items.map((podcast) => (
                <div
                  key={podcast.id}
                  class="relative group rounded-[12px] bg-surface-dark border border-border-subtle hover:border-overlay-medium shadow-[0_1px_3px_rgba(0,0,0,0.1)]"
                >
                  <a
                    href={detailHref(podcast.id)}
                    class="flex flex-col gap-3 p-6 pr-10 no-underline text-inherit cursor-pointer"
                  >
                    <div class="flex items-center gap-2">
                      {podcast.status ===
                        ListPodcasts200ItemsItemStatus.generating && (
                        <span class="text-[10px] uppercase tracking-widest px-2 py-0.5 rounded-full bg-warning/15 text-warning">
                          {t("podcast.list.status.generating")}
                        </span>
                      )}
                    </div>
                    <h3 class="text-base font-medium text-text-primary line-clamp-2">
                      {podcast.name}
                    </h3>
                    <p class="text-sm text-text-secondary line-clamp-2 mb-auto">
                      {podcast.description ||
                        t("podcast.list.noDescription")}
                    </p>
                    <p class="text-xs text-text-disabled">
                      {t("podcast.list.createdOn", {
                        date: formatPodcastDate(
                          podcast.created_at,
                          i18n.language,
                        ),
                      })}
                    </p>
                  </a>
                  <div class="absolute top-2 right-2 z-10">
                    <RowActionMenu
                      ariaLabel={t("podcast.list.menuLabel", { name: podcast.name })}
                      icon="more_vert"
                      onEdit={() => openEdit(podcast)}
                      onDelete={() => openDelete(podcast)}
                    />
                  </div>
                </div>
              ))}
            </div>
          )}

          {hasMore && (
            <div class="mt-16 flex justify-center">
              <button
                type="button"
                onClick={loadMore}
                disabled={loadingMore}
                class="btn-secondary"
              >
                <span class="material-symbols-outlined text-[18px]">
                  expand_more
                </span>
                {loadingMore
                  ? t("podcast.loading")
                  : t("podcast.list.loadMore")}
              </button>
            </div>
          )}
        </div>
      </main>

      {editTarget && (
        <EditPodcastDialog
          open={editOpen}
          onClose={() => setEditOpen(false)}
          projectId={projectId}
          podcastId={editTarget.id}
          initialName={editTarget.name}
          initialDescription={editTarget.description}
          onSuccess={refresh}
        />
      )}

      {deleteTarget && (
        <DeleteItemDialog
          open={deleteOpen}
          onClose={() => setDeleteOpen(false)}
          id={deleteTarget.id}
          name={deleteTarget.name}
          customPath={`projects/${projectId}/podcasts/${deleteTarget.id}`}
          onSuccess={refresh}
        />
      )}
    </div>
  );
}
