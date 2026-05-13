import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import AccountTab from "./profile/AccountTab";
import SecurityTab from "./profile/SecurityTab";

type Tab = "account" | "security";

export default function ProfilePage() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<Tab>("account");

  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-[var(--sidebar-width)] p-8 min-h-[calc(100vh-3.5rem)] transition-[margin-left] duration-200 ease-in-out">
        <div class="max-w-[800px] mx-auto flex flex-col gap-8">
          <nav class="flex items-center gap-1 bg-surface-dark p-1 rounded-full border border-border-dark w-fit">
            <TabButton
              active={tab === "account"}
              onClick={() => setTab("account")}
              label={t("profile.tabs.account")}
            />
            <TabButton
              active={tab === "security"}
              onClick={() => setTab("security")}
              label={t("profile.tabs.security")}
            />
          </nav>

          {tab === "account" && <AccountTab />}
          {tab === "security" && <SecurityTab />}
        </div>
      </main>
    </div>
  );
}

interface TabButtonProps {
  active: boolean;
  onClick: () => void;
  label: string;
}

function TabButton({ active, onClick, label }: TabButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      class={`px-5 py-1.5 rounded-full text-sm font-bold cursor-pointer border-none whitespace-nowrap ${
        active
          ? "bg-cta text-cta-fg"
          : "bg-transparent text-text-muted-dark hover:text-text-primary"
      }`}
    >
      {label}
    </button>
  );
}
