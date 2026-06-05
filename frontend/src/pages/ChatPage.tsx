import { useEffect } from "preact/hooks";
import { useLocation } from "preact-iso";
import { AppLayout } from "../components/AppLayout";
import { StateMessage } from "../components/StateMessage";
import { useDocumentHead } from "../hooks/useDocumentHead";

export default function ChatPage() {
  const { route } = useLocation();
  useDocumentHead({ title: "Workspace — kioku", robots: "noindex,nofollow" });

  useEffect(() => {
    route("/dashboard", true);
  }, [route]);

  return (
    <AppLayout>
      <StateMessage>Redirecting...</StateMessage>
    </AppLayout>
  );
}
