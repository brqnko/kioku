import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useLocation } from "preact-iso";
import { CreateProjectDialog } from "./CreateProjectDialog";

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

  return (
    <nav class="bg-background-dark text-text-primary text-sm h-screen w-64 border-r border-border-subtle fixed left-0 top-0 flex flex-col p-4 z-50">
      <a href="/" class="flex items-center mb-8 px-2 no-underline text-inherit">
        <h2 class="text-2xl font-bold text-text-primary tracking-tight leading-tight">kioku</h2>
      </a>

      <div class="flex flex-col gap-1 flex-grow">
        {navItems.map((item) => {
          const active = path === item.href;
          return (
            <a
              key={item.href}
              href={item.href}
              class={`flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 ease-in-out ${
                active
                  ? "bg-overlay-soft text-text-primary font-medium"
                  : "text-text-secondary hover:text-text-primary hover:bg-overlay-faint"
              }`}
            >
              <span
                class="material-symbols-outlined text-[20px]"
                style={active ? { fontVariationSettings: "'FILL' 1" } : undefined}
              >
                {item.icon}
              </span>
              {t(item.labelKey)}
            </a>
          );
        })}
      </div>

      <a
        href="/"
        class="px-2 mt-4 text-text-secondary hover:text-text-primary text-sm transition-colors no-underline"
      >
        {t("nav.about")}
      </a>
      <a
        href="https://github.com/brqnko/kioku"
        target="_blank"
        rel="noopener noreferrer"
        class="px-2 mt-3 text-text-secondary hover:text-text-primary text-sm transition-colors no-underline"
      >
        GitHub
      </a>
      <div class="px-2 mt-3 flex items-center gap-3 text-sm">
        <a
          href="/privacy"
          class="text-text-secondary hover:text-text-primary transition-colors no-underline"
        >
          {t("footer.privacy")}
        </a>
        <a
          href="/tos"
          class="text-text-secondary hover:text-text-primary transition-colors no-underline"
        >
          {t("footer.terms")}
        </a>
      </div>

      <div class="mt-auto pt-8">
        <button
          type="button"
          onClick={() => setDialogOpen(true)}
          class="w-full bg-overlay-faint hover:bg-overlay-soft text-text-primary font-medium rounded-lg py-2 flex items-center justify-center gap-2 transition-all duration-200 ease-in-out border border-border-subtle cursor-pointer"
        >
          <span class="material-symbols-outlined text-[18px]">add</span>
          {t("nav.newProject")}
        </button>
      </div>
      <CreateProjectDialog
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
      />
    </nav>
  );
}
