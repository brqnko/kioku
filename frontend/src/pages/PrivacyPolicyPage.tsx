import { useTranslation } from "react-i18next";
import { LegalLayout } from "../components/LegalLayout";
import { LegalSections, type LegalSection } from "../components/LegalSections";
import { useDocumentHead } from "../hooks/useDocumentHead";

const sections: LegalSection[] = [
  { num: "01", key: "definition" },
  { num: "02", key: "collection", items: true },
  { num: "03", key: "purpose", items: true },
  { num: "04", key: "purposeChange" },
  { num: "05", key: "thirdParty", items: true },
  { num: "06", key: "disclosure" },
  { num: "07", key: "correction" },
  { num: "08", key: "suspensionOfUse" },
  { num: "09", key: "policyChange" },
  {
    num: "10",
    key: "contact",
    link: {
      url: "mailto:contact@brqnko.rs",
      labelKey: "privacy.sections.contact.linkLabel",
    },
  },
];

export default function PrivacyPolicyPage() {
  const { t } = useTranslation();
  const title = `${t("privacy.title")} — kioku`;
  const description = t("privacy.metaDescription");
  useDocumentHead({
    title,
    description,
    canonical: "/privacy",
    robots: "index,follow",
    ogTitle: title,
    ogDescription: description,
    ogUrl: "/privacy",
  });

  return (
    <LegalLayout>
      <LegalSections ns="privacy" sections={sections} />
    </LegalLayout>
  );
}
