import { useTranslation } from "react-i18next";

export default function NotFound() {
  const { t } = useTranslation();
  return (
    <div style={{ textAlign: "center", padding: "4rem" }}>
      <h1>{t("notFound.title")}</h1>
      <p>{t("notFound.message")}</p>
    </div>
  );
}
