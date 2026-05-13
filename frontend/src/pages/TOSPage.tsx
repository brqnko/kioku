import { useTranslation } from "react-i18next";
import type { ComponentChildren } from "preact";
import { LegalLayout } from "../components/LegalLayout";

interface DefinitionItem {
  term: string;
  desc: string;
}

const sections = [
  "intro",
  "definitions",
  "service",
  "account",
  "privacy",
  "ip",
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

  const renderBody = (key: SectionKey) => {
    switch (key) {
      case "intro":
        return introParagraphs.map((p, i) => <p key={i} class="m-0">{p}</p>);
      case "definitions":
        return (
          <>
            <p class="m-0">{t("tos.sections.definitions.lead")}</p>
            <ul class="list-disc pl-6 space-y-2 m-0">
              {definitions.map((it, i) => (
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
      case "service":
        return (
          <>
            <p class="m-0">{t("tos.sections.service.intro")}</p>
            <p class="m-0 text-charcoal dark:text-white font-bold">
              {t("tos.sections.service.prohibitedHeading")}
            </p>
            <p class="m-0">{t("tos.sections.service.prohibitedLead")}</p>
            <ul class="list-disc pl-6 space-y-2 m-0">
              {prohibitedItems.map((p, i) => (
                <li key={i}>{p}</li>
              ))}
            </ul>
          </>
        );
      case "account":
        return (
          <>
            <p class="m-0">{t("tos.sections.account.body")}</p>
            <div class="border-l-4 border-primary bg-primary/5 dark:bg-primary/10 rounded-r-lg p-5 mt-2">
              <p class="m-0 text-base text-charcoal dark:text-white">
                {t("tos.sections.account.callout")}
              </p>
            </div>
          </>
        );
      case "privacy":
        return privacyParagraphs.map((p, i) => <p key={i} class="m-0">{p}</p>);
      case "ip":
        return ipParagraphs.map((p, i) => <p key={i} class="m-0">{p}</p>);
    }
  };

  return (
    <LegalLayout>
      <div class="mb-16 text-center">
        <h1 class="text-4xl md:text-6xl font-bold leading-tight tracking-tight mb-6 text-charcoal dark:text-white">
          {t("tos.title")}
        </h1>
      </div>

      <div class="max-w-4xl mx-auto">
        <div class="bg-card-light dark:bg-card-dark border border-border-light dark:border-border-dark rounded-2xl overflow-hidden mb-12 shadow-[0_2px_8px_rgba(0,0,0,0.2)]">
          <div class="p-10 md:p-16 text-lg text-taupe dark:text-text-muted-dark leading-8 tracking-[0.01em]">
            {sections.map((key, idx) => (
              <Section
                key={key}
                num={String(idx + 1).padStart(2, "0")}
                title={t(`tos.sections.${key}.title`)}
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
