import { useLocation } from "preact-iso";
import { useTranslation } from "react-i18next";
import { RowActionMenu } from "./RowActionMenu";
import { StateMessage } from "./StateMessage";
import { fileMeta, folderTone, formatDate, formatSize } from "../utils/file";

export interface ResourceTableItem {
  kind: "file" | "folder";
  id: string;
  name: string;
  description?: string | null;
  changed_at: string;
  file_size?: number | null;
}

export interface ResourceActionTarget {
  kind: "file" | "folder";
  id: string;
  name: string;
  description?: string | null;
}

interface ResourceTableProps {
  items: ResourceTableItem[];
  loading: boolean;
  emptyLabel: string;
  onEdit: (target: ResourceActionTarget) => void;
  onDelete: (target: ResourceActionTarget) => void;
}

export function ResourceTable({
  items,
  loading,
  emptyLabel,
  onEdit,
  onDelete,
}: ResourceTableProps) {
  const { t, i18n } = useTranslation();
  const { route: navigate } = useLocation();
  const folders = items.filter((item) => item.kind === "folder");
  const files = items.filter((item) => item.kind === "file");
  const orderedItems = [...folders, ...files];

  const openItem = (item: ResourceTableItem) => {
    navigate(
      item.kind === "folder" ? `/folders/${item.id}` : `/files/${item.id}`,
    );
  };

  return (
    <div class="resource-table">
      <table class="w-full text-left border-collapse">
        <thead>
          <tr>
            <th>{t("project.table.name")}</th>
            <th class="hidden tablet:table-cell">{t("project.table.type")}</th>
            <th class="hidden tablet:table-cell">{t("project.table.size")}</th>
            <th>{t("project.table.modified")}</th>
            <th class="text-right">{t("project.table.action")}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-border-subtle">
          {loading && items.length === 0 && (
            <tr>
              <td colSpan={5}>
                <StateMessage className="text-center py-5">
                  {t("project.loading")}
                </StateMessage>
              </td>
            </tr>
          )}

          {!loading && items.length === 0 && (
            <tr>
              <td colSpan={5}>
                <StateMessage className="text-center py-5">
                  {emptyLabel}
                </StateMessage>
              </td>
            </tr>
          )}

          {orderedItems.map((item) => {
            const isFolder = item.kind === "folder";
            const meta = isFolder ? null : fileMeta(item.name);
            const iconClass = isFolder ? folderTone(item.id) : meta!.tone;
            const icon = isFolder ? "folder" : meta!.icon;
            const type = isFolder ? t("project.table.folder") : meta!.type;
            const size = isFolder ? "-" : formatSize(item.file_size ?? 0);

            return (
              <tr
                key={`${item.kind}-${item.id}`}
                class="resource-row"
                onClick={() => openItem(item)}
              >
                <td>
                  <div class="flex items-center gap-3 min-w-0">
                    <span
                      class={`material-symbols-outlined ${iconClass}`}
                      style={
                        isFolder
                          ? { fontVariationSettings: "'FILL' 1" }
                          : undefined
                      }
                    >
                      {icon}
                    </span>
                    <div class="flex flex-col min-w-0">
                      <span class="text-sm font-medium text-text-primary truncate">
                        {item.name}
                      </span>
                      {item.description && (
                        <span class="text-xs text-text-disabled truncate">
                          {item.description}
                        </span>
                      )}
                    </div>
                  </div>
                </td>
                <td class="hidden tablet:table-cell text-text-secondary">
                  {type}
                </td>
                <td class="hidden tablet:table-cell text-text-secondary">
                  {size}
                </td>
                <td class="text-text-secondary">
                  {formatDate(item.changed_at, i18n.language)}
                </td>
                <td class="text-right">
                  <RowActionMenu
                    ariaLabel={t("project.table.more")}
                    onEdit={() => onEdit(item)}
                    onDelete={() => onDelete(item)}
                  />
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
