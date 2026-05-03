import { useTranslation } from "react-i18next";
import LanguageSwitcher from "../components/LanguageSwitcher.jsx";

export default function LandingPage() {
  const { t } = useTranslation(["landing", "common"]);

  return (
    <div class="bg-[#191918] text-white/95 overflow-x-hidden">
      {/* Header */}
      <header class="sticky top-0 z-40 flex items-center justify-between px-6 bg-[#191918]/80 backdrop-blur-md w-full h-14 border-b border-white/10 text-sm text-white">
        <div class="flex items-center gap-6">
          <span class="text-xl font-black text-white">kioku</span>
          <nav class="hidden md:flex gap-4">
            <a
              class="text-white hover:bg-white/10 rounded-md px-3 py-1 transition-transform active:scale-95"
              href="#"
            >
              {t("common:nav.product")}
            </a>
            <a
              class="text-white/60 hover:bg-white/10 rounded-md px-3 py-1 transition-transform active:scale-95"
              href="#"
            >
              {t("common:nav.docs")}
            </a>
            <a
              class="text-white/60 hover:bg-white/10 rounded-md px-3 py-1 transition-transform active:scale-95"
              href="#"
            >
              {t("common:nav.blog")}
            </a>
          </nav>
        </div>
        <div class="flex items-center gap-4">
          <LanguageSwitcher />
          <button class="px-3 py-1 text-white/80 hover:bg-white/10 rounded-md transition-transform active:scale-95">
            {t("common:nav.signIn")}
          </button>
          <button class="px-4 py-1.5 bg-white text-[#191918] font-semibold rounded-lg hover:opacity-90 transition-all active:scale-95">
            {t("common:nav.getStarted")}
          </button>
        </div>
      </header>

      <main>
        {/* Hero Split Screen */}
        <section class="min-h-screen flex flex-col md:flex-row border-b border-white/10">
          {/* Left: Content */}
          <div class="w-full md:w-1/2 flex flex-col justify-center px-6 md:px-8 py-16 md:py-0">
            <h1 class="text-[54px] leading-[1.04] font-bold text-white mb-4 max-w-lg">
              {t("hero.title")}
            </h1>
            <p class="text-base text-white/60 leading-relaxed mb-8 max-w-md">
              {t("hero.subtitle")}
            </p>
            <div class="flex flex-col sm:flex-row gap-4">
              <button class="bg-white text-[#191918] font-bold py-3 px-8 rounded-lg hover:opacity-90 transition-all active:scale-95">
                {t("hero.ctaPrimary")}
              </button>
              <button class="bg-transparent border border-white/20 text-white font-bold py-3 px-8 rounded-lg hover:bg-white/5 transition-all active:scale-95">
                {t("hero.ctaSecondary")}
              </button>
            </div>
          </div>

          {/* Right: UI Mockup */}
          <div class="w-full md:w-1/2 flex items-center justify-center p-6 md:p-8 relative overflow-hidden">
            <div class="w-full max-w-md aspect-[4/3] bg-[#191918] border border-white/10 rounded-lg shadow-2xl p-4 flex flex-col gap-2">
              <div class="flex items-center justify-between border-b border-white/5 pb-2">
                <div class="flex gap-1">
                  <div class="w-2 h-2 rounded-full bg-white/20" />
                  <div class="w-2 h-2 rounded-full bg-white/20" />
                  <div class="w-2 h-2 rounded-full bg-white/20" />
                </div>
                <div class="h-3 w-24 bg-white/5 rounded-full" />
              </div>
              <div class="flex-1 flex gap-2">
                <div class="w-1/3 bg-white/5 rounded-lg border border-white/5" />
                <div class="flex-1 flex flex-col gap-2">
                  <div class="h-4 bg-white/10 rounded-full w-3/4" />
                  <div class="h-3 bg-white/5 rounded-full w-full" />
                  <div class="h-3 bg-white/5 rounded-full w-full" />
                  <div class="h-3 bg-white/5 rounded-full w-1/2" />
                  <div class="mt-auto h-24 bg-white/5 rounded-lg border border-white/5" />
                </div>
              </div>
            </div>
            {/* Glow decoration */}
            <div class="absolute -bottom-20 -right-20 w-80 h-80 bg-[#2383E2]/10 blur-[120px] rounded-full pointer-events-none" />
          </div>
        </section>

        {/* Features — alternating left/right */}
        <section class="max-w-[1200px] mx-auto px-6 py-16 space-y-16">
          <div class="max-w-2xl">
            <h2 class="text-[54px] leading-[1.04] font-bold text-white mb-4">
              {t("features.synthesis.title")}
            </h2>
            <p class="text-base text-white/60 leading-relaxed">
              {t("features.synthesis.body")}
            </p>
          </div>

          <div class="max-w-2xl ml-auto text-right">
            <h2 class="text-[54px] leading-[1.04] font-bold text-white mb-4">
              {t("features.library.title")}
            </h2>
            <p class="text-base text-white/60 leading-relaxed">
              {t("features.library.body")}
            </p>
          </div>

          <div class="max-w-2xl">
            <h2 class="text-[54px] leading-[1.04] font-bold text-white mb-4">
              {t("features.research.title")}
            </h2>
            <p class="text-base text-white/60 leading-relaxed">
              {t("features.research.body")}
            </p>
          </div>

          <div class="max-w-2xl ml-auto text-right">
            <h2 class="text-[54px] leading-[1.04] font-bold text-white mb-4">
              {t("features.collaborate.title")}
            </h2>
            <p class="text-base text-white/60 leading-relaxed">
              {t("features.collaborate.body")}
            </p>
          </div>
        </section>

        {/* CTA Section */}
        <section class="py-16 px-6 border-t border-white/10 text-center">
          <div class="max-w-xl mx-auto">
            <h2 class="text-[54px] leading-[1.04] font-bold text-white mb-4">
              {t("cta.title")}
            </h2>
            <p class="text-base text-white/60 leading-relaxed mb-8">
              {t("cta.body")}
            </p>
            <button class="bg-white text-[#191918] font-bold py-3 px-16 rounded-lg hover:opacity-90 transition-all active:scale-95">
              {t("cta.button")}
            </button>
          </div>
        </section>
      </main>

      {/* Footer */}
      <footer class="py-8 border-t border-white/5">
        <div class="max-w-[1200px] mx-auto px-6 flex flex-col md:flex-row justify-between items-center gap-4">
          <span class="text-xl font-black text-white">kioku</span>
          <div class="flex gap-6 text-sm text-white/50">
            <a class="hover:text-white transition-colors" href="#">
              {t("common:footer.privacy")}
            </a>
            <a class="hover:text-white transition-colors" href="#">
              {t("common:footer.terms")}
            </a>
            <a class="hover:text-white transition-colors" href="#">
              {t("common:footer.status")}
            </a>
            <a class="hover:text-white transition-colors" href="#">
              {t("common:footer.twitter")}
            </a>
          </div>
          <p class="text-sm text-white/30">{t("common:footer.copyright")}</p>
        </div>
      </footer>
    </div>
  );
}
