import { useEffect, useRef, useState } from "preact/hooks";
import { createPortal } from "preact/compat";
import { useTranslation } from "react-i18next";

interface RowActionMenuProps {
  ariaLabel: string;
  onEdit: () => void;
  onDelete: () => void;
  icon?: string;
}

export function RowActionMenu({
  ariaLabel,
  onEdit,
  onDelete,
  icon = "more_horiz",
}: RowActionMenuProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [menuPos, setMenuPos] = useState({ top: 0, right: 0 });
  const btnRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      const target = e.target as Node;
      if (
        btnRef.current?.contains(target) ||
        menuRef.current?.contains(target)
      ) {
        return;
      }
      setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    const onScroll = () => setOpen(false);
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    window.addEventListener("scroll", onScroll, true);
    window.addEventListener("resize", onScroll);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
      window.removeEventListener("scroll", onScroll, true);
      window.removeEventListener("resize", onScroll);
    };
  }, [open]);

  const toggle = (e: MouseEvent) => {
    e.stopPropagation();
    if (!open && btnRef.current) {
      const rect = btnRef.current.getBoundingClientRect();
      setMenuPos({
        top: rect.bottom + window.scrollY + 4,
        right: window.innerWidth - rect.right - window.scrollX,
      });
    }
    setOpen((v) => !v);
  };

  return (
    <>
      <button
        ref={btnRef}
        type="button"
        aria-label={ariaLabel}
        aria-haspopup="menu"
        aria-expanded={open}
        class="material-symbols-outlined text-text-disabled group-hover:text-text-primary hover:text-text-primary transition-colors cursor-pointer rounded-md p-1 hover:bg-overlay-faint bg-transparent border-none"
        onClick={toggle}
      >
        {icon}
      </button>
      {open &&
        typeof document !== "undefined" &&
        createPortal(
          <div
            ref={menuRef}
            role="menu"
            style={{
              position: "absolute",
              top: menuPos.top,
              right: menuPos.right,
            }}
            class="z-[200] min-w-[160px] bg-surface-container border border-border-dark rounded-lg shadow-lg p-1"
            onClick={(e) => e.stopPropagation()}
          >
            <button
              type="button"
              role="menuitem"
              onClick={(e) => {
                e.stopPropagation();
                setOpen(false);
                onEdit();
              }}
              class="w-full text-left px-3 py-2 text-sm font-medium text-text-primary hover:bg-overlay-faint rounded-md cursor-pointer bg-transparent border-none flex items-center gap-2"
            >
              <span class="material-symbols-outlined text-[18px] text-text-secondary">
                edit
              </span>
              {t("renameItem.menu")}
            </button>
            <button
              type="button"
              role="menuitem"
              onClick={(e) => {
                e.stopPropagation();
                setOpen(false);
                onDelete();
              }}
              class="w-full text-left px-3 py-2 text-sm font-medium text-danger hover:bg-danger/10 rounded-md cursor-pointer bg-transparent border-none flex items-center gap-2"
            >
              <span class="material-symbols-outlined text-[18px]">delete</span>
              {t("deleteItem.menu")}
            </button>
          </div>,
          document.body,
        )}
    </>
  );
}
