import { useState } from "preact/hooks";
import { useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import { AppLayout } from "../components/AppLayout";
import { CreateFolderDialog } from "../components/CreateFolderDialog";
import { DeleteItemDialog } from "../components/DeleteItemDialog";
import { PageHeader } from "../components/PageHeader";
import {
  ResourceTable,
  type ResourceActionTarget,
  type ResourceTableItem,
} from "../components/ResourceTable";
import { RenameItemDialog } from "../components/RenameItemDialog";
import { StateMessage } from "../components/StateMessage";
import { UploadDialog } from "../components/UploadDialog";
import { useProject, useProjectChildren } from "../hooks/useProject";
import { useDocumentHead } from "../hooks/useDocumentHead";

export default function ProjectPage() {
  const { t } = useTranslation();
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
  const [deleteTarget, setDeleteTarget] = useState<ResourceActionTarget | null>(
    null,
  );
  const [renameTarget, setRenameTarget] = useState<ResourceActionTarget | null>(
    null,
  );

  const title = project?.name ?? t("project.loading");

  return (
    <AppLayout>
      <PageHeader
        breadcrumbs={[
          { label: t("project.breadcrumb.library"), href: "/library" },
          { label: project?.name ?? (projectError ? "-" : "...") },
        ]}
        title={title}
        description={project?.description}
        actions={
          <>
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
              <span class="material-symbols-outlined text-[20px]">
                note_add
              </span>
              {t("project.upload.label")}
            </button>
          </>
        }
      />

      <section class="content-section">
        <div class="section-heading">
          <h2>
            <span class="material-symbols-outlined text-text-secondary text-[18px]">
              folder_open
            </span>
            {t("project.allFiles")}
          </h2>
        </div>

        {projectError && (
          <StateMessage tone="danger" className="mb-4">
            {t("project.errors.load")}
          </StateMessage>
        )}
        {childrenError && (
          <StateMessage tone="danger" className="mb-4">
            {t("project.errors.children")}
          </StateMessage>
        )}

        <ResourceTable
          items={items as ResourceTableItem[]}
          loading={childrenLoading}
          emptyLabel={t("project.empty")}
          onEdit={setRenameTarget}
          onDelete={setDeleteTarget}
        />

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
    </AppLayout>
  );
}
