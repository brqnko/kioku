import { useTranslation } from "react-i18next";
import { LegalLayout } from "../components/LegalLayout";
import { LegalSections, type LegalSection } from "../components/LegalSections";
import { useDocumentHead } from "../hooks/useDocumentHead";

const sections: LegalSection[] = [
  { num: "01", key: "application" },
  { num: "02", key: "registration" },
  { num: "03", key: "account" },
  { num: "04", key: "prohibited", items: true },
  { num: "05", key: "suspension" },
  { num: "06", key: "restriction" },
  { num: "07", key: "withdrawal" },
  { num: "08", key: "disclaimer" },
  { num: "09", key: "serviceChange" },
  { num: "10", key: "termsChange" },
  { num: "11", key: "personalInfo" },
  { num: "12", key: "notification" },
  { num: "13", key: "assignment" },
  { num: "14", key: "governingLaw" },
];

export default function TOSPage() {
  const { t } = useTranslation();
  const title = `${t("tos.title")} — kioku`;
  const description = t("tos.metaDescription");
  useDocumentHead({
    title,
    description,
    canonical: "/tos",
    robots: "index,follow",
    ogTitle: title,
    ogDescription: description,
    ogUrl: "/tos",
  });

  return (
    <LegalLayout>
      <LegalSections ns="tos" sections={sections} />
    </LegalLayout>
  );
}
