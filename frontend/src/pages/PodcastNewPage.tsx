import { useEffect, useRef, useState } from "preact/hooks";
import { useLocation, useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { kyInstance } from "../api/mutator";
import { useProject, useProjectChildren } from "../hooks/useProject";
import { useFolderChildren } from "../hooks/useFolder";
import { useDocumentHead } from "../hooks/useDocumentHead";
import { fileMeta, folderTone } from "../utils/file";
import type {
  CreatePodcast200,
  CreatePodcastBody,
  ListFolderChildren200ItemsItem,
  ListProjectChildren200ItemsItem,
} from "../api/generated/backend.schemas";

type ChildItem =
  | ListProjectChildren200ItemsItem
  | ListFolderChildren200ItemsItem;
type FolderItem = Extract<ChildItem, { kind: "folder" }>;
type FileItem = Extract<ChildItem, { kind: "file" }>;

type VoiceStyle = "female" | "male";

const FEMALE_VOICES: VoiceStyle[] = ["female"];
const MALE_VOICES: VoiceStyle[] = ["male"];
const DEFAULT_VOICE: VoiceStyle = "female";
const DEFAULT_VOICE_2: VoiceStyle = "male";
const VOICE_STORAGE_KEY = "podcast.voiceStyle";
const VOICE_2_STORAGE_KEY = "podcast.voiceStyle2";
const ALL_VOICES: VoiceStyle[] = [...FEMALE_VOICES, ...MALE_VOICES];

type SpeakerCount = 1 | 2;
const SPEAKER_COUNTS: SpeakerCount[] = [1, 2];
const DEFAULT_SPEAKER_COUNT: SpeakerCount = 2;
const SPEAKER_COUNT_STORAGE_KEY = "podcast.speakerCount";

type PodcastLength = "short" | "normal" | "long";
const LENGTHS: PodcastLength[] = ["short", "normal", "long"];
const DEFAULT_LENGTH: PodcastLength = "normal";
const LENGTH_STORAGE_KEY = "podcast.length";

const loadStoredLength = (): PodcastLength => {
  try {
    const saved = localStorage.getItem(LENGTH_STORAGE_KEY);
    if (saved && (LENGTHS as string[]).includes(saved)) {
      return saved as PodcastLength;
    }
  } catch {
    // ignore (SSR / private mode)
  }
  return DEFAULT_LENGTH;
};

const loadStoredVoice = (): VoiceStyle => {
  try {
    const saved = localStorage.getItem(VOICE_STORAGE_KEY);
    if (saved && (ALL_VOICES as string[]).includes(saved)) {
      return saved as VoiceStyle;
    }
  } catch {
    // ignore (SSR / private mode)
  }
  return DEFAULT_VOICE;
};

const loadStoredVoice2 = (): VoiceStyle => {
  try {
    const saved = localStorage.getItem(VOICE_2_STORAGE_KEY);
    if (saved && (ALL_VOICES as string[]).includes(saved)) {
      return saved as VoiceStyle;
    }
  } catch {
    // ignore (SSR / private mode)
  }
  return DEFAULT_VOICE_2;
};

const loadStoredSpeakerCount = (): SpeakerCount => {
  try {
    const saved = localStorage.getItem(SPEAKER_COUNT_STORAGE_KEY);
    const parsed = saved ? Number(saved) : NaN;
    if (parsed === 1 || parsed === 2) {
      return parsed;
    }
  } catch {
    // ignore (SSR / private mode)
  }
  return DEFAULT_SPEAKER_COUNT;
};

interface VoicePickerProps {
  value: VoiceStyle;
  onChange: (v: VoiceStyle) => void;
  playing: VoiceStyle | null;
  onPreview: (v: VoiceStyle) => void;
  titleKey?: string;
  hintKey?: string;
  disabledVoice?: VoiceStyle;
}

function VoicePicker({
  value,
  onChange,
  playing,
  onPreview,
  titleKey,
  hintKey,
  disabledVoice,
}: VoicePickerProps) {
  const { t } = useTranslation();

  const renderGroup = (title: string, items: VoiceStyle[]) => (
    <div class="flex flex-col gap-1.5">
      <span class="text-[10px] font-bold uppercase tracking-widest text-text-secondary">
        {title}
      </span>
      <div class="grid grid-cols-1 tablet:grid-cols-2 gap-1.5">
        {items.map((v) => {
          const selected = value === v;
          const isPlaying = playing === v;
          const isDisabled = disabledVoice === v && !selected;
          return (
            <div
              key={v}
              class={`flex items-center gap-1 rounded-md border pl-2.5 pr-1 transition-colors ${
                selected
                  ? "border-accent-blue bg-overlay-faint"
                  : isDisabled
                    ? "border-border-subtle opacity-40"
                    : "border-border-subtle hover:bg-overlay-faint"
              }`}
            >
              <button
                type="button"
                onClick={() => !isDisabled && onChange(v)}
                disabled={isDisabled}
                aria-pressed={selected}
                class="flex flex-1 min-w-0 items-center gap-2 bg-transparent border-none p-0 py-1.5 text-left cursor-pointer disabled:cursor-not-allowed"
              >
                <span
                  aria-hidden
                  class={`flex h-3.5 w-3.5 shrink-0 items-center justify-center rounded-full border ${
                    selected ? "border-accent-blue" : "border-overlay-medium"
                  }`}
                >
                  {selected && (
                    <span class="h-1.5 w-1.5 rounded-full bg-accent-blue" />
                  )}
                </span>
                <span class="text-[13px] leading-tight text-text-primary truncate">
                  {t(`podcast.create.voice.styles.${v}`)}
                </span>
              </button>
              <button
                type="button"
                onClick={() => onPreview(v)}
                aria-label={
                  isPlaying
                    ? t("podcast.create.voice.stop", { name: v })
                    : t("podcast.create.voice.play", { name: v })
                }
                class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-transparent text-text-secondary hover:text-text-primary hover:bg-overlay-faint cursor-pointer"
              >
                <span
                  class="material-symbols-outlined text-[16px]"
                  style={{ fontVariationSettings: "'FILL' 1" }}
                >
                  {isPlaying ? "stop" : "play_arrow"}
                </span>
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );

  return (
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1">
        <label class="text-xs font-bold uppercase tracking-widest text-text-secondary">
          {t(titleKey ?? "podcast.create.voice.title")}
        </label>
        <span class="text-[11px] text-text-disabled leading-snug">
          {t(hintKey ?? "podcast.create.voice.hint")}
        </span>
      </div>
      {renderGroup(t("podcast.create.voice.female"), FEMALE_VOICES)}
      {renderGroup(t("podcast.create.voice.male"), MALE_VOICES)}
    </div>
  );
}

interface LengthPickerProps {
  value: PodcastLength;
  onChange: (v: PodcastLength) => void;
}

function LengthPicker({ value, onChange }: LengthPickerProps) {
  const { t } = useTranslation();
  return (
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1">
        <label class="text-xs font-bold uppercase tracking-widest text-text-secondary">
          {t("podcast.create.length.title")}
        </label>
        <span class="text-[11px] text-text-disabled leading-snug">
          {t("podcast.create.length.hint")}
        </span>
      </div>
      <div role="radiogroup" class="grid grid-cols-3 gap-1.5">
        {LENGTHS.map((v) => {
          const selected = value === v;
          return (
            <button
              key={v}
              type="button"
              role="radio"
              aria-checked={selected}
              onClick={() => onChange(v)}
              class={`flex flex-col items-center justify-center gap-0.5 rounded-md border px-2 py-2 text-center transition-colors cursor-pointer bg-transparent ${
                selected
                  ? "border-accent-blue bg-overlay-faint"
                  : "border-border-subtle hover:bg-overlay-faint"
              }`}
            >
              <span
                class={`text-[13px] leading-tight font-medium ${
                  selected ? "text-text-primary" : "text-text-secondary"
                }`}
              >
                {t(`podcast.create.length.options.${v}.label`)}
              </span>
              <span class="text-[10px] leading-tight text-text-disabled">
                {t(`podcast.create.length.options.${v}.desc`)}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

interface SpeakerCountPickerProps {
  value: SpeakerCount;
  onChange: (v: SpeakerCount) => void;
}

function SpeakerCountPicker({ value, onChange }: SpeakerCountPickerProps) {
  const { t } = useTranslation();
  return (
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-1">
        <label class="text-xs font-bold uppercase tracking-widest text-text-secondary">
          {t("podcast.create.speakers.title")}
        </label>
        <span class="text-[11px] text-text-disabled leading-snug">
          {t("podcast.create.speakers.hint")}
        </span>
      </div>
      <div role="radiogroup" class="grid grid-cols-2 gap-1.5">
        {SPEAKER_COUNTS.map((v) => {
          const selected = value === v;
          return (
            <button
              key={v}
              type="button"
              role="radio"
              aria-checked={selected}
              onClick={() => onChange(v)}
              class={`flex flex-col items-center justify-center gap-0.5 rounded-md border px-2 py-2 text-center transition-colors cursor-pointer bg-transparent ${
                selected
                  ? "border-accent-blue bg-overlay-faint"
                  : "border-border-subtle hover:bg-overlay-faint"
              }`}
            >
              <span
                class={`text-[13px] leading-tight font-medium ${
                  selected ? "text-text-primary" : "text-text-secondary"
                }`}
              >
                {t(`podcast.create.speakers.options.${v}.label`)}
              </span>
              <span class="text-[10px] leading-tight text-text-disabled">
                {t(`podcast.create.speakers.options.${v}.desc`)}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

interface FileRowProps {
  file: FileItem;
  selected: boolean;
  onToggle: () => void;
}

function FileRow({ file, selected, onToggle }: FileRowProps) {
  const meta = fileMeta(file.name);
  return (
    <button
      type="button"
      onClick={onToggle}
      aria-pressed={selected}
      class="w-full flex items-center gap-3 p-2 rounded-lg hover:bg-overlay-faint cursor-pointer text-left bg-transparent border-none"
    >
      <input
        type="checkbox"
        checked={selected}
        readOnly
        tabIndex={-1}
        class="w-3.5 h-3.5 rounded border-overlay-medium bg-transparent text-accent-blue focus:ring-0 focus:ring-offset-0 pointer-events-none"
      />
      <span class={`material-symbols-outlined text-[20px] ${meta.tone}`}>
        {meta.icon}
      </span>
      <span class="text-sm text-text-primary truncate flex-1">{file.name}</span>
    </button>
  );
}

interface FolderNodeProps {
  folder: FolderItem;
  selectedIds: Map<string, string>;
  onToggleFile: (id: string, name: string) => void;
}

function FolderNode({ folder, selectedIds, onToggleFile }: FolderNodeProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);
  const { items, isLoading, error, hasMore, loadingMore, loadMore } =
    useFolderChildren(expanded ? folder.id : undefined);

  const subFolders = items.filter((i): i is FolderItem => i.kind === "folder");
  const subFiles = items.filter((i): i is FileItem => i.kind === "file");

  return (
    <div class="space-y-0.5">
      <button
        type="button"
        onClick={() => setExpanded((e) => !e)}
        class="w-full flex items-center gap-2 p-2 rounded-lg hover:bg-overlay-faint cursor-pointer text-left bg-transparent border-none"
      >
        <span class="material-symbols-outlined text-text-secondary text-[18px]">
          {expanded ? "keyboard_arrow_down" : "keyboard_arrow_right"}
        </span>
        <span
          class={`material-symbols-outlined text-[20px] ${folderTone(folder.id)}`}
          style={{ fontVariationSettings: "'FILL' 1" }}
        >
          folder
        </span>
        <span class="text-sm font-medium text-text-primary truncate flex-1">
          {folder.name}
        </span>
      </button>

      {expanded && (
        <div class="ml-3 pl-1 tablet:ml-6 tablet:pl-2 border-l border-border-subtle space-y-0.5">
          {isLoading && items.length === 0 && (
            <p class="p-2 text-xs text-text-disabled">
              {t("podcast.create.loading")}
            </p>
          )}
          {error && (
            <p class="p-2 text-xs text-danger">
              {t("podcast.create.errors.load")}
            </p>
          )}
          {!isLoading && !error && items.length === 0 && (
            <p class="p-2 text-xs text-text-disabled italic">
              {t("podcast.create.source.emptyFolder")}
            </p>
          )}
          {subFolders.map((sub) => (
            <FolderNode
              key={sub.id}
              folder={sub}
              selectedIds={selectedIds}
              onToggleFile={onToggleFile}
            />
          ))}
          {subFiles.map((file) => (
            <FileRow
              key={file.id}
              file={file}
              selected={selectedIds.has(file.id)}
              onToggle={() => onToggleFile(file.id, file.name)}
            />
          ))}
          {hasMore && (
            <button
              type="button"
              onClick={loadMore}
              disabled={loadingMore}
              class="w-full text-xs text-text-secondary hover:text-text-primary p-2 cursor-pointer bg-transparent border-none disabled:opacity-50"
            >
              {loadingMore
                ? t("podcast.create.loading")
                : t("podcast.create.loadMore")}
            </button>
          )}
        </div>
      )}
    </div>
  );
}

export default function PodcastNewPage() {
  const { t, i18n } = useTranslation();
  useDocumentHead({ title: "New podcast — kioku", robots: "noindex,nofollow" });
  const route = useRoute();
  const { route: navigate } = useLocation();
  const projectId = route.params.projectId;

  const { data: project, error: projectError } = useProject(projectId);
  const {
    items,
    isLoading,
    error: childrenError,
    hasMore,
    loadingMore,
    loadMore,
  } = useProjectChildren(projectId);

  const [selected, setSelected] = useState<Map<string, string>>(new Map());
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [voiceStyle, setVoiceStyle] = useState<VoiceStyle>(loadStoredVoice);
  const [voiceStyle2, setVoiceStyle2] = useState<VoiceStyle>(loadStoredVoice2);
  const [speakerCount, setSpeakerCount] = useState<SpeakerCount>(
    loadStoredSpeakerCount,
  );
  const [length, setLength] = useState<PodcastLength>(loadStoredLength);

  useEffect(() => {
    try {
      localStorage.setItem(VOICE_STORAGE_KEY, voiceStyle);
    } catch {
      // ignore (private mode / quota)
    }
  }, [voiceStyle]);

  useEffect(() => {
    try {
      localStorage.setItem(VOICE_2_STORAGE_KEY, voiceStyle2);
    } catch {
      // ignore (private mode / quota)
    }
  }, [voiceStyle2]);

  useEffect(() => {
    try {
      localStorage.setItem(SPEAKER_COUNT_STORAGE_KEY, String(speakerCount));
    } catch {
      // ignore (private mode / quota)
    }
  }, [speakerCount]);

  useEffect(() => {
    if (speakerCount === 2 && voiceStyle === voiceStyle2) {
      const fallback = ALL_VOICES.find((v) => v !== voiceStyle);
      if (fallback) setVoiceStyle2(fallback);
    }
  }, [speakerCount, voiceStyle, voiceStyle2]);

  useEffect(() => {
    try {
      localStorage.setItem(LENGTH_STORAGE_KEY, length);
    } catch {
      // ignore (private mode / quota)
    }
  }, [length]);
  const [playingVoice, setPlayingVoice] = useState<VoiceStyle | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  useEffect(() => {
    return () => {
      audioRef.current?.pause();
      audioRef.current = null;
    };
  }, []);

  const previewVoice = (v: VoiceStyle) => {
    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current.currentTime = 0;
    }
    if (playingVoice === v) {
      setPlayingVoice(null);
      return;
    }
    const lang = i18n.language.startsWith("ja") ? "ja" : "en";
    const src =
      lang === "ja" ? `/voice-samples/${v}.wav` : `/voice-samples/${v}_en.wav`;
    const audio = new Audio(src);
    audio.onended = () => setPlayingVoice(null);
    audio.onerror = () => setPlayingVoice(null);
    audio.play().catch(() => setPlayingVoice(null));
    audioRef.current = audio;
    setPlayingVoice(v);
  };

  const folders = items.filter((i): i is FolderItem => i.kind === "folder");
  const files = items.filter((i): i is FileItem => i.kind === "file");

  const derivedName = Array.from(selected.values()).join(", ").slice(0, 256);
  const displayName = name || derivedName;

  const toggleFile = (id: string, fileName: string) => {
    setSelected((prev) => {
      const next = new Map(prev);
      if (next.has(id)) next.delete(id);
      else next.set(id, fileName);
      return next;
    });
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    if (!projectId) return;
    if (selected.size === 0) {
      setSubmitError(t("podcast.create.errors.filesRequired"));
      return;
    }
    const finalName = name.trim() || derivedName;
    setSubmitting(true);
    setSubmitError(null);
    try {
      const body: CreatePodcastBody = {
        name: finalName,
        description: description.trim(),
        used_file_ids: Array.from(selected.keys()),
        voice_style: voiceStyle,
        voice_style_2: speakerCount === 2 ? voiceStyle2 : null,
        length,
      };
      await kyInstance
        .post(`projects/${projectId}/podcasts`, { json: body })
        .json<CreatePodcast200>();
      navigate(`/projects/${projectId}/podcasts`);
    } catch {
      setSubmitError(t("podcast.create.errors.failed"));
      setSubmitting(false);
    }
  };

  const canSubmit = !submitting && selected.size > 0 && !!projectId;

  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-[var(--sidebar-width)] p-4 tablet:p-8 h-[calc(100vh-3.5rem)] overflow-hidden flex flex-col transition-[margin-left] duration-200 ease-in-out">
        <header class="mb-6 flex flex-col gap-2">
          <nav class="flex items-center gap-1.5 text-text-secondary text-sm font-medium flex-wrap">
            <a
              href="/podcast"
              class="hover:text-text-primary no-underline text-inherit"
            >
              {t("nav.podcast")}
            </a>
            <span class="material-symbols-outlined text-[16px] select-none">
              chevron_right
            </span>
            <a
              href={`/projects/${projectId}/podcasts`}
              class="hover:text-text-primary no-underline text-inherit truncate max-w-[160px]"
            >
              {project?.name ?? (projectError ? "—" : "...")}
            </a>
            <span class="material-symbols-outlined text-[16px] select-none">
              chevron_right
            </span>
            <span class="text-text-primary">{t("podcast.create.crumb")}</span>
          </nav>
          <h1 class="heading-h2">{t("podcast.create.title")}</h1>
        </header>

        <form
          onSubmit={handleSubmit}
          class="flex-1 grid grid-cols-12 gap-4 tablet:gap-6 overflow-hidden min-h-0"
        >
          <section class="col-span-12 lg:col-span-7 flex flex-col bg-surface-dark border border-border-subtle rounded-[12px] overflow-hidden min-h-0">
            <div class="p-4 border-b border-border-subtle flex items-center justify-between">
              <h3 class="text-sm font-bold text-text-primary">
                {t("podcast.create.source.title")}
              </h3>
              <span class="text-xs text-text-secondary">
                {t("podcast.create.source.count", { count: selected.size })}
              </span>
            </div>
            <div class="flex-1 overflow-y-auto overflow-x-hidden p-2 space-y-1">
              {childrenError && (
                <p class="p-2 text-sm text-danger">
                  {t("project.errors.children")}
                </p>
              )}
              {isLoading && items.length === 0 && (
                <p class="p-2 text-sm text-text-secondary">
                  {t("podcast.create.loading")}
                </p>
              )}
              {!isLoading && !childrenError && items.length === 0 && (
                <p class="p-4 text-sm text-text-secondary italic text-center">
                  {t("podcast.create.source.empty")}
                </p>
              )}
              {folders.map((folder) => (
                <FolderNode
                  key={folder.id}
                  folder={folder}
                  selectedIds={selected}
                  onToggleFile={toggleFile}
                />
              ))}
              {files.map((file) => (
                <FileRow
                  key={file.id}
                  file={file}
                  selected={selected.has(file.id)}
                  onToggle={() => toggleFile(file.id, file.name)}
                />
              ))}
              {hasMore && (
                <button
                  type="button"
                  onClick={loadMore}
                  disabled={loadingMore}
                  class="w-full text-xs text-text-secondary hover:text-text-primary p-2 cursor-pointer bg-transparent border-none disabled:opacity-50"
                >
                  {loadingMore
                    ? t("podcast.create.loading")
                    : t("podcast.create.loadMore")}
                </button>
              )}
            </div>
          </section>

          <aside class="col-span-12 lg:col-span-5 flex flex-col gap-6 overflow-y-auto min-h-0">
            <div class="bg-surface-dark border border-border-subtle rounded-[12px] p-6 flex flex-col gap-6">
              <h3 class="text-sm font-bold text-text-primary border-b border-border-subtle pb-4">
                {t("podcast.create.settings.title")}
              </h3>

              <div class="flex flex-col gap-2">
                <label
                  for="podcast-name"
                  class="text-xs font-bold uppercase tracking-widest text-text-secondary"
                >
                  {t("podcast.create.fields.name")}
                </label>
                <input
                  id="podcast-name"
                  type="text"
                  value={displayName}
                  onInput={(e) => setName((e.target as HTMLInputElement).value)}
                  placeholder={t("podcast.create.placeholders.name")}
                  maxLength={256}
                  class="input-field"
                />
              </div>

              <div class="flex flex-col gap-2">
                <label
                  for="podcast-description"
                  class="text-xs font-bold uppercase tracking-widest text-text-secondary"
                >
                  {t("podcast.create.fields.description")}{" "}
                  <span class="text-text-disabled font-normal normal-case tracking-normal">
                    {t("podcast.create.fields.optional")}
                  </span>
                </label>
                <textarea
                  id="podcast-description"
                  value={description}
                  onInput={(e) =>
                    setDescription((e.target as HTMLTextAreaElement).value)
                  }
                  placeholder={t("podcast.create.placeholders.description")}
                  rows={4}
                  maxLength={1024}
                  class="textarea-field"
                />
              </div>

              <SpeakerCountPicker
                value={speakerCount}
                onChange={(v) => setSpeakerCount(v)}
              />

              <VoicePicker
                value={voiceStyle}
                onChange={(v) => setVoiceStyle(v)}
                playing={playingVoice}
                onPreview={previewVoice}
                titleKey={
                  speakerCount === 2
                    ? "podcast.create.voice.titleSpeaker1"
                    : "podcast.create.voice.title"
                }
                disabledVoice={speakerCount === 2 ? voiceStyle2 : undefined}
              />

              {speakerCount === 2 && (
                <VoicePicker
                  value={voiceStyle2}
                  onChange={(v) => setVoiceStyle2(v)}
                  playing={playingVoice}
                  onPreview={previewVoice}
                  titleKey="podcast.create.voice.titleSpeaker2"
                  disabledVoice={voiceStyle}
                />
              )}

              <LengthPicker value={length} onChange={(v) => setLength(v)} />
            </div>

            <div class="mt-auto flex flex-col gap-3">
              {submitError && (
                <p class="text-sm text-danger text-center">{submitError}</p>
              )}
              <button
                type="submit"
                disabled={!canSubmit}
                class="btn-primary w-full py-3 text-base shadow-[0_8px_24px_rgba(0,0,0,0.2)]"
              >
                <span
                  class="material-symbols-outlined"
                  style={{ fontVariationSettings: "'FILL' 1" }}
                >
                  auto_awesome
                </span>
                {submitting
                  ? t("podcast.create.submitting")
                  : t("podcast.create.submit")}
              </button>
              <p class="text-center text-[11px] text-text-disabled leading-relaxed">
                {t("podcast.create.notice")}
              </p>
            </div>
          </aside>
        </form>
      </main>
    </div>
  );
}
