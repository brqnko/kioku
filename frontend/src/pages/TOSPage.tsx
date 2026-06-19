import { LegalLayout } from "../components/LegalLayout";
import { LegalArticles } from "../components/LegalArticles";
import { useDocumentHead } from "../hooks/useDocumentHead";

export default function TOSPage() {
  useDocumentHead({
    title: "Terms of Service — kioku",
    description: "Terms of service for kioku.",
    canonical: "/tos",
    robots: "index,follow",
    ogTitle: "Terms of Service — kioku",
    ogDescription: "Terms of service for kioku.",
    ogUrl: "/tos",
  });

  return (
    <LegalLayout>
      <LegalArticles baseKey="tos" />
    </LegalLayout>
  );
}
