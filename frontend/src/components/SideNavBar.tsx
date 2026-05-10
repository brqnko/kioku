import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useLocation } from "preact-iso";
import { CreateProjectDialog } from "./CreateProjectDialog";
import { REPORT_FORM_URL } from "../constants";
import { useDashboard } from "../hooks/useDashboard";
import { fileMeta } from "../utils/file";

interface NavItem {
  href: string;
  icon: string;
  labelKey: string;
}

const navItems: NavItem[] = [
  { href: "/dashboard", icon: "dashboard", labelKey: "nav.dashboard" },
  { href: "/library", icon: "folder_open", labelKey: "nav.library" },
  { href: "/podcast", icon: "mic", labelKey: "nav.podcast" },
  { href: "/chat", icon: "smart_toy", labelKey: "nav.chat" },
];

export default function SideNavBar() {
  const { t } = useTranslation();
  const { path } = useLocation();
  const [dialogOpen, setDialogOpen] = useState(false);
  const { data: dashboard } = useDashboard();
  const recentFiles = dashboard?.recent_seen_files?.slice(0, 5) ?? [];

  const navItemClass = (active: boolean) =>
    `flex items-center gap-3 px-3 py-2 rounded-lg no-underline cursor-pointer ${
      active
        ? "bg-overlay-soft text-text-primary font-medium"
        : "text-text-secondary hover:text-text-primary hover:bg-overlay-faint"
    }`;

  return (
    <nav
      class="bg-background-dark text-text-primary text-sm h-[calc(100vh-3.5rem)] w-[var(--sidebar-width)] border-r border-border-subtle fixed left-0 top-14 flex flex-col z-40 overflow-hidden transition-[width] duration-200 ease-in-out"
      aria-label={t("nav.sidebar")}
    >
      <div class="flex flex-col overflow-y-auto flex-1 min-w-64">
        <div class="flex flex-col gap-1 p-2 pt-4">
          {navItems.map((item) => {
            const active = path === item.href;
            return (
              <a
                key={item.href}
                href={item.href}
                class={navItemClass(active)}
                aria-current={active ? "page" : undefined}
              >
                <span
                  class="material-symbols-outlined shrink-0 text-[20px]"
                  style={
                    active ? { fontVariationSettings: "'FILL' 1" } : undefined
                  }
                >
                  {item.icon}
                </span>
                <span class="truncate text-sm">{t(item.labelKey)}</span>
              </a>
            );
          })}
        </div>

        {recentFiles.length > 0 && (
          <div class="flex flex-col gap-0.5 p-2 pt-1">
            <p class="text-[11px] text-text-disabled px-3 py-1.5 uppercase tracking-widest font-medium">
              {t("nav.recentFiles")}
            </p>
            {recentFiles.map((file) => {
              const { icon, tone } = fileMeta(file.name);
              return (
                <a
                  key={file.id}
                  href={`/files/${file.id}`}
                  class="flex items-center gap-2.5 px-3 py-1.5 rounded-lg no-underline text-text-secondary hover:text-text-primary hover:bg-overlay-faint cursor-pointer"
                >
                  <span class={`material-symbols-outlined text-[16px] shrink-0 ${tone}`}>
                    {icon}
                  </span>
                  <span class="truncate text-sm">{file.name}</span>
                </a>
              );
            })}
          </div>
        )}
      </div>

      <div class="shrink-0 min-w-64 px-4 pb-5 flex flex-col gap-4">
        <hr class="border-border-subtle" />
        <div class="flex flex-col gap-2">
          <a
            href={REPORT_FORM_URL}
            target="_blank"
            rel="noopener noreferrer"
            class="flex items-center gap-1.5 text-xs text-text-secondary hover:text-text-primary no-underline"
          >
            <span class="material-symbols-outlined text-[14px]">flag</span>
            {t("nav.report")}
          </a>
          <hr class="border-border-subtle my-1" />
          <a
            href="/"
            class="text-xs text-text-secondary hover:text-text-primary no-underline"
          >
            {t("nav.about")}
          </a>
          <a
            href="https://github.com/brqnko/kioku"
            target="_blank"
            rel="noopener noreferrer"
            class="text-xs text-text-secondary hover:text-text-primary no-underline"
          >
            GitHub
          </a>
          <span class="flex gap-3">
            <a
              href="/tos"
              class="text-xs text-text-secondary hover:text-text-primary no-underline"
            >
              {t("footer.terms")}
            </a>
            <a
              href="/privacy"
              class="text-xs text-text-secondary hover:text-text-primary no-underline"
            >
              {t("footer.privacy")}
            </a>
          </span>
        </div>
        <button
          type="button"
          onClick={() => setDialogOpen(true)}
          class="w-full bg-cta text-cta-fg hover:bg-cta-hover font-bold rounded-lg py-2 flex items-center justify-center gap-2 cursor-pointer"
        >
          <span class="material-symbols-outlined text-[18px]">add</span>
          <span class="text-sm">{t("nav.newProject")}</span>
        </button>
      </div>
      <CreateProjectDialog
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
      />
    </nav>
  );
}
