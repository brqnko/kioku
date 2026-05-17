import { useTranslation } from "react-i18next";
import type { ComponentChildren } from "preact";
import { LegalLayout } from "../components/LegalLayout";
import { useDocumentHead } from "../hooks/useDocumentHead";

interface CollectedItem {
  term: string;
  desc: string;
}
interface Card {
  title: string;
  body: string;
}

const sections = [
  "collected",
  "aiCommitment",
  "sharing",
  "retention",
  "rights",
  "contact",
] as const;

type SectionKey = (typeof sections)[number];

function Section({
  num,
  title,
  children,
}: {
  num: string;
  title: string;
  children: ComponentChildren;
}) {
  return (
    <section class="mb-12 last:mb-0">
      <h4 class="font-bold text-xl mb-6 text-charcoal dark:text-white flex items-center gap-3 m-0">
        <span class="text-primary">{num}.</span> {title}
      </h4>
      <div class="space-y-4">{children}</div>
    </section>
  );
}

export default function PrivacyPolicyPage() {
  const { t } = useTranslation();
  useDocumentHead({
    title: "Privacy Policy — kioku",
    description:
      "How kioku handles your account data and uploaded materials.",
    canonical: "/privacy",
    robots: "index,follow",
    ogTitle: "Privacy Policy — kioku",
    ogDescription:
      "How kioku handles your account data and uploaded materials.",
    ogUrl: "/privacy",
  });

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

  const renderBody = (key: SectionKey) => {
    switch (key) {
      case "collected":
        return (
          <>
            <p class="m-0">{t("privacy.sections.collected.lead")}</p>
            <ul class="list-disc pl-6 space-y-2 m-0">
              {collectedItems.map((it, i) => (
                <li key={i}>
                  <strong class="text-charcoal dark:text-white font-bold">
                    {it.term}
                  </strong>
                  {it.desc}
                </li>
              ))}
            </ul>
          </>
        );
      case "aiCommitment":
        return (
          <>
            <p class="m-0">{t("privacy.sections.aiCommitment.lead")}</p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-2">
              {aiCards.map((card, i) => (
                <div
                  key={i}
                  class="border-l-4 border-primary bg-primary/5 dark:bg-primary/10 rounded-r-lg p-6"
                >
                  <strong class="block text-charcoal dark:text-white font-bold mb-2">
                    {card.title}
                  </strong>
                  <p class="m-0 text-base">{card.body}</p>
                </div>
              ))}
            </div>
          </>
        );
      case "sharing":
        return (
          <>
            <p class="m-0">{t("privacy.sections.sharing.lead")}</p>
            <ul class="list-disc pl-6 space-y-2 m-0">
              {sharingItems.map((p, i) => (
                <li key={i}>{p}</li>
              ))}
            </ul>
          </>
        );
      case "retention":
        return retentionParagraphs.map((p, i) => <p key={i} class="m-0">{p}</p>);
      case "rights":
        return (
          <>
            <p class="m-0">{t("privacy.sections.rights.lead")}</p>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-2">
              {rightsCards.map((card, i) => (
                <div
                  key={i}
                  class="border border-border-light dark:border-border-dark rounded-lg p-5 bg-background-light dark:bg-background-dark"
                >
                  <strong class="block text-charcoal dark:text-white font-bold mb-2">
                    {card.title}
                  </strong>
                  <p class="m-0 text-base">{card.body}</p>
                </div>
              ))}
            </div>
          </>
        );
      case "contact":
        return (
          <>
            <p class="m-0">{t("privacy.sections.contact.lead")}</p>
            <p class="m-0">
              <a
                href={`mailto:${t("privacy.sections.contact.email")}`}
                class="text-primary underline font-medium"
              >
                {t("privacy.sections.contact.email")}
              </a>
            </p>
          </>
        );
    }
  };

  return (
    <LegalLayout>
      <div class="mb-16 text-center">
        <h1 class="text-4xl md:text-6xl font-bold leading-tight tracking-tight mb-6 text-charcoal dark:text-white">
          {t("privacy.title")}
        </h1>
      </div>

      <div class="max-w-4xl mx-auto">
        <div class="bg-card-light dark:bg-card-dark border border-border-light dark:border-border-dark rounded-[12px] overflow-hidden mb-12 shadow-[0_1px_3px_rgba(0,0,0,0.1)]">
          <div class="p-10 md:p-16 text-lg text-taupe dark:text-text-muted-dark leading-8 tracking-[0.01em]">
            {introParagraphs.length > 0 && (
              <section class="mb-12 pb-12 border-b border-border-light dark:border-border-dark space-y-4">
                {introParagraphs.map((p, i) => (
                  <p key={i} class="m-0">
                    {p}
                  </p>
                ))}
              </section>
            )}

            {sections.map((key, idx) => (
              <Section
                key={key}
                num={String(idx + 1).padStart(2, "0")}
                title={t(`privacy.sections.${key}.title`)}
              >
                {renderBody(key)}
              </Section>
            ))}
          </div>
        </div>
      </div>
    </LegalLayout>
  );
}
