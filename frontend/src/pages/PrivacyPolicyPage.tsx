import { LegalLayout } from "../components/LegalLayout";
import { LegalArticles } from "../components/LegalArticles";
import { useDocumentHead } from "../hooks/useDocumentHead";

export default function PrivacyPolicyPage() {
  useDocumentHead({
    title: "Privacy Policy — kioku",
    description: "How kioku handles your personal information.",
    canonical: "/privacy",
    robots: "index,follow",
    ogTitle: "Privacy Policy — kioku",
    ogDescription: "How kioku handles your personal information.",
    ogUrl: "/privacy",
  });

  return (
    <LegalLayout>
      <LegalArticles baseKey="privacy" />
    </LegalLayout>
  );
}
