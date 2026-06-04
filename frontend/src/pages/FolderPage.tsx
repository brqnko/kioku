import { useEffect, useState } from "preact/hooks";
import { useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import { AppLayout } from "../components/AppLayout";
import { CreateFolderDialog } from "../components/CreateFolderDialog";
import { DeleteItemDialog } from "../components/DeleteItemDialog";
import {
  PageHeader,
  type BreadcrumbItem,
} from "../components/PageHeader";
import {
  ResourceTable,
  type ResourceActionTarget,
  type ResourceTableItem,
} from "../components/ResourceTable";
import { RenameItemDialog } from "../components/RenameItemDialog";
import { StateMessage } from "../components/StateMessage";
import { UploadDialog } from "../components/UploadDialog";
import {
  fetchFolderAncestors,
  useFolder,
  useFolderChildren,
  type BreadcrumbAncestor,
} from "../hooks/useFolder";
import { useDocumentHead } from "../hooks/useDocumentHead";

export default function FolderPage() {
  const { t } = useTranslation();
  useDocumentHead({ title: "Folder — kioku", robots: "noindex,nofollow" });
  const route = useRoute();
  const folderId = route.params.folderId;

  const {
    data: folder,
    error: folderError,
    mutate: refreshFolder,
  } = useFolder(folderId);
  const {
    items,
    error: childrenError,
    isLoading: childrenLoading,
    hasMore,
    loadingMore,
    loadMore,
    refresh: refreshChildren,
  } = useFolderChildren(folderId);

  const [folderDialogOpen, setFolderDialogOpen] = useState(false);
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false);
  const [editFolderOpen, setEditFolderOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<ResourceActionTarget | null>(
    null,
  );
  const [renameTarget, setRenameTarget] = useState<ResourceActionTarget | null>(
    null,
  );
  const [ancestors, setAncestors] = useState<BreadcrumbAncestor[]>([]);

  useEffect(() => {
    if (!folderId) return;
    let cancelled = false;
    fetchFolderAncestors(folderId)
      .then((chain) => {
        if (!cancelled) setAncestors(chain);
      })
      .catch(() => {
        if (!cancelled) setAncestors([]);
      });
    return () => {
      cancelled = true;
    };
  }, [folderId]);

  const breadcrumbs: BreadcrumbItem[] = [
    { label: t("project.breadcrumb.library"), href: "/library" },
    ...ancestors.map((ancestor) => ({
      label: ancestor.name,
      href:
        ancestor.kind === "project"
          ? `/projects/${ancestor.id}`
          : `/folders/${ancestor.id}`,
    })),
    { label: folder?.name ?? (folderError ? "-" : "...") },
  ];

  return (
    <AppLayout>
      <PageHeader
        breadcrumbs={breadcrumbs}
        title={folder?.name ?? t("project.loading")}
        description={folder?.description}
        actions={
          <>
            <button
              type="button"
              onClick={() => setEditFolderOpen(true)}
              disabled={!folder}
              class="btn-secondary"
            >
              <span class="material-symbols-outlined text-[20px]">edit</span>
              {t("folder.editFolder")}
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

        {folderError && (
          <StateMessage tone="danger" className="mb-4">
            {t("folder.errors.load")}
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
          emptyLabel={t("folder.empty")}
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

      {folderId && (
        <>
          <CreateFolderDialog
            open={folderDialogOpen}
            onClose={() => setFolderDialogOpen(false)}
            parentId={folderId}
            parentKind="folder"
            onSuccess={refreshChildren}
          />
          <UploadDialog
            open={uploadDialogOpen}
            onClose={() => setUploadDialogOpen(false)}
            parentId={folderId}
            parentKind="folder"
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
            open={editFolderOpen}
            onClose={() => setEditFolderOpen(false)}
            kind="folder"
            id={folderId}
            initialName={folder?.name ?? ""}
            initialDescription={folder?.description ?? ""}
            onSuccess={refreshFolder}
          />
        </>
      )}
    </AppLayout>
  );
}
