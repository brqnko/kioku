import { useEffect, useRef, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import type { GetPodcast200 } from "../api/generated/backend.schemas";

const PLAYBACK_RATES = [1, 1.25, 1.5, 1.75, 2] as const;

function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return "0:00";
  const total = Math.floor(seconds);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}

interface PodcastPlayerProps {
  podcast: GetPodcast200;
  showTranscript?: boolean;
  compact?: boolean;
}

export function PodcastPlayer({
  podcast,
  showTranscript = true,
  compact = false,
}: PodcastPlayerProps) {
  const { t } = useTranslation();
  const audioRef = useRef<HTMLAudioElement>(null);
  const progressRef = useRef<HTMLDivElement>(null);

  const [playing, setPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [muted, setMuted] = useState(false);
  const [volume, setVolume] = useState(1);
  const [rateIndex, setRateIndex] = useState(0);
  const [audioError, setAudioError] = useState(false);

  const playbackRate = PLAYBACK_RATES[rateIndex];
  const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

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
  }, [podcast.id, podcast.audio_url]);

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

  const changeVolume = (v: number) => {
    const audio = audioRef.current;
    if (!audio) return;
    audio.volume = v;
    setVolume(v);
    if (v === 0) {
      audio.muted = true;
      setMuted(true);
    } else if (audio.muted) {
      audio.muted = false;
      setMuted(false);
    }
  };

  const seekFromEvent = (e: MouseEvent) => {
    const audio = audioRef.current;
    const bar = progressRef.current;
    if (!audio || !bar || !Number.isFinite(audio.duration)) return;
    const rect = bar.getBoundingClientRect();
    const ratio = Math.max(
      0,
      Math.min(1, (e.clientX - rect.left) / rect.width),
    );
    audio.currentTime = ratio * audio.duration;
    setCurrentTime(audio.currentTime);
  };

  const seekBy = (delta: number) => {
    const audio = audioRef.current;
    if (!audio || !Number.isFinite(audio.duration)) return;
    audio.currentTime = Math.max(
      0,
      Math.min(audio.duration, audio.currentTime + delta),
    );
    setCurrentTime(audio.currentTime);
  };

  const seekTo = (ratio: number) => {
    const audio = audioRef.current;
    if (!audio || !Number.isFinite(audio.duration)) return;
    audio.currentTime = Math.max(0, Math.min(1, ratio)) * audio.duration;
    setCurrentTime(audio.currentTime);
  };

  const handleProgressKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case "ArrowLeft":
        e.preventDefault();
        seekBy(-5);
        break;
      case "ArrowRight":
        e.preventDefault();
        seekBy(5);
        break;
      case "Home":
        e.preventDefault();
        seekTo(0);
        break;
      case "End":
        e.preventDefault();
        seekTo(1);
        break;
      case "PageDown":
        e.preventDefault();
        seekBy(-30);
        break;
      case "PageUp":
        e.preventDefault();
        seekBy(30);
        break;
    }
  };

  return (
    <div class="flex flex-col gap-5">
      <section
        class={`bg-surface-dark border border-border-subtle rounded-[12px] flex flex-col gap-5 shadow-[0_4px_12px_rgba(0,0,0,0.15)] ${
          compact ? "p-4" : "p-6"
        }`}
      >
        <audio
          ref={audioRef}
          src={podcast.audio_url}
          preload="metadata"
          onLoadedMetadata={(e) => {
            setDuration((e.currentTarget as HTMLAudioElement).duration);
          }}
          onTimeUpdate={(e) => {
            setCurrentTime((e.currentTarget as HTMLAudioElement).currentTime);
          }}
          onPlay={() => setPlaying(true)}
          onPause={() => setPlaying(false)}
          onEnded={() => setPlaying(false)}
          onError={() => setAudioError(true)}
        />

        <div class="flex flex-col gap-2">
          <div class="flex justify-between text-sm text-text-secondary">
            <span>{formatTime(currentTime)}</span>
            <span>{duration > 0 ? formatTime(duration) : "--:--"}</span>
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
            onKeyDown={handleProgressKeyDown}
            class="h-2 w-full bg-surface-container-high rounded-full overflow-hidden relative cursor-pointer group focus:outline-none focus-visible:ring-2 focus-visible:ring-accent-blue/60"
          >
            <div
              class="absolute left-0 top-0 h-full bg-text-primary group-hover:bg-accent-blue rounded-full"
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>

        <div class="flex items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <button
              type="button"
              onClick={toggleMute}
              aria-label={
                muted ? t("podcast.detail.unmute") : t("podcast.detail.mute")
              }
              title={
                muted ? t("podcast.detail.unmute") : t("podcast.detail.mute")
              }
              class="icon-button"
            >
              <span class="material-symbols-outlined">
                {muted || volume === 0
                  ? "volume_off"
                  : volume < 0.5
                    ? "volume_down"
                    : "volume_up"}
              </span>
            </button>
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={muted ? 0 : volume}
              onInput={(e) =>
                changeVolume(
                  parseFloat((e.currentTarget as HTMLInputElement).value),
                )
              }
              aria-label={t("podcast.detail.volume")}
              class={`${compact ? "hidden tablet:block w-14" : "w-20"} h-1 cursor-pointer`}
              style={{ accentColor: "#2383e2" }}
            />
          </div>

          <div class="flex items-center gap-2 tablet:gap-4">
            <button
              type="button"
              onClick={() => skip(-10)}
              aria-label={t("podcast.detail.replay10")}
              title={t("podcast.detail.replay10")}
              class="icon-button"
            >
              <span class="material-symbols-outlined text-[26px]">
                replay_10
              </span>
            </button>
            <button
              type="button"
              onClick={togglePlay}
              disabled={audioError || duration === 0}
              aria-label={
                playing ? t("podcast.detail.pause") : t("podcast.detail.play")
              }
              class={`rounded-full bg-cta text-cta-fg flex items-center justify-center hover:bg-cta-hover shadow-lg cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed ${
                compact ? "h-14 w-14" : "h-16 w-16"
              }`}
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
              class="icon-button"
            >
              <span class="material-symbols-outlined text-[26px]">
                forward_10
              </span>
            </button>
          </div>

          <button
            type="button"
            onClick={cycleRate}
            aria-label={t("podcast.detail.speed")}
            title={t("podcast.detail.speed")}
            class="btn-secondary text-xs px-2.5 min-w-[3rem]"
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

      {showTranscript && podcast.podcast_script.length > 0 && (
        <section class="flex flex-col gap-4">
          <div class="flex items-center gap-2 text-text-secondary border-b border-border-subtle pb-2">
            <span class="material-symbols-outlined">subject</span>
            <h2 class={compact ? "text-sm font-bold" : "heading-h2"}>
              {t("podcast.detail.transcript")}
            </h2>
          </div>
          <ol class="flex flex-col gap-4 list-none p-0 m-0">
            {podcast.podcast_script.map((entry, idx) => (
              <li
                key={`${idx}-${entry.speaker}`}
                class={`flex flex-col gap-1 ${compact ? "" : "sm:flex-row sm:gap-4"}`}
              >
                <span
                  class={`shrink-0 text-xs font-bold uppercase tracking-widest text-accent-blue ${
                    compact ? "" : "sm:w-32 pt-1"
                  }`}
                >
                  {entry.speaker}
                </span>
                <p
                  class={`text-text-primary leading-[1.6] flex-1 whitespace-pre-wrap ${
                    compact ? "text-sm" : "text-base"
                  }`}
                >
                  {entry.text}
                </p>
              </li>
            ))}
          </ol>
        </section>
      )}
    </div>
  );
}
