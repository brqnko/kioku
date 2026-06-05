import { useTranslation } from "react-i18next";
import { useLocation } from "preact-iso";

export default function TopAppBar() {
  const { t } = useTranslation();
  const { path } = useLocation();

  return (
    <header class="bg-background-dark/90 backdrop-blur text-text-primary text-sm w-full h-14 border-b border-border-subtle sticky top-0 z-50 px-3 tablet:px-4">
      <div class="flex h-full items-center justify-between gap-3 min-w-0">
        <a
          href="/dashboard"
          class="no-underline text-inherit shrink-0 flex items-center"
        >
          <span class="text-xl font-bold tracking-tight whitespace-nowrap">
            kioku
          </span>
        </a>

        <a
          href="/profile"
          aria-label={t("topbar.settings")}
          aria-current={path === "/profile" ? "page" : undefined}
          class={`icon-button no-underline shrink-0 ${
            path === "/profile" ? "bg-overlay-soft text-text-primary" : ""
          }`}
        >
          <span class="material-symbols-outlined text-[20px]">settings</span>
        </a>
      </div>
    </header>
  );
}
