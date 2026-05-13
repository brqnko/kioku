import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { CreateProjectDialog } from "./CreateProjectDialog";

interface CreateProjectTileProps {
  labelKey?: string;
}

export function CreateProjectTile({
  labelKey = "nav.newProject",
}: CreateProjectTileProps) {
  const { t } = useTranslation();
  const [dialogOpen, setDialogOpen] = useState(false);

  return (
    <>
      <button
        type="button"
        onClick={() => setDialogOpen(true)}
        class="group flex flex-col items-center justify-center gap-2 min-h-[160px] p-6 rounded-[12px] border border-dashed border-border-subtle bg-transparent hover:bg-overlay-faint hover:border-text-disabled cursor-pointer text-center"
      >
        <div class="w-10 h-10 rounded-full bg-surface-dark flex items-center justify-center">
          <span class="material-symbols-outlined text-text-secondary group-hover:text-text-primary text-[20px]">
            add
          </span>
        </div>
        <span class="text-sm text-text-secondary group-hover:text-text-primary font-medium">
          {t(labelKey)}
        </span>
      </button>
      <CreateProjectDialog
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
      />
    </>
  );
}
