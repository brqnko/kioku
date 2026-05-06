import { useEffect, useRef, useState } from "preact/hooks";
import { useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import { HTTPError } from "ky";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { useProject } from "../hooks/useProject";
import { usePodcast } from "../hooks/usePodcasts";

const PLAYBACK_RATES = [1, 1.25, 1.5, 1.75, 2] as const;

function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return "0:00";
  const total = Math.floor(seconds);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

export default function PodcastDetailPage() {
  const { t } = useTranslation();
  const route = useRoute();
  const projectId = route.params.projectId;
  const podcastId = route.params.podcastId;

  const { data: project } = useProject(projectId);
  const { data: podcast, error, isLoading } = usePodcast(projectId, podcastId);

  const audioRef = useRef<HTMLAudioElement>(null);
  const progressRef = useRef<HTMLDivElement>(null);

  const [playing, setPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [muted, setMuted] = useState(false);
  const [rateIndex, setRateIndex] = useState(0);
  const [audioError, setAudioError] = useState(false);

  const playbackRate = PLAYBACK_RATES[rateIndex];

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.playbackRate = playbackRate;
  }, [playbackRate]);

  useEffect(() => {
    setPlaying(false);
    setCurrentTime(0);
    setDuration(0);
    setAudioError(false);
  }, [podcastId]);

  const togglePlay = () => {
    const audio = audioRef.current;
    if (!audio) return;
    if (audio.paused) {
      audio.play().catch(() => setAudioError(true));
    } else {
      audio.pause();
    }
  };

  const skip = (delta: number) => {
    const audio = audioRef.current;
    if (!audio || !Number.isFinite(audio.duration)) return;
    audio.currentTime = Math.max(
      0,
      Math.min(audio.duration, audio.currentTime + delta),
    );
  };

  const cycleRate = () => {
    setRateIndex((i) => (i + 1) % PLAYBACK_RATES.length);
  };

  const toggleMute = () => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.muted = !audio.muted;
    setMuted(audio.muted);
  };

  const seekFromEvent = (e: MouseEvent) => {
    const audio = audioRef.current;
    const bar = progressRef.current;
    if (!audio || !bar || !Number.isFinite(audio.duration)) return;
    const rect = bar.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    audio.currentTime = ratio * audio.duration;
    setCurrentTime(audio.currentTime);
  };

  const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

  const isGenerating =
    error instanceof HTTPError && error.response?.status === 404;
  const showLoading = isLoading && !podcast && !error;
  const showError = error && !isGenerating;

  const podcastsHref = `/projects/${projectId}/podcasts`;

  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-[var(--sidebar-width)] p-8 min-h-[calc(100vh-3.5rem)] overflow-y-auto transition-[margin-left] duration-200 ease-in-out">
        <div class="max-w-[800px] mx-auto flex flex-col gap-8">
          <nav class="flex items-center gap-2 text-text-secondary text-sm font-medium flex-wrap">
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
              {project?.name ?? "..."}
            </a>
            <span class="material-symbols-outlined text-[16px]">
              chevron_right
            </span>
            <a
              href={podcastsHref}
              class="hover:text-text-primary no-underline text-inherit"
            >
              {t("podcast.list.crumb")}
            </a>
            <span class="material-symbols-outlined text-[16px]">
              chevron_right
            </span>
            <span class="text-text-primary">
              {t("podcast.detail.crumb")}
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
                <h1 class="text-[40px] sm:text-[54px] leading-[1.04] font-bold tracking-tight max-w-[600px]">
                  {podcast.name}
                </h1>
                {podcast.description && (
                  <p class="text-base text-text-secondary max-w-[500px]">
                    {podcast.description}
                  </p>
                )}
              </header>

              <section class="bg-surface-dark border border-border-subtle rounded-xl p-6 flex flex-col gap-6 shadow-[0_4px_24px_rgba(0,0,0,0.2)]">
                <audio
                  ref={audioRef}
                  src={podcast.audio_url}
                  preload="metadata"
                  onLoadedMetadata={(e) => {
                    setDuration((e.currentTarget as HTMLAudioElement).duration);
                  }}
                  onTimeUpdate={(e) => {
                    setCurrentTime(
                      (e.currentTarget as HTMLAudioElement).currentTime,
                    );
                  }}
                  onPlay={() => setPlaying(true)}
                  onPause={() => setPlaying(false)}
                  onEnded={() => setPlaying(false)}
                  onError={() => setAudioError(true)}
                />

                <div class="flex flex-col gap-2">
                  <div class="flex justify-between text-sm text-text-secondary">
                    <span>{formatTime(currentTime)}</span>
                    <span>
                      {duration > 0 ? formatTime(duration) : "--:--"}
                    </span>
                  </div>
                  <div
                    ref={progressRef}
                    role="slider"
                    aria-label={t("podcast.detail.progress")}
                    aria-valuemin={0}
                    aria-valuemax={duration || 0}
                    aria-valuenow={currentTime}
                    tabIndex={0}
                    onClick={seekFromEvent}
                    class="h-2 w-full bg-surface-container-high rounded-full overflow-hidden relative cursor-pointer group"
                  >
                    <div
                      class="absolute left-0 top-0 h-full bg-text-primary group-hover:bg-accent-blue transition-colors rounded-full"
                      style={{ width: `${progress}%` }}
                    />
                  </div>
                </div>

                <div class="flex items-center justify-between">
                  <button
                    type="button"
                    onClick={toggleMute}
                    aria-label={
                      muted
                        ? t("podcast.detail.unmute")
                        : t("podcast.detail.mute")
                    }
                    title={
                      muted
                        ? t("podcast.detail.unmute")
                        : t("podcast.detail.mute")
                    }
                    class="p-2 rounded hover:bg-overlay-faint text-text-secondary hover:text-text-primary transition-colors flex items-center justify-center cursor-pointer"
                  >
                    <span class="material-symbols-outlined">
                      {muted ? "volume_off" : "volume_up"}
                    </span>
                  </button>

                  <div class="flex items-center gap-4">
                    <button
                      type="button"
                      onClick={() => skip(-10)}
                      aria-label={t("podcast.detail.replay10")}
                      title={t("podcast.detail.replay10")}
                      class="p-2 rounded hover:bg-overlay-faint text-text-secondary hover:text-text-primary transition-colors flex items-center justify-center cursor-pointer"
                    >
                      <span class="material-symbols-outlined text-[28px]">
                        replay_10
                      </span>
                    </button>
                    <button
                      type="button"
                      onClick={togglePlay}
                      disabled={audioError || duration === 0}
                      aria-label={
                        playing
                          ? t("podcast.detail.pause")
                          : t("podcast.detail.play")
                      }
                      class="w-16 h-16 rounded-full bg-cta text-cta-fg flex items-center justify-center hover:bg-cta-hover transition-colors shadow-lg active:scale-95 cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed disabled:active:scale-100"
                    >
                      <span
                        class="material-symbols-outlined text-[32px]"
                        style={{ fontVariationSettings: "'FILL' 1" }}
                      >
                        {playing ? "pause" : "play_arrow"}
                      </span>
                    </button>
                    <button
                      type="button"
                      onClick={() => skip(10)}
                      aria-label={t("podcast.detail.forward10")}
                      title={t("podcast.detail.forward10")}
                      class="p-2 rounded hover:bg-overlay-faint text-text-secondary hover:text-text-primary transition-colors flex items-center justify-center cursor-pointer"
                    >
                      <span class="material-symbols-outlined text-[28px]">
                        forward_10
                      </span>
                    </button>
                  </div>

                  <button
                    type="button"
                    onClick={cycleRate}
                    aria-label={t("podcast.detail.speed")}
                    title={t("podcast.detail.speed")}
                    class="px-3 py-1.5 rounded border border-border-subtle hover:bg-overlay-faint text-text-secondary hover:text-text-primary transition-colors text-xs font-medium cursor-pointer min-w-[3rem]"
                  >
                    {playbackRate}x
                  </button>
                </div>

                {audioError && (
                  <p class="text-sm text-danger text-center">
                    {t("podcast.detail.audioError")}
                  </p>
                )}
              </section>

              {podcast.podcast_script.length > 0 && (
                <section class="flex flex-col gap-4">
                  <div class="flex items-center gap-2 text-text-secondary border-b border-border-subtle pb-2">
                    <span class="material-symbols-outlined">subject</span>
                    <h2 class="text-[18px] font-bold text-text-primary">
                      {t("podcast.detail.transcript")}
                    </h2>
                  </div>
                  <ol class="flex flex-col gap-4 list-none p-0 m-0">
                    {podcast.podcast_script.map((entry, idx) => (
                      <li
                        key={`${idx}-${entry.speaker}`}
                        class="flex flex-col sm:flex-row gap-2 sm:gap-4"
                      >
                        <span class="shrink-0 sm:w-32 text-xs font-bold uppercase tracking-widest text-accent-blue pt-1">
                          {entry.speaker}
                        </span>
                        <p class="text-base text-text-primary leading-[1.6] flex-1 whitespace-pre-wrap">
                          {entry.text}
                        </p>
                      </li>
                    ))}
                  </ol>
                </section>
              )}
            </>
          )}
        </div>
      </main>
    </div>
  );
}
