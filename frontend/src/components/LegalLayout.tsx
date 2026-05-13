import { useLocation } from "preact-iso";
import { useTranslation } from "react-i18next";
import type { ComponentChildren } from "preact";
import HeaderControls from "./HeaderControls";

const navLinks = [
  { href: "/tos", labelKey: "legal.navTerms" },
  { href: "/privacy", labelKey: "legal.navPrivacy" },
];

export function LegalLayout({ children }: { children: ComponentChildren }) {
  const { t } = useTranslation();
  const { url } = useLocation();

  return (
    <div class="min-h-dvh bg-background-light dark:bg-background-dark text-charcoal dark:text-white/95">
      <header class="border-b border-border-light dark:border-border-dark bg-background-light/90 dark:bg-background-dark/90 backdrop-blur sticky top-0 z-50">
        <div class="max-w-[1000px] mx-auto px-6 py-4 flex items-center justify-between">
          <a href="/" class="no-underline text-inherit">
            <span class="text-xl font-bold tracking-tight">kioku</span>
          </a>
          <div class="flex items-center gap-6">
            <nav class="hidden sm:flex items-center gap-6">
              {navLinks.map((link) => (
                <a
                  key={link.href}
                  href={link.href}
                  class={`text-sm font-bold no-underline ${
                    url === link.href
                      ? "text-primary"
                      : "text-taupe dark:text-text-muted-dark hover:text-primary"
                  }`}
                >
                  {t(link.labelKey)}
                </a>
              ))}
            </nav>
            <HeaderControls />
          </div>
        </div>
      </header>

      <main class="max-w-[1000px] mx-auto px-6 py-16 md:py-24 text-left">
        {children}

        <footer class="max-w-4xl mx-auto py-16 border-t border-border-light dark:border-border-dark flex flex-col md:flex-row justify-between items-center gap-6 text-sm text-taupe dark:text-text-muted-dark">
          <nav class="flex items-center gap-6">
            {navLinks.map((link) => (
              <a
                key={link.href}
                href={link.href}
                class={`font-medium no-underline ${
                  url === link.href
                    ? "text-primary"
                    : "text-inherit hover:text-primary"
                }`}
              >
                {t(link.labelKey)}
              </a>
            ))}
          </nav>
          <span>{t("footer.copyright")}</span>
        </footer>
      </main>
    </div>
  );
}
