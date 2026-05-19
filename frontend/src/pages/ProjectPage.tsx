import { useState } from "preact/hooks";
import { useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { CreateFolderDialog } from "../components/CreateFolderDialog";
import { UploadDialog } from "../components/UploadDialog";
import { DeleteItemDialog } from "../components/DeleteItemDialog";
import { RenameItemDialog } from "../components/RenameItemDialog";
import { RowActionMenu } from "../components/RowActionMenu";
import { useProject, useProjectChildren } from "../hooks/useProject";
import { useDocumentHead } from "../hooks/useDocumentHead";
import { fileMeta, folderTone, formatDate, formatSize } from "../utils/file";
import type { ListProjectChildren200ItemsItem } from "../api/generated/backend.schemas";

export default function ProjectPage() {
  const { t, i18n } = useTranslation();
  useDocumentHead({ title: "Project — kioku", robots: "noindex,nofollow" });
  const route = useRoute();
  const projectId = route.params.projectId;

  const {
    data: project,
    error: projectError,
    mutate: refreshProject,
  } = useProject(projectId);
  const {
    items,
    error: childrenError,
    isLoading: childrenLoading,
    hasMore,
    loadingMore,
    loadMore,
    refresh: refreshChildren,
  } = useProjectChildren(projectId);

  const [folderDialogOpen, setFolderDialogOpen] = useState(false);
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false);
  const [editProjectOpen, setEditProjectOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<{
    kind: "file" | "folder";
    id: string;
    name: string;
  } | null>(null);
  const [renameTarget, setRenameTarget] = useState<{
    kind: "file" | "folder";
    id: string;
    name: string;
    description?: string | null;
  } | null>(null);

  const folders = items.filter(
    (i): i is Extract<ListProjectChildren200ItemsItem, { kind: "folder" }> =>
      i.kind === "folder",
  );
  const files = items.filter(
    (i): i is Extract<ListProjectChildren200ItemsItem, { kind: "file" }> =>
      i.kind === "file",
  );

  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-[var(--sidebar-width)] p-4 tablet:p-8 h-[calc(100vh-3.5rem)] overflow-y-auto transition-[margin-left] duration-200 ease-in-out">
        <section class="mb-8 flex items-end justify-between flex-wrap gap-4">
          <div class="flex flex-col gap-3">
            <nav class="flex items-center gap-2 text-text-secondary text-sm font-medium">
              <a href="/library" class="hover:text-text-primary no-underline text-inherit">
                {t("project.breadcrumb.library")}
              </a>
              <span class="material-symbols-outlined text-[16px]">
                chevron_right
              </span>
              <span class="text-text-primary">
                {project?.name ?? (projectError ? "—" : "...")}
              </span>
            </nav>
            <h1 class="heading-h2">
              {project?.name ?? t("project.loading")}
            </h1>
            {project?.description && (
              <p class="text-body text-text-secondary max-w-2xl">
                {project.description}
              </p>
            )}
          </div>
          <div class="flex items-center gap-2 tablet:gap-3 flex-wrap">
            <button
              type="button"
              onClick={() => setEditProjectOpen(true)}
              disabled={!project}
              class="btn-secondary"
            >
              <span class="material-symbols-outlined text-[20px]">edit</span>
              {t("project.editProject")}
            </button>
            <button
              type="button"
              onClick={() => setFolderDialogOpen(true)}
              class="btn-secondary"
            >
              <span class="material-symbols-outlined text-[20px]">
                create_new_folder
              </span>
              {t("project.newFolder")}
            </button>
            <button
              type="button"
              onClick={() => setUploadDialogOpen(true)}
              class="btn-secondary"
            >
              <span class="material-symbols-outlined text-[20px]">note_add</span>
              {t("project.upload.label")}
            </button>
          </div>
        </section>

        <section>
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-base font-bold flex items-center gap-2">
              <span class="material-symbols-outlined text-text-secondary text-[18px]">
                folder_open
              </span>
              {t("project.allFiles")}
            </h2>
          </div>

          {projectError && (
            <p class="text-sm text-danger mb-4">{t("project.errors.load")}</p>
          )}
          {childrenError && (
            <p class="text-sm text-danger mb-4">
              {t("project.errors.children")}
            </p>
          )}

          <div class="bg-surface-dark border border-border-subtle rounded-[12px] overflow-hidden">
            <table class="w-full text-left border-collapse">
              <thead>
                <tr class="bg-surface-container text-text-secondary text-sm border-b border-border-subtle">
                  <th class="px-3 py-3 tablet:px-6 font-semibold">
                    {t("project.table.name")}
                  </th>
                  <th class="hidden tablet:table-cell px-3 py-3 tablet:px-6 font-semibold">
                    {t("project.table.type")}
                  </th>
                  <th class="hidden tablet:table-cell px-3 py-3 tablet:px-6 font-semibold">
                    {t("project.table.size")}
                  </th>
                  <th class="px-3 py-3 tablet:px-6 font-semibold">
                    {t("project.table.modified")}
                  </th>
                  <th class="px-3 py-3 tablet:px-6 font-semibold text-right">
                    {t("project.table.action")}
                  </th>
                </tr>
              </thead>
              <tbody class="divide-y divide-border-subtle">
                {childrenLoading && items.length === 0 && (
                  <tr>
                    <td
                      colSpan={5}
                      class="px-3 py-8 tablet:px-6 text-sm text-text-secondary text-center"
                    >
                      {t("project.loading")}
                    </td>
                  </tr>
                )}

                {!childrenLoading && items.length === 0 && !childrenError && (
                  <tr>
                    <td
                      colSpan={5}
                      class="px-3 py-8 tablet:px-6 text-sm text-text-secondary text-center"
                    >
                      {t("project.empty")}
                    </td>
                  </tr>
                )}

                {folders.map((folder) => (
                  <tr
                    key={folder.id}
                    class="hover:bg-overlay-faint group cursor-pointer"
                    onClick={() => {
                      window.location.href = `/folders/${folder.id}`;
                    }}
                  >
                    <td class="px-3 py-3 tablet:px-6">
                      <div class="flex items-center gap-3 min-w-0">
                        <span
                          class={`material-symbols-outlined ${folderTone(folder.id)}`}
                          style={{ fontVariationSettings: "'FILL' 1" }}
                        >
                          folder
                        </span>
                        <div class="flex flex-col min-w-0">
                          <span class="text-sm font-medium text-text-primary truncate">
                            {folder.name}
                          </span>
                          {folder.description && (
                            <span class="text-xs text-text-disabled truncate">
                              {folder.description}
                            </span>
                          )}
                        </div>
                      </div>
                    </td>
                    <td class="hidden tablet:table-cell px-3 py-3 tablet:px-6 text-sm text-text-secondary">
                      {t("project.table.folder")}
                    </td>
                    <td class="hidden tablet:table-cell px-3 py-3 tablet:px-6 text-sm text-text-secondary">—</td>
                    <td class="px-3 py-3 tablet:px-6 text-sm text-text-secondary">
                      {formatDate(folder.changed_at, i18n.language)}
                    </td>
                    <td class="px-3 py-3 tablet:px-6 text-right">
                      <RowActionMenu
                        ariaLabel={t("project.table.more")}
                        onEdit={() =>
                          setRenameTarget({
                            kind: "folder",
                            id: folder.id,
                            name: folder.name,
                            description: folder.description,
                          })
                        }
                        onDelete={() =>
                          setDeleteTarget({
                            kind: "folder",
                            id: folder.id,
                            name: folder.name,
                          })
                        }
                      />
                    </td>
                  </tr>
                ))}

                {files.map((file) => {
                  const meta = fileMeta(file.name);
                  return (
                    <tr
                      key={file.id}
                      class="hover:bg-overlay-faint group cursor-pointer"
                      onClick={() => {
                        window.location.href = `/files/${file.id}`;
                      }}
                    >
                      <td class="px-3 py-3 tablet:px-6">
                        <div class="flex items-center gap-3 min-w-0">
                          <span
                            class={`material-symbols-outlined ${meta.tone}`}
                          >
                            {meta.icon}
                          </span>
                          <div class="flex flex-col min-w-0">
                            <span class="text-sm font-medium text-text-primary truncate">
                              {file.name}
                            </span>
                            {file.description && (
                              <span class="text-xs text-text-disabled truncate">
                                {file.description}
                              </span>
                            )}
                          </div>
                        </div>
                      </td>
                      <td class="hidden tablet:table-cell px-3 py-3 tablet:px-6 text-sm text-text-secondary">
                        {meta.type}
                      </td>
                      <td class="hidden tablet:table-cell px-3 py-3 tablet:px-6 text-sm text-text-secondary">
                        {formatSize(file.file_size)}
                      </td>
                      <td class="px-3 py-3 tablet:px-6 text-sm text-text-secondary">
                        {formatDate(file.changed_at, i18n.language)}
                      </td>
                      <td class="px-3 py-3 tablet:px-6 text-right">
                        <RowActionMenu
                          ariaLabel={t("project.table.more")}
                          onEdit={() =>
                            setRenameTarget({
                              kind: "file",
                              id: file.id,
                              name: file.name,
                              description: file.description,
                            })
                          }
                          onDelete={() =>
                            setDeleteTarget({
                              kind: "file",
                              id: file.id,
                              name: file.name,
                            })
                          }
                        />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>

          {hasMore && (
            <div class="mt-6 flex justify-center">
              <button
                type="button"
                onClick={loadMore}
                disabled={loadingMore}
                class="btn-secondary"
              >
                {loadingMore ? t("project.loading") : t("library.loadMore")}
              </button>
            </div>
          )}
        </section>
      </main>

      {projectId && (
        <>
          <CreateFolderDialog
            open={folderDialogOpen}
            onClose={() => setFolderDialogOpen(false)}
            parentId={projectId}
            parentKind="project"
            onSuccess={refreshChildren}
          />
          <UploadDialog
            open={uploadDialogOpen}
            onClose={() => setUploadDialogOpen(false)}
            parentId={projectId}
            parentKind="project"
            onSuccess={refreshChildren}
          />
          <DeleteItemDialog
            open={deleteTarget !== null}
            onClose={() => setDeleteTarget(null)}
            kind={deleteTarget?.kind ?? "file"}
            id={deleteTarget?.id ?? ""}
            name={deleteTarget?.name ?? ""}
            onSuccess={refreshChildren}
          />
          <RenameItemDialog
            open={renameTarget !== null}
            onClose={() => setRenameTarget(null)}
            kind={renameTarget?.kind ?? "file"}
            id={renameTarget?.id ?? ""}
            initialName={renameTarget?.name ?? ""}
            initialDescription={renameTarget?.description ?? ""}
            onSuccess={refreshChildren}
          />
          <RenameItemDialog
            open={editProjectOpen}
            onClose={() => setEditProjectOpen(false)}
            kind="project"
            id={projectId}
            initialName={project?.name ?? ""}
            initialDescription={project?.description ?? ""}
            onSuccess={refreshProject}
          />
        </>
      )}
    </div>
  );
}
