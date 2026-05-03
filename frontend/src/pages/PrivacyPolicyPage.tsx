import { useTranslation } from "react-i18next";
import LanguageSwitcher from "../components/LanguageSwitcher.jsx";

interface CollectedItem {
  term: string;
  desc: string;
}
interface Card {
  title: string;
  body: string;
}

export default function PrivacyPolicyPage() {
  const { t } = useTranslation(["privacy", "common"]);

  const introParagraphs = t("sections.intro.paragraphs", {
    returnObjects: true,
  }) as string[];
  const collectedItems = t("sections.collected.items", {
    returnObjects: true,
  }) as CollectedItem[];
  const aiCards = t("sections.aiCommitment.cards", {
    returnObjects: true,
  }) as Card[];
  const sharingItems = t("sections.sharing.items", {
    returnObjects: true,
  }) as string[];
  const retentionParagraphs = t("sections.retention.paragraphs", {
    returnObjects: true,
  }) as string[];
  const rightsCards = t("sections.rights.cards", {
    returnObjects: true,
  }) as Card[];

  const aiCardIcons = ["shield", "lock"] as const;
  const aiCardIconClasses = ["text-success", "text-accent-blue"] as const;
  const rightsCardIcons = ["download", "delete_forever"] as const;
  const rightsCardIconClasses = ["text-white", "text-danger"] as const;

  return (
    <div class="bg-background-dark min-h-screen flex flex-col text-text-primary antialiased">
      <header class="sticky top-0 z-40 bg-background-dark/90 backdrop-blur-md h-14 border-b border-border-subtle flex items-center px-6 shrink-0">
        <a
          class="flex items-center gap-2 text-text-secondary hover:text-white transition-colors group cursor-pointer"
          href="/"
        >
          <span class="material-symbols-outlined text-[20px] group-hover:-translate-x-0.5 transition-transform">
            arrow_back
          </span>
          <span class="text-base font-medium">{t("common:legal.backToSettings")}</span>
        </a>
        <div class="ml-auto flex items-center gap-3">
          <LanguageSwitcher />
          <div class="text-xs text-text-secondary hidden sm:block">
            {t("common:legal.documentsLabel")}
          </div>
        </div>
      </header>

      <main class="flex-grow flex justify-center py-16 px-6">
        <article class="w-full max-w-[800px]">
          <header class="mb-16">
            <h1 class="text-[54px] leading-[1.04] font-bold text-white mb-4 tracking-tight">
              {t("title")}
            </h1>
            <div class="flex items-center gap-4 border-b border-border-subtle pb-8">
              <p class="text-base text-text-secondary">{t("effectiveDate")}</p>
              <span class="h-1 w-1 rounded-full bg-text-disabled" />
              <p class="text-base text-text-secondary">{t("lastUpdated")}</p>
            </div>
          </header>

          <div class="space-y-8 text-base text-text-primary leading-relaxed">
            <section>
              {introParagraphs.map((p, i) => (
                <p key={i} class={i < introParagraphs.length - 1 ? "mb-4" : ""}>
                  {p}
                </p>
              ))}
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                {t("sections.collected.title")}
              </h2>
              <p class="mb-2 text-text-secondary">{t("sections.collected.lead")}</p>
              <ul class="list-disc list-outside ml-6 space-y-2 text-text-secondary">
                {collectedItems.map((it, i) => (
                  <li key={i}>
                    <strong class="text-white font-medium">{it.term}</strong>
                    {it.desc}
                  </li>
                ))}
              </ul>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                {t("sections.aiCommitment.title")}
              </h2>
              <p class="mb-2">{t("sections.aiCommitment.lead")}</p>
              <div class="bg-surface-dark border border-border-subtle rounded-lg p-6 mt-4">
                <ul class="space-y-4">
                  {aiCards.map((card, i) => (
                    <li key={i} class="flex items-start gap-4">
                      <span
                        class={`material-symbols-outlined ${aiCardIconClasses[i] ?? "text-white"} mt-0.5`}
                        style="font-variation-settings: 'FILL' 1;"
                      >
                        {aiCardIcons[i] ?? "info"}
                      </span>
                      <div>
                        <strong class="block text-white font-medium mb-1">{card.title}</strong>
                        <p class="text-text-secondary text-sm">{card.body}</p>
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                {t("sections.sharing.title")}
              </h2>
              <p class="mb-4 text-text-secondary">{t("sections.sharing.lead")}</p>
              <ul class="list-disc list-outside ml-6 space-y-2 text-text-secondary">
                {sharingItems.map((p, i) => (
                  <li key={i}>{p}</li>
                ))}
              </ul>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                {t("sections.retention.title")}
              </h2>
              {retentionParagraphs.map((p, i) => (
                <p
                  key={i}
                  class={`text-text-secondary ${i < retentionParagraphs.length - 1 ? "mb-4" : ""}`}
                >
                  {p}
                </p>
              ))}
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                {t("sections.rights.title")}
              </h2>
              <p class="mb-2 text-text-secondary">{t("sections.rights.lead")}</p>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                {rightsCards.map((card, i) => (
                  <div
                    key={i}
                    class="border border-border-subtle rounded p-4 bg-background-dark"
                  >
                    <span
                      class={`material-symbols-outlined ${rightsCardIconClasses[i] ?? "text-white"} mb-2 block`}
                    >
                      {rightsCardIcons[i] ?? "info"}
                    </span>
                    <strong class="block text-white font-medium text-sm mb-1">
                      {card.title}
                    </strong>
                    <p class="text-text-disabled text-sm">{card.body}</p>
                  </div>
                ))}
              </div>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-white mb-4">
                {t("sections.contact.title")}
              </h2>
              <p class="mb-4 text-text-secondary">{t("sections.contact.lead")}</p>
              <div class="flex items-center gap-2 text-white">
                <span class="material-symbols-outlined text-text-secondary text-[20px]">
                  mail
                </span>
                <span class="text-base">{t("sections.contact.email")}</span>
              </div>
            </section>
          </div>

          <div class="mt-16 pt-8 border-t border-border-subtle text-center">
            <p class="text-sm text-text-disabled">{t("copyright")}</p>
          </div>
        </article>
      </main>
    </div>
  );
}
