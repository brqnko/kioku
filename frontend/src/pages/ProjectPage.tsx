import { useEffect, useState } from "preact/hooks";
import { useLocation, useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import { AppLayout } from "../components/AppLayout";
import { CreateFolderDialog } from "../components/CreateFolderDialog";
import { DeleteItemDialog } from "../components/DeleteItemDialog";
import { InlineFilePreview } from "../components/InlineFilePreview";
import { PageHeader } from "../components/PageHeader";
import {
  ProjectExplorer,
  type ProjectExplorerActionTarget,
  type ProjectExplorerFileItem,
  type ProjectExplorerItem,
  type ProjectExplorerParentKind,
  type ProjectExplorerRefresh,
} from "../components/ProjectExplorer";
import { ProjectPodcastPanel } from "../components/ProjectPodcastPanel";
import { ProjectChatPanel } from "./ProjectChatPage";
import { RenameItemDialog } from "../components/RenameItemDialog";
import { StateMessage } from "../components/StateMessage";
import { UploadDialog } from "../components/UploadDialog";
import { useProject, useProjectChildren } from "../hooks/useProject";
import { useDocumentHead } from "../hooks/useDocumentHead";
import { useMediaQuery } from "../hooks/useMediaQuery";

type UtilityPanel = "chat" | "podcast";
type MobileView = "home" | "preview" | "chat" | "podcast";

interface CreateParentTarget {
  id: string;
  kind: ProjectExplorerParentKind;
  refresh: ProjectExplorerRefresh;
}

interface ProjectUtilityTabsProps {
  active: UtilityPanel;
  onChange: (panel: UtilityPanel) => void;
}

function utilityTabClass(active: boolean) {
  return `min-h-9 rounded-md px-3 text-sm font-medium transition-colors flex items-center justify-center gap-2 ${
    active
      ? "bg-surface-dark text-text-primary shadow-sm"
      : "text-text-secondary hover:text-text-primary hover:bg-overlay-faint"
  }`;
}

function ProjectUtilityTabs({ active, onChange }: ProjectUtilityTabsProps) {
  const { t } = useTranslation();

  return (
    <div
      class="grid grid-cols-2 gap-1 rounded-lg border border-border-subtle bg-surface-container-low p-1"
      role="tablist"
      aria-label={t("project.sections.utility.title")}
    >
      <button
        type="button"
        role="tab"
        aria-selected={active === "chat"}
        onClick={() => onChange("chat")}
        class={utilityTabClass(active === "chat")}
      >
        <span class="material-symbols-outlined text-[18px]">smart_toy</span>
        {t("project.sections.chat.title")}
      </button>
      <button
        type="button"
        role="tab"
        aria-selected={active === "podcast"}
        onClick={() => onChange("podcast")}
        class={utilityTabClass(active === "podcast")}
      >
        <span class="material-symbols-outlined text-[18px]">podcasts</span>
        {t("project.sections.podcasts.title")}
      </button>
    </div>
  );
}

function MobileBackBar({
  label,
  onBack,
}: {
  label: string;
  onBack: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onBack}
      class="mb-3 flex w-fit items-center gap-1 rounded-md px-2 py-1 text-sm font-medium text-text-secondary transition-colors hover:bg-overlay-faint hover:text-text-primary"
    >
      <span class="material-symbols-outlined text-[20px]">arrow_back</span>
      {label}
    </button>
  );
}

