import { useTranslation } from "react-i18next";
import { useSidebar } from "../hooks/useSidebar";

export default function TopAppBar() {
  const { t } = useTranslation();
  const { collapsed, toggle, isMobile, isOpen } = useSidebar();

  const toggleLabel = isMobile
    ? t(isOpen ? "nav.closeMenu" : "nav.openMenu")
    : t(collapsed ? "nav.expand" : "nav.collapse");

  return (
    <header class="bg-background-dark/80 text-text-primary text-sm w-full h-14 border-b border-border-subtle sticky top-0 z-50 flex items-center justify-between px-4">
      <div class="flex items-center gap-2">
        <button
          type="button"
          onClick={toggle}
          aria-label={toggleLabel}
          aria-expanded={isMobile ? isOpen : !collapsed}
          title={toggleLabel}
          class="icon-button"
        >
          <span class="material-symbols-outlined text-[24px]">menu</span>
        </button>
        <a href="/dashboard" class="no-underline text-inherit">
          <span class="text-xl font-bold tracking-tight whitespace-nowrap">
            kioku
          </span>
        </a>
      </div>
      <a
        href="/profile"
        aria-label={t("topbar.settings")}
        class="icon-button no-underline"
      >
        <span class="material-symbols-outlined text-[20px]">settings</span>
      </a>
    </header>
  );
}
