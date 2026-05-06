import { useTranslation } from "react-i18next";
import { useSidebar } from "../hooks/useSidebar";

export default function TopAppBar() {
  const { t } = useTranslation();
  const { collapsed, toggle } = useSidebar();

  return (
    <header class="bg-background-dark/80 text-text-primary text-sm w-full h-14 border-b border-border-subtle sticky top-0 z-50 flex items-center justify-between px-4">
      <div class="flex items-center gap-2">
        <button
          type="button"
          onClick={toggle}
          aria-label={t(collapsed ? "nav.expand" : "nav.collapse")}
          aria-expanded={!collapsed}
          title={t(collapsed ? "nav.expand" : "nav.collapse")}
          class="p-1.5 rounded-md hover:bg-overlay-soft text-text-secondary hover:text-text-primary transition-colors duration-200 ease-in-out cursor-pointer flex items-center justify-center bg-transparent border-none"
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
        class="text-text-secondary hover:text-text-primary hover:bg-overlay-soft rounded-md p-1.5 cursor-pointer transition-colors duration-200 ease-in-out flex items-center justify-center"
      >
        <span class="material-symbols-outlined text-[20px]">settings</span>
      </a>
    </header>
  );
}
