import { useTranslation } from "react-i18next";
import LanguageSwitcher from "../components/LanguageSwitcher.jsx";

interface DefinitionItem {
  term: string;
  desc: string;
}

export default function TOSPage() {
  const { t } = useTranslation(["tos", "common"]);

  const introParagraphs = t("sections.intro.paragraphs", {
    returnObjects: true,
  }) as string[];
  const definitions = t("sections.definitions.items", {
    returnObjects: true,
  }) as DefinitionItem[];
  const prohibitedItems = t("sections.service.prohibitedItems", {
    returnObjects: true,
  }) as string[];
  const privacyParagraphs = t("sections.privacy.paragraphs", {
    returnObjects: true,
  }) as string[];
  const ipParagraphs = t("sections.ip.paragraphs", {
    returnObjects: true,
  }) as string[];

  return (
    <div class="bg-background-dark text-text-primary min-h-screen flex flex-col">
      <header class="sticky top-0 z-50 bg-background-dark/80 backdrop-blur-md border-b border-border-subtle w-full h-14 flex items-center px-6">
        <div class="flex items-center gap-2">
          <span class="text-[22px] font-bold text-text-primary tracking-tight">kioku</span>
          <span class="text-text-disabled mx-2">/</span>
          <span class="text-[14px] text-text-secondary">{t("common:legal.breadcrumb")}</span>
        </div>
        <div class="ml-auto flex items-center gap-3">
          <LanguageSwitcher />
          <a
            href="/"
            class="flex items-center gap-1 text-[14px] text-text-secondary hover:text-text-primary transition-colors px-4 py-2 rounded hover:bg-surface-dark"
          >
            <span class="material-symbols-outlined text-[18px]">close</span>
            <span>{t("common:legal.close")}</span>
          </a>
        </div>
      </header>

      <div class="flex-1 flex w-full">
        <main class="flex-1 w-full max-w-3xl px-6 py-16 mx-auto">
          <div class="mb-16">
            <div class="flex items-center gap-2 mb-4">
              <span class="material-symbols-outlined text-text-disabled text-[32px]">gavel</span>
            </div>
            <h1 class="text-[54px] leading-[1.04] font-bold text-text-primary mb-2">
              {t("title")}
            </h1>
            <p class="text-[14px] text-text-disabled">{t("lastUpdated")}</p>
          </div>

          <div class="space-y-8 text-text-secondary text-base">
            <section id="section-1" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                {t("sections.intro.title")}
              </h2>
              {introParagraphs.map((p, i) => (
                <p key={i} class={i < introParagraphs.length - 1 ? "mb-4" : ""}>
                  {p}
                </p>
              ))}
            </section>

            <section id="section-2" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                {t("sections.definitions.title")}
              </h2>
              <p class="mb-4">{t("sections.definitions.lead")}</p>
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
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                {t("sections.service.title")}
              </h2>
              <p class="mb-4">{t("sections.service.intro")}</p>
              <p class="mb-2 text-text-primary font-medium">
                {t("sections.service.prohibitedHeading")}
              </p>
              <p class="mb-4">{t("sections.service.prohibitedLead")}</p>
              <ul class="list-disc pl-6 space-y-2">
                {prohibitedItems.map((p, i) => (
                  <li key={i}>{p}</li>
                ))}
              </ul>
            </section>

            <section id="section-4" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                {t("sections.account.title")}
              </h2>
              <p class="mb-4">{t("sections.account.body")}</p>
              <div class="bg-surface-dark border border-border-subtle rounded-lg p-6 mt-4">
                <span class="material-symbols-outlined text-accent-blue mb-2 block">security</span>
                <p class="text-[14px] text-text-secondary">{t("sections.account.callout")}</p>
              </div>
            </section>

            <section id="section-5" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                {t("sections.privacy.title")}
              </h2>
              {privacyParagraphs.map((p, i) => (
                <p key={i} class={i < privacyParagraphs.length - 1 ? "mb-4" : ""}>
                  {p}
                </p>
              ))}
            </section>

            <section id="section-6" class="scroll-mt-32">
              <h2 class="text-[22px] font-bold text-text-primary mb-4 border-b border-border-subtle pb-2">
                {t("sections.ip.title")}
              </h2>
              {ipParagraphs.map((p, i) => (
                <p key={i} class={i < ipParagraphs.length - 1 ? "mb-4" : ""}>
                  {p}
                </p>
              ))}
            </section>
          </div>

          <div class="mt-16 pt-6 border-t border-border-subtle text-center">
            <p class="text-[12px] text-text-disabled">{t("copyright")}</p>
          </div>
        </main>
      </div>
    </div>
  );
}