export default function ProjectPage() {
  const { t } = useTranslation();
  useDocumentHead({ title: "Project — kioku", robots: "noindex,nofollow" });
  const route = useRoute();
  const { url } = useLocation();
  const projectId = route.params.projectId;

  const {
    data: project,
    error: projectError,
    mutate: refreshProject,
  } = useProject(projectId);
  const projectChildren = useProjectChildren(projectId);

  const [folderDialogOpen, setFolderDialogOpen] = useState(false);
  const [uploadDialogOpen, setUploadDialogOpen] = useState(false);
  const [editProjectOpen, setEditProjectOpen] = useState(false);
  const [utilityPanel, setUtilityPanel] = useState<UtilityPanel>("chat");
  const isMobile = useMediaQuery("(max-width: 1279px)");
  const [mobileView, setMobileView] = useState<MobileView>("home");
  const [selectedFileId, setSelectedFileId] = useState<string | null>(null);
  const [autoExpandFolderIds, setAutoExpandFolderIds] = useState<Set<string>>(
    () => new Set(),
  );
  const [deleteTarget, setDeleteTarget] =
    useState<ProjectExplorerActionTarget | null>(null);
  const [renameTarget, setRenameTarget] =
    useState<ProjectExplorerActionTarget | null>(null);
  const [mutationRefresh, setMutationRefresh] =
    useState<ProjectExplorerRefresh | null>(null);
  const [createParent, setCreateParent] = useState<CreateParentTarget | null>(
    null,
  );

  const title = project?.name ?? t("project.loading");

  useEffect(() => {
    const search = url.includes("?") ? url.slice(url.indexOf("?")) : "";
    const params = new URLSearchParams(search);
    setSelectedFileId(params.get("file"));
    setAutoExpandFolderIds(
      new Set(
        (params.get("folders") ?? "")
          .split(",")
          .map((id) => id.trim())
          .filter(Boolean),
      ),
    );
  }, [projectId, url]);

  const handleSelectFile = (file: ProjectExplorerFileItem) => {
    setSelectedFileId(file.id);
    if (isMobile) setMobileView("preview");
  };

  const openRenameDialog = (
    target: ProjectExplorerActionTarget,
    refreshParent: ProjectExplorerRefresh,
  ) => {
    setRenameTarget(target);
    setMutationRefresh(() => refreshParent);
  };

  const openDeleteDialog = (
    target: ProjectExplorerActionTarget,
    refreshParent: ProjectExplorerRefresh,
  ) => {
    setDeleteTarget(target);
    setMutationRefresh(() => refreshParent);
  };

  const openCreateFolderDialog = (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => {
    setCreateParent({ id: parentId, kind: parentKind, refresh: refreshParent });
    setFolderDialogOpen(true);
  };

  const openUploadDialog = (
    parentId: string,
    parentKind: ProjectExplorerParentKind,
    refreshParent: ProjectExplorerRefresh,
  ) => {
    setCreateParent({ id: parentId, kind: parentKind, refresh: refreshParent });
    setUploadDialogOpen(true);
  };

  const closeCreateFolderDialog = () => {
    setFolderDialogOpen(false);
    setCreateParent(null);
  };

  const closeUploadDialog = () => {
    setUploadDialogOpen(false);
    setCreateParent(null);
  };

  const handleCreateSuccess = async () => {
    await (createParent?.refresh ?? projectChildren.refresh)();
  };

  const handleRenameSuccess = async () => {
    await (mutationRefresh ?? projectChildren.refresh)();
  };

  const handleDeleteSuccess = async () => {
    if (
      deleteTarget?.kind === "folder" ||
      (deleteTarget?.kind === "file" && deleteTarget.id === selectedFileId)
    ) {
      setSelectedFileId(null);
      if (isMobile && mobileView === "preview") setMobileView("home");
    }
    await (mutationRefresh ?? projectChildren.refresh)();
  };

  const explorerErrors = (
    <>
      {projectError && (
        <StateMessage tone="danger" className="mb-4">
          {t("project.errors.load")}
        </StateMessage>
      )}
      {projectChildren.error && (
        <StateMessage tone="danger" className="mb-4">
          {t("project.errors.children")}
        </StateMessage>
      )}
    </>
  );

  const explorerNode = (
    <ProjectExplorer
      parentId={projectId}
      parentKind="project"
      items={projectChildren.items as ProjectExplorerItem[]}
      loading={projectChildren.isLoading}
      emptyLabel={t("project.empty")}
      hasMore={projectChildren.hasMore}
      loadingMore={projectChildren.loadingMore}
      loadMore={projectChildren.loadMore}
      selectedFileId={selectedFileId}
      autoExpandFolderIds={autoExpandFolderIds}
      onSelectFile={handleSelectFile}
      onEdit={openRenameDialog}
      onDelete={openDeleteDialog}
      onCreateFolder={openCreateFolderDialog}
      onCreateFile={openUploadDialog}
      refresh={projectChildren.refresh}
    />
  );

  return (
    <AppLayout className="flex min-h-0 flex-col">
      <PageHeader
        breadcrumbs={[
          { label: t("workspace.title"), href: "/dashboard" },
          { label: project?.name ?? (projectError ? "-" : "...") },
        ]}
        title={title}
        titleAction={
          <button
            type="button"
            onClick={() => setEditProjectOpen(true)}
            disabled={!project}
            class="icon-button shrink-0 !h-8 !w-8"
            aria-label={t("project.editProject")}
            title={t("project.editProject")}
          >
            <span class="material-symbols-outlined text-[18px]">edit</span>
          </button>
        }
        description={project?.description}
      />

      {isMobile ? (
        <div class="flex min-h-0 flex-1 flex-col">
          {mobileView === "home" && (
            <section class="content-section flex min-h-0 flex-1 flex-col">
              {explorerErrors}
              {explorerNode}

              <div class="mt-4 grid grid-cols-2 gap-2">
                <button
                  type="button"
                  onClick={() => setMobileView("chat")}
                  class={`${utilityTabClass(false)} border border-border-subtle bg-surface-container-low`}
                >
                  <span class="material-symbols-outlined text-[18px]">
                    smart_toy
                  </span>
                  {t("project.sections.chat.title")}
                </button>
                <button
                  type="button"
                  onClick={() => setMobileView("podcast")}
                  class={`${utilityTabClass(false)} border border-border-subtle bg-surface-container-low`}
                >
                  <span class="material-symbols-outlined text-[18px]">
                    podcasts
                  </span>
                  {t("project.sections.podcasts.title")}
                </button>
              </div>
            </section>
          )}

          {mobileView === "preview" && (
            <section class="content-section flex min-h-0 flex-1 flex-col">
              <MobileBackBar
                label={t("project.back")}
                onBack={() => setMobileView("home")}
              />
              <InlineFilePreview fileId={selectedFileId} />
            </section>
          )}

          {mobileView === "chat" && (
            <section class="flex min-h-0 flex-1 flex-col">
              <MobileBackBar
                label={t("project.back")}
                onBack={() => setMobileView("home")}
              />
              <div class="min-h-0 flex-1">
                <ProjectChatPanel projectId={projectId} compact />
              </div>
            </section>
          )}

          {mobileView === "podcast" && (
            <section class="flex min-h-0 flex-1 flex-col">
              <MobileBackBar
                label={t("project.back")}
                onBack={() => setMobileView("home")}
              />
              <div class="min-h-0 flex-1">
                <ProjectPodcastPanel projectId={projectId} compact />
              </div>
            </section>
          )}
        </div>
      ) : (
        <div class="grid min-h-0 flex-1 grid-cols-1 items-stretch gap-6 xl:grid-cols-[minmax(0,1fr)_38rem]">
          <section class="content-section flex min-h-0 flex-col">
            {explorerErrors}
            {explorerNode}

            <InlineFilePreview fileId={selectedFileId} />
          </section>

          <aside class="flex min-h-0 min-w-0 flex-col gap-4 xl:h-full xl:overflow-hidden">
            <ProjectUtilityTabs active={utilityPanel} onChange={setUtilityPanel} />

            <div class="min-h-0 xl:flex-1">
              {utilityPanel === "chat" ? (
                <ProjectChatPanel projectId={projectId} compact />
              ) : (
                <ProjectPodcastPanel projectId={projectId} compact />
              )}
            </div>
          </aside>
        </div>
      )}

      {projectId && (
        <>
          <CreateFolderDialog
            open={folderDialogOpen}
            onClose={closeCreateFolderDialog}
            parentId={createParent?.id ?? projectId}
            parentKind={createParent?.kind ?? "project"}
            onSuccess={handleCreateSuccess}
          />
          <UploadDialog
            open={uploadDialogOpen}
            onClose={closeUploadDialog}
            parentId={createParent?.id ?? projectId}
            parentKind={createParent?.kind ?? "project"}
            onSuccess={handleCreateSuccess}
          />
          <DeleteItemDialog
            open={deleteTarget !== null}
            onClose={() => {
              setDeleteTarget(null);
              setMutationRefresh(null);
            }}
            kind={deleteTarget?.kind ?? "file"}
            id={deleteTarget?.id ?? ""}
            name={deleteTarget?.name ?? ""}
            onSuccess={handleDeleteSuccess}
          />
          <RenameItemDialog
            open={renameTarget !== null}
            onClose={() => {
              setRenameTarget(null);
              setMutationRefresh(null);
            }}
            kind={renameTarget?.kind ?? "file"}
            id={renameTarget?.id ?? ""}
            initialName={renameTarget?.name ?? ""}
            initialDescription={renameTarget?.description ?? ""}
            onSuccess={handleRenameSuccess}
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
