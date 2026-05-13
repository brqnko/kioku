import { useState } from "preact/hooks";
import { useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { kyInstance } from "../api/mutator";
import { useProject, useProjectChildren } from "../hooks/useProject";
import { useFolderChildren } from "../hooks/useFolder";
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
  const {
    items,
    isLoading,
    error,
    hasMore,
    loadingMore,
    loadMore,
  } = useFolderChildren(expanded ? folder.id : undefined);

  const subFolders = items.filter(
    (i): i is FolderItem => i.kind === "folder",
  );
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
  const { t } = useTranslation();
  const route = useRoute();
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
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

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
      };
      await kyInstance
        .post(`projects/${projectId}/podcasts`, { json: body })
        .json<CreatePodcast200>();
      window.location.href = `/projects/${projectId}/podcasts`;
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
              {project?.name ?? (projectError ? "—" : "...")}
            </a>
            <span class="material-symbols-outlined text-[16px]">
              chevron_right
            </span>
            <a
              href={`/projects/${projectId}/podcasts`}
              class="hover:text-text-primary no-underline text-inherit"
            >
              {t("podcast.list.crumb")}
            </a>
            <span class="material-symbols-outlined text-[16px]">
              chevron_right
            </span>
            <span class="text-text-primary">
              {t("podcast.create.crumb")}
            </span>
          </nav>
          <h1 class="text-[22px] leading-[1.27] font-bold tracking-tight">
            {t("podcast.create.title")}
          </h1>
        </header>

        <form
          onSubmit={handleSubmit}
          class="flex-1 grid grid-cols-12 gap-4 tablet:gap-6 overflow-hidden min-h-0"
        >
          <section class="col-span-12 lg:col-span-7 flex flex-col bg-surface-dark border border-border-subtle rounded-xl overflow-hidden min-h-0">
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
            <div class="bg-surface-dark border border-border-subtle rounded-xl p-4 tablet:p-6 flex flex-col gap-6">
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
                  onInput={(e) =>
                    setName((e.target as HTMLInputElement).value)
                  }
                  placeholder={t("podcast.create.placeholders.name")}
                  maxLength={256}
                  class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50"
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
                  class="w-full bg-surface-container-high border border-border-dark rounded-lg px-4 py-2.5 text-base text-text-primary placeholder:text-text-disabled focus:outline-none focus:ring-1 focus:ring-accent-blue/50 focus:border-accent-blue/50 resize-none"
                />
              </div>
            </div>

            <div class="mt-auto flex flex-col gap-3">
              {submitError && (
                <p class="text-sm text-danger text-center">{submitError}</p>
              )}
              <button
                type="submit"
                disabled={!canSubmit}
                class="w-full bg-cta text-cta-fg font-bold py-4 rounded-lg flex items-center justify-center gap-3 hover:bg-cta-hover shadow-[0_8px_24px_rgba(0,0,0,0.2)] cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
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
