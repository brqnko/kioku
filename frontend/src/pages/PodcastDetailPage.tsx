import { useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import { HTTPError } from "ky";
import { AppLayout } from "../components/AppLayout";
import { PodcastPlayer } from "../components/PodcastPlayer";
import { useProject } from "../hooks/useProject";
import { usePodcast } from "../hooks/usePodcasts";
import { useDocumentHead } from "../hooks/useDocumentHead";

export default function PodcastDetailPage() {
  const { t } = useTranslation();
  useDocumentHead({ title: "Podcast — kioku", robots: "noindex,nofollow" });
  const route = useRoute();
  const projectId = route.params.projectId;
  const podcastId = route.params.podcastId;

  const { data: project } = useProject(projectId);
  const { data: podcast, error, isLoading } = usePodcast(projectId, podcastId);

  const isGenerating =
    error instanceof HTTPError && error.response?.status === 404;
  const showLoading = isLoading && !podcast && !error;
  const showError = error && !isGenerating;
  const podcastsHref = `/projects/${projectId}/podcasts`;

  return (
    <AppLayout>
      <div class="max-w-[800px] mx-auto flex flex-col gap-8">
        <nav class="flex items-center gap-1.5 text-text-secondary text-sm font-medium flex-wrap">
          <a
            href="/dashboard"
            class="hover:text-text-primary no-underline text-inherit"
          >
            {t("workspace.title")}
          </a>
          <span class="material-symbols-outlined text-[16px] select-none">
            chevron_right
          </span>
          <a
            href={podcastsHref}
            class="hover:text-text-primary no-underline text-inherit truncate max-w-[160px]"
          >
            {project?.name ?? "..."}
          </a>
          <span class="material-symbols-outlined text-[16px] select-none">
            chevron_right
          </span>
          <span class="text-text-primary truncate max-w-[200px]">
            {podcast?.name ?? t("podcast.detail.crumb")}
          </span>
        </nav>

        {showLoading && (
          <p class="text-sm text-text-secondary text-center py-16">
            {t("podcast.loading")}
          </p>
        )}

        {isGenerating && (
          <div class="flex flex-col items-center gap-4 py-16 text-center">
            <span
              class="material-symbols-outlined text-warning text-[32px]"
              style={{ fontVariationSettings: "'FILL' 1" }}
            >
              hourglass_top
            </span>
            <p class="text-sm text-text-secondary max-w-md">
              {t("podcast.detail.generating")}
            </p>
            <a
              href={podcastsHref}
              class="text-sm text-accent-blue hover:underline no-underline"
            >
              {t("podcast.detail.backToList")}
            </a>
          </div>
        )}

        {showError && (
          <p class="text-sm text-danger text-center py-16">
            {t("podcast.errors.load")}
          </p>
        )}

        {podcast && (
          <>
            <header class="flex flex-col gap-4 items-center text-center">
              <h1 class="heading-h1 max-w-[600px]">{podcast.name}</h1>
              {podcast.description && (
                <p class="text-body text-text-secondary max-w-[500px]">
                  {podcast.description}
                </p>
              )}
            </header>

            <PodcastPlayer podcast={podcast} />
          </>
        )}
      </div>
    </AppLayout>
  );
}
