import { useEffect, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useFolderChildren } from "../hooks/useFolder";
import { fileMeta, folderTone, formatSize } from "../utils/file";
import type {
  ListFolderChildren200ItemsItem,
  ListProjectChildren200ItemsItem,
} from "../api/generated/backend.schemas";
import { RowActionMenu } from "./RowActionMenu";
import { StateMessage } from "./StateMessage";

export type ProjectExplorerParentKind = "project" | "folder";
export type ProjectExplorerRefresh = () => unknown | Promise<unknown>;
export type ProjectExplorerItem =
  | ListProjectChildren200ItemsItem
  | ListFolderChildren200ItemsItem;
export type ProjectExplorerFolderItem = Extract<
  ProjectExplorerItem,
  { kind: "folder" }
>;
export type ProjectExplorerFileItem = Extract<
  ProjectExplorerItem,
  { kind: "file" }
>;

export interface ProjectExplorerActionTarget {
  kind: "file" | "folder";
  id: string;
  name: string;
  description?: string | null;
}

interface ProjectExplorerProps {
  parentId: string;
  parentKind: ProjectExplorerParentKind;
  items: ProjectExplorerItem[];
  loading: boolean;
  emptyLabel: string;
  hasMore: boolean;
  loadingMore: boolean;
  loadMore: () => void;
  selectedFileId: string | null;
  autoExpandFolderIds: Set<string>;
  onSelectFile: (file: ProjectExplorerFileItem) => void;
  onEdit: (
    target: ProjectExplorerActionTarget,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onDelete: (
    target: ProjectExplorerActionTarget,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onCreateFolder: (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onCreateFile: (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  refresh: ProjectExplorerRefresh;
}

interface ProjectExplorerNodeProps {
  item: ProjectExplorerItem;
  selectedFileId: string | null;
  autoExpandFolderIds: Set<string>;
  onSelectFile: (file: ProjectExplorerFileItem) => void;
  onEdit: (
    target: ProjectExplorerActionTarget,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onDelete: (
    target: ProjectExplorerActionTarget,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onCreateFolder: (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onCreateFile: (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  refreshParent: ProjectExplorerRefresh;
}

interface ExplorerCreateActionsProps {
  parentId: string;
  parentKind: ProjectExplorerParentKind;
  refresh: ProjectExplorerRefresh;
  onCreateFolder: (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onCreateFile: (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
}

interface ProjectExplorerFolderProps {
  folder: ProjectExplorerFolderItem;
  selectedFileId: string | null;
  autoExpandFolderIds: Set<string>;
  onSelectFile: (file: ProjectExplorerFileItem) => void;
  onEdit: (
    target: ProjectExplorerActionTarget,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onDelete: (
    target: ProjectExplorerActionTarget,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onCreateFolder: (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  onCreateFile: (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => void;
  refreshParent: ProjectExplorerRefresh;
}

interface ProjectExplorerFileProps {
  file: ProjectExplorerFileItem;
  selected: boolean;
  onSelect: () => void;
  onEdit: () => void;
  onDelete: () => void;
}

function toActionTarget(
  item: ProjectExplorerItem,
): ProjectExplorerActionTarget {
  return {
    kind: item.kind,
    id: item.id,
    name: item.name,
    description: item.description,
  };
}

function orderExplorerItems(items: ProjectExplorerItem[]) {
  const folders = items.filter(
    (item): item is ProjectExplorerFolderItem => item.kind === "folder",
  );
  const files = items.filter(
    (item): item is ProjectExplorerFileItem => item.kind === "file",
  );
  return [...folders, ...files];
}

function ExplorerCreateActions({
  parentId,
  parentKind,
  refresh,
  onCreateFolder,
  onCreateFile,
}: ExplorerCreateActionsProps) {
  const { t } = useTranslation();

  return (
    <div class="mt-1 px-1 pb-0.5 pt-1">
      <div class="flex items-center gap-2">
        <span class="h-px min-w-4 flex-1 bg-border-subtle" />
        <span class="shrink-0 text-[11px] font-medium text-text-disabled">
          {t("project.createActions.add")}
        </span>
        <div class="flex shrink-0 items-center gap-0.5 rounded-lg border border-border-subtle bg-surface-container-low/70 p-0.5">
          <button
            type="button"
            onClick={() => onCreateFolder(parentId, parentKind, refresh)}
            aria-label={t("project.newFolder")}
            title={t("project.newFolder")}
            class="inline-flex h-7 items-center gap-1.5 rounded-md border-none bg-transparent px-2 text-xs font-medium text-text-secondary hover:bg-overlay-faint hover:text-text-primary"
          >
            <span class="material-symbols-outlined text-[16px]">
              create_new_folder
            </span>
            {t("project.createActions.folder")}
          </button>
          <button
            type="button"
            onClick={() => onCreateFile(parentId, parentKind, refresh)}
            aria-label={t("project.upload.label")}
            title={t("project.upload.label")}
            class="inline-flex h-7 items-center gap-1.5 rounded-md border-none bg-transparent px-2 text-xs font-medium text-text-secondary hover:bg-overlay-faint hover:text-text-primary"
          >
            <span class="material-symbols-outlined text-[16px]">note_add</span>
            {t("project.createActions.file")}
          </button>
        </div>
      </div>
    </div>
  );
}

export function ProjectExplorer({
  parentId,
  parentKind,
  items,
  loading,
  emptyLabel,
  hasMore,
  loadingMore,
  loadMore,
  selectedFileId,
  autoExpandFolderIds,
  onSelectFile,
  onEdit,
  onDelete,
  onCreateFolder,
  onCreateFile,
  refresh,
}: ProjectExplorerProps) {
  const { t } = useTranslation();
  const orderedItems = orderExplorerItems(items);

  return (
    <div class="shrink-0 overflow-hidden rounded-lg border border-border-subtle bg-surface-dark">
      <div class="space-y-0.5 p-2">
        {loading && items.length === 0 && (
          <StateMessage className="py-5 text-center">
            {t("project.loading")}
          </StateMessage>
        )}

        {!loading && items.length === 0 && (
          <StateMessage className="py-5 text-center">{emptyLabel}</StateMessage>
        )}

        {orderedItems.map((item) => (
          <ProjectExplorerNode
            key={`${item.kind}-${item.id}`}
            item={item}
            selectedFileId={selectedFileId}
            autoExpandFolderIds={autoExpandFolderIds}
            onSelectFile={onSelectFile}
            onEdit={onEdit}
            onDelete={onDelete}
            onCreateFolder={onCreateFolder}
            onCreateFile={onCreateFile}
            refreshParent={refresh}
          />
        ))}

        {hasMore && (
          <button
            type="button"
            onClick={loadMore}
            disabled={loadingMore}
            class="w-full rounded-lg border-none bg-transparent p-2 text-xs text-text-secondary hover:bg-overlay-faint hover:text-text-primary disabled:opacity-50"
          >
            {loadingMore ? t("project.loading") : t("library.loadMore")}
          </button>
        )}

        <ExplorerCreateActions
          parentId={parentId}
          parentKind={parentKind}
          refresh={refresh}
          onCreateFolder={onCreateFolder}
          onCreateFile={onCreateFile}
        />
      </div>
    </div>
  );
}

function ProjectExplorerNode({
  item,
  selectedFileId,
  autoExpandFolderIds,
  onSelectFile,
  onEdit,
  onDelete,
  onCreateFolder,
  onCreateFile,
  refreshParent,
}: ProjectExplorerNodeProps) {
  if (item.kind === "folder") {
    return (
      <ProjectExplorerFolder
        folder={item}
        selectedFileId={selectedFileId}
        autoExpandFolderIds={autoExpandFolderIds}
        onSelectFile={onSelectFile}
        onEdit={onEdit}
        onDelete={onDelete}
        onCreateFolder={onCreateFolder}
        onCreateFile={onCreateFile}
        refreshParent={refreshParent}
      />
    );
  }

  return (
    <ProjectExplorerFile
      file={item}
      selected={selectedFileId === item.id}
      onSelect={() => onSelectFile(item)}
      onEdit={() => onEdit(toActionTarget(item), refreshParent)}
      onDelete={() => onDelete(toActionTarget(item), refreshParent)}
    />
  );
}

function ProjectExplorerFolder({
  folder,
  selectedFileId,
  autoExpandFolderIds,
  onSelectFile,
  onEdit,
  onDelete,
  onCreateFolder,
  onCreateFile,
  refreshParent,
}: ProjectExplorerFolderProps) {
  const { t } = useTranslation();
  const shouldAutoExpand = autoExpandFolderIds.has(folder.id);
  const [expanded, setExpanded] = useState(shouldAutoExpand);
  const { items, isLoading, error, hasMore, loadingMore, loadMore, refresh } =
    useFolderChildren(expanded ? folder.id : undefined);
  const orderedItems = orderExplorerItems(items);

  useEffect(() => {
    if (shouldAutoExpand) setExpanded(true);
  }, [shouldAutoExpand]);

  return (
    <div class="space-y-0.5">
      <div class="group flex items-center gap-1 rounded-lg pr-1 hover:bg-overlay-faint">
        <button
          type="button"
          onClick={() => setExpanded((value) => !value)}
          aria-expanded={expanded}
          class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 border-none bg-transparent p-2 text-left"
        >
          <span class="material-symbols-outlined text-[18px] text-text-secondary">
            {expanded ? "keyboard_arrow_down" : "keyboard_arrow_right"}
          </span>
          <span
            class={`material-symbols-outlined text-[20px] ${folderTone(folder.id)}`}
            style={{ fontVariationSettings: "'FILL' 1" }}
          >
            folder
          </span>
          <span class="min-w-0 flex-1 truncate text-sm font-medium text-text-primary">
            {folder.name}
          </span>
        </button>
        <RowActionMenu
          ariaLabel={t("project.table.more")}
          onEdit={() => onEdit(toActionTarget(folder), refreshParent)}
          onDelete={() => onDelete(toActionTarget(folder), refreshParent)}
        />
      </div>

      {expanded && (
        <div class="ml-3 space-y-0.5 border-l border-border-subtle pl-1 tablet:ml-6 tablet:pl-2">
          {isLoading && items.length === 0 && (
            <p class="p-2 text-xs text-text-disabled">
              {t("project.loading")}
            </p>
          )}
          {error && (
            <p class="p-2 text-xs text-danger">
              {t("project.errors.children")}
            </p>
          )}
          {orderedItems.map((child) => (
            <ProjectExplorerNode
              key={`${child.kind}-${child.id}`}
              item={child}
              selectedFileId={selectedFileId}
              autoExpandFolderIds={autoExpandFolderIds}
              onSelectFile={onSelectFile}
              onEdit={onEdit}
              onDelete={onDelete}
              onCreateFolder={onCreateFolder}
              onCreateFile={onCreateFile}
              refreshParent={refresh}
            />
          ))}
          {hasMore && (
            <button
              type="button"
              onClick={loadMore}
              disabled={loadingMore}
              class="w-full rounded-lg border-none bg-transparent p-2 text-xs text-text-secondary hover:bg-overlay-faint hover:text-text-primary disabled:opacity-50"
            >
              {loadingMore ? t("project.loading") : t("library.loadMore")}
            </button>
          )}
          <ExplorerCreateActions
            parentId={folder.id}
            parentKind="folder"
            refresh={refresh}
            onCreateFolder={onCreateFolder}
            onCreateFile={onCreateFile}
          />
        </div>
      )}
    </div>
  );
}

function ProjectExplorerFile({
  file,
  selected,
  onSelect,
  onEdit,
  onDelete,
}: ProjectExplorerFileProps) {
  const { t } = useTranslation();
  const meta = fileMeta(file.name);

  return (
    <div
      class={`group flex items-center gap-1 rounded-lg pr-1 ${
        selected ? "bg-overlay-soft" : "hover:bg-overlay-faint"
      }`}
    >
      <button
        type="button"
        onClick={onSelect}
        aria-pressed={selected}
        class="flex min-w-0 flex-1 cursor-pointer items-center gap-3 border-none bg-transparent p-2 text-left"
      >
        <span class={`material-symbols-outlined text-[20px] ${meta.tone}`}>
          {meta.icon}
        </span>
        <span class="min-w-0 flex-1 truncate text-sm text-text-primary">
          {file.name}
        </span>
        <span class="hidden shrink-0 text-xs text-text-disabled tablet:inline">
          {formatSize(file.file_size)}
        </span>
      </button>
      <RowActionMenu
        ariaLabel={t("project.table.more")}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    </div>
  );
}
