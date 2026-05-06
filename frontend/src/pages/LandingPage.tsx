import { useTranslation } from "react-i18next";
import DashboardButton from "../components/DashboardButton";
import GoogleSignInButton from "../components/GoogleSignInButton";
import HeaderControls from "../components/HeaderControls";
import { useAuth } from "../hooks/useAuth";

export default function LandingPage() {
  const { t } = useTranslation();
  const { isAuthenticated } = useAuth();

  return (
    <div class="bg-background-light dark:bg-background-dark text-charcoal dark:text-white/95 overflow-x-hidden">
      {/* Header */}
      <header class="sticky top-0 z-50 border-b border-border-light dark:border-border-dark bg-background-light/80 dark:bg-background-dark/80">
        <div class="max-w-[1200px] mx-auto px-6 md:px-8 py-4 flex items-center justify-between">
          <a href="/" class="no-underline text-inherit">
            <span class="text-xl font-bold tracking-tight">kioku</span>
          </a>
          <HeaderControls />
        </div>
      </header>

      <main>
        {/* Hero Split Screen */}
        <section class="border-b border-border-light dark:border-border-dark">
          <div class="max-w-[1200px] mx-auto px-6 min-h-screen flex flex-col md:flex-row">
            {/* Left: Content */}
            <div class="w-full md:w-1/2 min-w-0 flex flex-col justify-center py-16 md:py-0 md:pr-8">
              <h1 class="text-[54px] leading-[1.04] font-bold text-charcoal dark:text-white mb-4 break-keep">
                {t("hero.title")}
              </h1>
              <p class="text-base text-taupe dark:text-text-muted-dark leading-relaxed mb-8">
                {t("hero.subtitle")}
              </p>
              {isAuthenticated ? (
                <DashboardButton align="start" />
              ) : (
                <GoogleSignInButton align="start" />
              )}
            </div>

            {/* Right: UI Mockup */}
            <div class="w-full md:w-1/2 min-w-0 flex items-center justify-center py-6 md:py-8 relative overflow-hidden">
              <div class="w-full max-w-md aspect-[4/3] bg-card-light dark:bg-card-dark border border-border-light dark:border-border-dark rounded-lg shadow-2xl p-4 flex flex-col gap-2">
                <div class="flex items-center justify-between border-b border-border-light dark:border-white/5 pb-2">
                  <div class="flex gap-1">
                    <div class="w-2 h-2 rounded-full bg-black/15 dark:bg-white/20" />
                    <div class="w-2 h-2 rounded-full bg-black/15 dark:bg-white/20" />
                    <div class="w-2 h-2 rounded-full bg-black/15 dark:bg-white/20" />
                  </div>
                  <div class="h-3 w-24 bg-black/5 dark:bg-white/5 rounded-full" />
                </div>
                <div class="flex-1 flex gap-2">
                  <div class="w-1/3 bg-black/5 dark:bg-white/5 rounded-lg border border-border-light dark:border-white/5" />
                  <div class="flex-1 flex flex-col gap-2">
                    <div class="h-4 bg-black/10 dark:bg-white/10 rounded-full w-3/4" />
                    <div class="h-3 bg-black/5 dark:bg-white/5 rounded-full w-full" />
                    <div class="h-3 bg-black/5 dark:bg-white/5 rounded-full w-full" />
                    <div class="h-3 bg-black/5 dark:bg-white/5 rounded-full w-1/2" />
                    <div class="mt-auto h-24 bg-black/5 dark:bg-white/5 rounded-lg border border-border-light dark:border-white/5" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Features — alternating left/right */}
        <section class="max-w-[1200px] mx-auto px-6 py-16 space-y-16">
          <div class="max-w-2xl">
            <h2 class="text-[54px] leading-[1.04] font-bold text-charcoal dark:text-white mb-4">
              {t("features.synthesis.title")}
            </h2>
            <p class="text-base text-taupe dark:text-text-muted-dark leading-relaxed">
              {t("features.synthesis.body")}
            </p>
          </div>

          <div class="max-w-2xl ml-auto text-right">
            <h2 class="text-[54px] leading-[1.04] font-bold text-charcoal dark:text-white mb-4">
              {t("features.library.title")}
            </h2>
            <p class="text-base text-taupe dark:text-text-muted-dark leading-relaxed">
              {t("features.library.body")}
            </p>
          </div>

          <div class="max-w-2xl">
            <h2 class="text-[54px] leading-[1.04] font-bold text-charcoal dark:text-white mb-4">
              {t("features.research.title")}
            </h2>
            <p class="text-base text-taupe dark:text-text-muted-dark leading-relaxed">
              {t("features.research.body")}
            </p>
          </div>

          <div class="max-w-2xl ml-auto text-right">
            <h2 class="text-[54px] leading-[1.04] font-bold text-charcoal dark:text-white mb-4">
              {t("features.collaborate.title")}
            </h2>
            <p class="text-base text-taupe dark:text-text-muted-dark leading-relaxed">
              {t("features.collaborate.body")}
            </p>
          </div>
        </section>

        {/* CTA Section */}
        <section class="py-16 px-6 border-t border-border-light dark:border-border-dark text-center">
          <div class="max-w-xl mx-auto">
            <h2 class="text-[54px] leading-[1.04] font-bold text-charcoal dark:text-white mb-4">
              {t("cta.title")}
            </h2>
            <p class="text-base text-taupe dark:text-text-muted-dark leading-relaxed mb-8">
              {t("cta.body")}
            </p>
            <div class="flex justify-center">
              {isAuthenticated ? (
                <DashboardButton align="center" />
              ) : (
                <GoogleSignInButton align="center" />
              )}
            </div>
          </div>
        </section>
      </main>

      {/* Footer */}
      <footer class="py-8 border-t border-border-light dark:border-white/5">
        <div class="max-w-[1200px] mx-auto px-6 flex flex-col md:flex-row justify-between items-center gap-4">
          <span class="text-xl font-black text-charcoal dark:text-white">kioku</span>
          <div class="flex gap-6 text-sm text-taupe dark:text-white/50">
            <a class="hover:text-charcoal dark:hover:text-white transition-colors" href="/privacy">
              {t("footer.privacy")}
            </a>
            <a class="hover:text-charcoal dark:hover:text-white transition-colors" href="/tos">
              {t("footer.terms")}
            </a>
          </div>
          <p class="text-sm text-taupe/70 dark:text-white/30">{t("footer.copyright")}</p>
        </div>
      </footer>
    </div>
  );
}
