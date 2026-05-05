import { useTranslation } from "react-i18next";

export default function TopAppBar() {
  const { t } = useTranslation();

  return (
    <header class="bg-background-dark/80 text-text-primary text-sm w-[calc(100%-16rem)] h-14 border-b border-border-subtle sticky top-0 z-40 flex items-center justify-end px-6 ml-64">
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
