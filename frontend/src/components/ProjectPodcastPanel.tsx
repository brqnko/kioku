import { HTTPError } from "ky";
import { useEffect, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSWRConfig } from "swr";
import { DeleteItemDialog } from "./DeleteItemDialog";
import { Dialog } from "./Dialog";
import { EditPodcastDialog } from "./EditPodcastDialog";
import { PodcastPlayer } from "./PodcastPlayer";
import { RowActionMenu } from "./RowActionMenu";
import { PodcastCreator } from "../pages/PodcastNewPage";
import { podcastKey } from "../api/keys";
import {
  ListPodcasts200ItemsItemStatus,
  type ListPodcasts200ItemsItem,
} from "../api/generated/backend.schemas";
import { usePodcast, usePodcasts } from "../hooks/usePodcasts";

interface ProjectPodcastPanelProps {
  projectId: string;
  compact?: boolean;
}

type PanelMode = "player" | "create";

function formatPodcastDate(iso: string, locale: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleDateString(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
}

export function ProjectPodcastPanel({
  projectId,
  compact = false,
}: ProjectPodcastPanelProps) {
  const { t, i18n } = useTranslation();
  const { mutate } = useSWRConfig();
  const {
    items,
    error: listError,
    isLoading: listLoading,
    hasMore,
    loadingMore,
    loadMore,
    refresh,
  } = usePodcasts(projectId);

  const [activePodcastId, setActivePodcastId] = useState<string | null>(null);
  const [mode, setMode] = useState<PanelMode>("player");
  const [historyOpen, setHistoryOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<ListPodcasts200ItemsItem | null>(
    null,
  );
  const [deleteTarget, setDeleteTarget] =
    useState<ListPodcasts200ItemsItem | null>(null);

  const activeSummary = items.find((podcast) => podcast.id === activePodcastId);
  const activeIsGenerating =
    activeSummary?.status === ListPodcasts200ItemsItemStatus.generating;
  const {
    data: activePodcast,
    error: podcastError,
    isLoading: podcastLoading,
  } = usePodcast(projectId, activePodcastId ?? undefined);

  const isGenerating =
    activeIsGenerating ||
    (podcastError instanceof HTTPError && podcastError.response?.status === 404);
  const showPlayerLoading =
    Boolean(activePodcastId) &&
    !activePodcast &&
    !podcastError &&
    !isGenerating &&
    podcastLoading;
  const showPlayerError = podcastError && !isGenerating;
  const activeTitle =
    mode === "create"
      ? t("podcast.panel.createTitle")
      : activeSummary?.name ??
        activePodcast?.name ??
        t("project.sections.podcasts.title");

  useEffect(() => {
    if (!activePodcastId && !listLoading && items.length > 0) {
      setActivePodcastId(items[0].id);
    }
  }, [activePodcastId, listLoading, items]);

  const openCreate = () => {
    setMode("create");
    setHistoryOpen(false);
  };

  const selectPodcast = (podcastId: string) => {
    setActivePodcastId(podcastId);
    setMode("player");
    setHistoryOpen(false);
  };

  const handleCreated = async (podcastId?: string) => {
    if (podcastId) {
      setActivePodcastId(podcastId);
    }
    setMode("player");
    await refresh();
  };

  const refreshAfterEdit = async () => {
    await Promise.all([
      refresh(),
      activePodcastId
        ? mutate(podcastKey(projectId, activePodcastId))
        : Promise.resolve(),
    ]);
  };

  const refreshAfterDelete = async () => {
    const deletedId = deleteTarget?.id;
    if (deletedId && deletedId === activePodcastId) {
      const next = items.find((podcast) => podcast.id !== deletedId);
      setActivePodcastId(next?.id ?? null);
    }
    await refresh();
  };

  const renderHistory = () => (
    <div class="flex min-h-0 flex-1 flex-col">
      <div class="shrink-0 p-3 border-b border-border-subtle">
        <button
          type="button"
          onClick={openCreate}
          class="btn-secondary w-full justify-center"
        >
          <span class="material-symbols-outlined text-[18px]">add</span>
          {t("podcast.panel.newPodcast")}
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-2">
        {listLoading && items.length === 0 && (
          <p class="text-xs text-text-disabled text-center px-2 py-4">
            {t("podcast.loading")}
          </p>
        )}
        {!listLoading && items.length === 0 && (
          <p class="text-xs text-text-disabled text-center px-2 py-4">
            {t("podcast.panel.history.empty")}
          </p>
        )}

        <div class="flex flex-col gap-1">
          {items.map((podcast) => {
            const active = podcast.id === activePodcastId;
            const generating =
              podcast.status === ListPodcasts200ItemsItemStatus.generating;
            return (
              <div
                key={podcast.id}
                class={`group flex items-center rounded-lg ${
                  active ? "bg-overlay-soft" : "hover:bg-overlay-faint"
                }`}
              >
                <button
                  type="button"
                  onClick={() => selectPodcast(podcast.id)}
                  class={`flex-1 min-w-0 text-left pl-3 pr-1 py-2.5 text-sm cursor-pointer bg-transparent border-none ${
                    active
                      ? "text-text-primary font-medium"
                      : "text-text-secondary group-hover:text-text-primary"
                  }`}
                >
                  <div class="flex items-center gap-2 min-w-0">
                    <p class="truncate leading-snug">{podcast.name}</p>
                    {generating && (
                      <span class="shrink-0 rounded-full bg-warning/15 px-2 py-0.5 text-[10px] font-medium uppercase tracking-widest text-warning">
                        {t("podcast.list.status.generating")}
                      </span>
                    )}
                  </div>
                  <p class="text-xs text-text-disabled mt-0.5">
                    {formatPodcastDate(podcast.created_at, i18n.language)}
                  </p>
                </button>
                <div class="shrink-0 pr-1 opacity-100 tablet:opacity-0 tablet:group-hover:opacity-100">
                  <RowActionMenu
                    icon="more_vert"
                    ariaLabel={t("podcast.list.menuLabel", {
                      name: podcast.name,
                    })}
                    onEdit={() => {
                      setHistoryOpen(false);
                      setEditTarget(podcast);
                    }}
                    onDelete={() => {
                      setHistoryOpen(false);
                      setDeleteTarget(podcast);
                    }}
                  />
                </div>
              </div>
            );
          })}
        </div>

        {hasMore && (
          <button
            type="button"
            onClick={loadMore}
            disabled={loadingMore}
            class="w-full mt-2 py-2 text-xs text-text-secondary hover:text-text-primary disabled:opacity-50 cursor-pointer bg-transparent border-none"
          >
            {loadingMore ? t("podcast.loading") : t("podcast.loadMore")}
          </button>
        )}
      </div>
    </div>
  );

  return (
    <div
      class={
        compact
          ? "flex h-full min-h-[34rem] flex-col overflow-hidden rounded-lg border border-border-subtle bg-background-dark xl:min-h-0"
          : "flex flex-1 min-h-0 flex-col gap-4"
      }
    >
      <section
        class={`flex flex-1 min-h-0 flex-col bg-background-dark overflow-hidden ${
          compact ? "" : "rounded-lg border border-border-subtle"
        }`}
      >
        <div class="shrink-0 border-b border-border-subtle px-3 tablet:px-5 py-3 flex items-center gap-3">
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-medium text-text-primary">
              {activeTitle}
            </p>
          </div>
          <div class="flex shrink-0 items-center gap-1.5">
            {mode === "create" && activePodcastId && (
              <button
                type="button"
                onClick={() => setMode("player")}
                class="h-8 rounded-md border border-border-subtle bg-transparent px-2.5 text-xs font-medium text-text-secondary hover:bg-overlay-faint hover:text-text-primary flex items-center gap-1.5"
              >
                <span class="material-symbols-outlined text-[17px]">
                  play_circle
                </span>
                <span class="hidden tablet:inline">
                  {t("podcast.panel.backToPlayer")}
                </span>
              </button>
            )}
            <button
              type="button"
              onClick={() => setHistoryOpen(true)}
              class="h-8 rounded-md border border-border-subtle bg-transparent px-2.5 text-xs font-medium text-text-secondary hover:bg-overlay-faint hover:text-text-primary flex items-center gap-1.5"
              aria-label={t("podcast.panel.history.open")}
            >
              <span class="material-symbols-outlined text-[17px]">history</span>
              <span class="hidden tablet:inline">
                {t("podcast.panel.history.open")}
              </span>
            </button>
            <button
              type="button"
              onClick={openCreate}
              class="h-8 rounded-md border border-border-subtle bg-transparent px-2.5 text-xs font-medium text-text-secondary hover:bg-overlay-faint hover:text-text-primary flex items-center gap-1.5"
            >
              <span class="material-symbols-outlined text-[17px]">add</span>
              <span class="hidden tablet:inline">
                {t("podcast.panel.newPodcast")}
              </span>
            </button>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto p-3 tablet:p-4">
          {mode === "create" ? (
            <PodcastCreator
              key="podcast-creator"
              projectId={projectId}
              compact
              onCreated={handleCreated}
            />
          ) : listError ? (
            <p class="text-sm text-danger text-center py-8">
              {t("podcast.errors.load")}
            </p>
          ) : listLoading && items.length === 0 ? (
            <p class="text-sm text-text-disabled text-center py-8">
              {t("podcast.loading")}
            </p>
          ) : !activePodcastId && items.length === 0 ? (
            <div class="flex min-h-[22rem] items-center justify-center py-10">
              <div class="max-w-sm text-center">
                <h2 class="text-base font-medium text-text-primary">
                  {t("podcast.panel.empty.title")}
                </h2>
                <p class="mt-2 text-sm text-text-secondary">
                  {t("podcast.panel.empty.body")}
                </p>
                <button
                  type="button"
                  onClick={openCreate}
                  class="btn-secondary mt-5"
                >
                  <span class="material-symbols-outlined text-[18px]">add</span>
                  {t("podcast.panel.newPodcast")}
                </button>
              </div>
            </div>
          ) : showPlayerLoading ? (
            <p class="text-sm text-text-disabled text-center py-8">
              {t("podcast.loading")}
            </p>
          ) : isGenerating ? (
            <div class="flex min-h-[22rem] flex-col items-center justify-center gap-3 py-10 text-center">
              <span
                class="material-symbols-outlined text-warning text-[30px]"
                style={{ fontVariationSettings: "'FILL' 1" }}
              >
                hourglass_top
              </span>
              <h2 class="text-base font-medium text-text-primary">
                {activeSummary?.name ?? t("project.sections.podcasts.title")}
              </h2>
              <p class="max-w-sm text-sm text-text-secondary">
                {t("podcast.detail.generating")}
              </p>
            </div>
          ) : showPlayerError ? (
            <p class="text-sm text-danger text-center py-8">
              {t("podcast.errors.load")}
            </p>
          ) : activePodcast ? (
            <div class="flex flex-col gap-5">
              <div class="text-center">
                <h2 class="text-xl font-bold text-text-primary leading-tight">
                  {activePodcast.name}
                </h2>
                {activePodcast.description && (
                  <p class="mt-2 text-sm text-text-secondary">
                    {activePodcast.description}
                  </p>
                )}
              </div>
              <PodcastPlayer podcast={activePodcast} compact />
            </div>
          ) : null}
        </div>
      </section>

      <Dialog
        open={historyOpen}
        onClose={() => setHistoryOpen(false)}
        ariaLabel={t("podcast.panel.history.title")}
        maxWidth="max-w-[420px]"
      >
        <div class="flex h-[min(80vh,520px)] min-h-0 flex-col">
          <div class="flex items-center justify-between border-b border-border-subtle p-4">
            <h2 class="text-base font-bold text-text-primary">
              {t("podcast.panel.history.title")}
            </h2>
            <button
              type="button"
              onClick={() => setHistoryOpen(false)}
              class="icon-button"
              aria-label={t("podcast.panel.history.close")}
            >
              <span class="material-symbols-outlined text-[20px]">close</span>
            </button>
          </div>
          {renderHistory()}
        </div>
      </Dialog>

      {editTarget && (
        <EditPodcastDialog
          open={editTarget !== null}
          onClose={() => setEditTarget(null)}
          projectId={projectId}
          podcastId={editTarget.id}
          initialName={editTarget.name}
          initialDescription={editTarget.description}
          onSuccess={refreshAfterEdit}
        />
      )}

      {deleteTarget && (
        <DeleteItemDialog
          open={deleteTarget !== null}
          onClose={() => setDeleteTarget(null)}
          id={deleteTarget.id}
          name={deleteTarget.name}
          customPath={`projects/${projectId}/podcasts/${deleteTarget.id}`}
          onSuccess={refreshAfterDelete}
        />
      )}
    </div>
  );
}
