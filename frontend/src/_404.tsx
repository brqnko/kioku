import { useTranslation } from "react-i18next";
import { useDocumentHead } from "./hooks/useDocumentHead";

export default function NotFound() {
  const { t } = useTranslation();
  useDocumentHead({
    title: "Page not found — kioku",
    description: "The page you requested does not exist.",
    robots: "noindex,follow",
  });
  return (
    <div style={{ textAlign: "center", padding: "4rem" }}>
      <h1>{t("notFound.title")}</h1>
      <p>{t("notFound.message")}</p>
    </div>
  );
}
