import { useTranslation } from "react-i18next";
import HeaderControls from "../components/HeaderControls";

interface DefinitionItem {
  term: string;
  desc: string;
}

export default function TOSPage() {
  const { t } = useTranslation();

  const introParagraphs = t("tos.sections.intro.paragraphs", {
    returnObjects: true,
  }) as string[];
  const definitions = t("tos.sections.definitions.items", {
    returnObjects: true,
  }) as DefinitionItem[];
  const prohibitedItems = t("tos.sections.service.prohibitedItems", {
    returnObjects: true,
  }) as string[];
  const privacyParagraphs = t("tos.sections.privacy.paragraphs", {
    returnObjects: true,
  }) as string[];
  const ipParagraphs = t("tos.sections.ip.paragraphs", {
    returnObjects: true,
  }) as string[];

  return (
    <div class="bg-background-light dark:bg-background-dark text-charcoal dark:text-charcoal dark:text-text-primary min-h-screen flex flex-col">
      <header class="sticky top-0 z-50 border-b border-border-light dark:border-border-dark bg-background-light/80 dark:bg-background-dark/80">
        <div class="max-w-[1200px] mx-auto px-6 md:px-8 py-4 flex items-center justify-between">
          <a href="/" class="no-underline text-inherit">
            <span class="text-xl font-bold tracking-tight">kioku</span>
          </a>
          <HeaderControls />
        </div>
      </header>

      <div class="flex-1 flex w-full">
        <main class="flex-1 w-full max-w-3xl px-6 py-16 mx-auto">
          <div class="mb-16">
<h1 class="text-[54px] leading-[1.04] font-bold text-charcoal dark:text-text-primary mb-2">
              {t("tos.title")}
            </h1>
          </div>

          <div class="space-y-8 text-taupe dark:text-text-secondary text-base">
            <section id="section-1" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-charcoal dark:text-text-primary mb-4 border-b border-border-light dark:border-border-subtle pb-2">
                {t("tos.sections.intro.title")}
              </h2>
              {introParagraphs.map((p, i) => (
                <p key={i} class={i < introParagraphs.length - 1 ? "mb-4" : ""}>
                  {p}
                </p>
              ))}
            </section>

            <section id="section-2" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-charcoal dark:text-text-primary mb-4 border-b border-border-light dark:border-border-subtle pb-2">
                {t("tos.sections.definitions.title")}
              </h2>
              <p class="mb-4">{t("tos.sections.definitions.lead")}</p>
              <ul class="list-disc pl-6 space-y-2">
                {definitions.map((it, i) => (
                  <li key={i}>
                    <strong>{it.term}</strong>
                    {it.desc}
                  </li>
                ))}
              </ul>
            </section>

            <section id="section-3" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-charcoal dark:text-text-primary mb-4 border-b border-border-light dark:border-border-subtle pb-2">
                {t("tos.sections.service.title")}
              </h2>
              <p class="mb-4">{t("tos.sections.service.intro")}</p>
              <p class="mb-2 text-charcoal dark:text-text-primary font-medium">
                {t("tos.sections.service.prohibitedHeading")}
              </p>
              <p class="mb-4">{t("tos.sections.service.prohibitedLead")}</p>
              <ul class="list-disc pl-6 space-y-2">
                {prohibitedItems.map((p, i) => (
                  <li key={i}>{p}</li>
                ))}
              </ul>
            </section>

            <section id="section-4" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-charcoal dark:text-text-primary mb-4 border-b border-border-light dark:border-border-subtle pb-2">
                {t("tos.sections.account.title")}
              </h2>
              <p class="mb-4">{t("tos.sections.account.body")}</p>
              <div class="bg-card-light dark:bg-surface-dark border border-border-light dark:border-border-subtle rounded-lg p-6 mt-4">
                <span class="material-symbols-outlined text-accent-blue mb-2 block">security</span>
                <p class="text-[14px] text-taupe dark:text-text-secondary">{t("tos.sections.account.callout")}</p>
              </div>
            </section>

            <section id="section-5" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-charcoal dark:text-text-primary mb-4 border-b border-border-light dark:border-border-subtle pb-2">
                {t("tos.sections.privacy.title")}
              </h2>
              {privacyParagraphs.map((p, i) => (
                <p key={i} class={i < privacyParagraphs.length - 1 ? "mb-4" : ""}>
                  {p}
                </p>
              ))}
            </section>

            <section id="section-6" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-charcoal dark:text-text-primary mb-4 border-b border-border-light dark:border-border-subtle pb-2">
                {t("tos.sections.ip.title")}
              </h2>
              {ipParagraphs.map((p, i) => (
                <p key={i} class={i < ipParagraphs.length - 1 ? "mb-4" : ""}>
                  {p}
                </p>
              ))}
            </section>
          </div>

        </main>
      </div>
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
