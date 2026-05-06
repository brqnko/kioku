import { useTranslation } from "react-i18next";
import HeaderControls from "../components/HeaderControls";

interface CollectedItem {
  term: string;
  desc: string;
}
interface Card {
  title: string;
  body: string;
}

export default function PrivacyPolicyPage() {
  const { t } = useTranslation();

  const introParagraphs = t("privacy.sections.intro.paragraphs", {
    returnObjects: true,
  }) as string[];
  const collectedItems = t("privacy.sections.collected.items", {
    returnObjects: true,
  }) as CollectedItem[];
  const aiCards = t("privacy.sections.aiCommitment.cards", {
    returnObjects: true,
  }) as Card[];
  const sharingItems = t("privacy.sections.sharing.items", {
    returnObjects: true,
  }) as string[];
  const retentionParagraphs = t("privacy.sections.retention.paragraphs", {
    returnObjects: true,
  }) as string[];
  const rightsCards = t("privacy.sections.rights.cards", {
    returnObjects: true,
  }) as Card[];

  const aiCardIcons = ["shield", "lock"] as const;
  const aiCardIconClasses = ["text-success", "text-accent-blue"] as const;
  const rightsCardIcons = ["download", "delete_forever"] as const;
  const rightsCardIconClasses = ["text-charcoal dark:text-white", "text-danger"] as const;

  return (
    <div class="bg-background-light dark:bg-background-dark min-h-screen flex flex-col text-charcoal dark:text-text-primary antialiased">
      <header class="sticky top-0 z-50 border-b border-border-light dark:border-border-dark bg-background-light/80 dark:bg-background-dark/80">
        <div class="max-w-[1200px] mx-auto px-6 md:px-8 py-4 flex items-center justify-between">
          <a href="/" class="no-underline text-inherit">
            <span class="text-xl font-bold tracking-tight">kioku</span>
          </a>
          <HeaderControls />
        </div>
      </header>

      <main class="flex-grow flex justify-center py-16 px-6">
        <article class="w-full max-w-[800px]">
          <header class="mb-16">
            <h1 class="text-[54px] leading-[1.04] font-bold text-charcoal dark:text-white mb-4 tracking-tight">
              {t("privacy.title")}
            </h1>
            <div class="border-b border-border-light dark:border-border-subtle pb-8" />
          </header>

          <div class="space-y-8 text-base text-charcoal dark:text-text-primary leading-relaxed">
            <section>
              {introParagraphs.map((p, i) => (
                <p key={i} class={i < introParagraphs.length - 1 ? "mb-4" : ""}>
                  {p}
                </p>
              ))}
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-charcoal dark:text-white mb-4">
                {t("privacy.sections.collected.title")}
              </h2>
              <p class="mb-2 text-taupe dark:text-text-secondary">{t("privacy.sections.collected.lead")}</p>
              <ul class="list-disc list-outside ml-6 space-y-2 text-taupe dark:text-text-secondary">
                {collectedItems.map((it, i) => (
                  <li key={i}>
                    <strong class="text-charcoal dark:text-white font-medium">{it.term}</strong>
                    {it.desc}
                  </li>
                ))}
              </ul>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-charcoal dark:text-white mb-4">
                {t("privacy.sections.aiCommitment.title")}
              </h2>
              <p class="mb-2">{t("privacy.sections.aiCommitment.lead")}</p>
              <div class="bg-card-light dark:bg-surface-dark border border-border-light dark:border-border-subtle rounded-lg p-6 mt-4">
                <ul class="space-y-4">
                  {aiCards.map((card, i) => (
                    <li key={i} class="flex items-start gap-4">
                      <span
                        class={`material-symbols-outlined ${aiCardIconClasses[i] ?? "text-charcoal dark:text-white"} mt-0.5`}
                        style="font-variation-settings: 'FILL' 1;"
                      >
                        {aiCardIcons[i] ?? "info"}
                      </span>
                      <div>
                        <strong class="block text-charcoal dark:text-white font-medium mb-1">{card.title}</strong>
                        <p class="text-taupe dark:text-text-secondary text-sm">{card.body}</p>
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-charcoal dark:text-white mb-4">
                {t("privacy.sections.sharing.title")}
              </h2>
              <p class="mb-4 text-taupe dark:text-text-secondary">{t("privacy.sections.sharing.lead")}</p>
              <ul class="list-disc list-outside ml-6 space-y-2 text-taupe dark:text-text-secondary">
                {sharingItems.map((p, i) => (
                  <li key={i}>{p}</li>
                ))}
              </ul>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-charcoal dark:text-white mb-4">
                {t("privacy.sections.retention.title")}
              </h2>
              {retentionParagraphs.map((p, i) => (
                <p
                  key={i}
                  class={`text-taupe dark:text-text-secondary ${i < retentionParagraphs.length - 1 ? "mb-4" : ""}`}
                >
                  {p}
                </p>
              ))}
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-charcoal dark:text-white mb-4">
                {t("privacy.sections.rights.title")}
              </h2>
              <p class="mb-2 text-taupe dark:text-text-secondary">{t("privacy.sections.rights.lead")}</p>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                {rightsCards.map((card, i) => (
                  <div
                    key={i}
                    class="border border-border-light dark:border-border-subtle rounded p-4 bg-card-light dark:bg-background-dark"
                  >
                    <span
                      class={`material-symbols-outlined ${rightsCardIconClasses[i] ?? "text-charcoal dark:text-white"} mb-2 block`}
                    >
                      {rightsCardIcons[i] ?? "info"}
                    </span>
                    <strong class="block text-charcoal dark:text-white font-medium text-sm mb-1">
                      {card.title}
                    </strong>
                    <p class="text-taupe/70 dark:text-text-disabled text-sm">{card.body}</p>
                  </div>
                ))}
              </div>
            </section>

            <section>
              <h2 class="text-[22px] leading-[1.27] font-bold text-charcoal dark:text-white mb-4">
                {t("privacy.sections.contact.title")}
              </h2>
              <p class="mb-4 text-taupe dark:text-text-secondary">{t("privacy.sections.contact.lead")}</p>
              <div class="flex items-center gap-2 text-charcoal dark:text-white">
                <span class="material-symbols-outlined text-taupe dark:text-text-secondary text-[20px]">
                  mail
                </span>
                <span class="text-base">{t("privacy.sections.contact.email")}</span>
              </div>
            </section>
          </div>

        </article>
      </main>
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
