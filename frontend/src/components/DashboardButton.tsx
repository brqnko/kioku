import { useTranslation } from "react-i18next";

interface DashboardButtonProps {
  align?: "center" | "start";
}

export default function DashboardButton({
  align = "center",
}: DashboardButtonProps) {
  const { t } = useTranslation();

  const alignment =
    align === "center" ? "items-center text-center" : "items-start text-left";

  return (
    <div class={`flex flex-col gap-3 ${alignment}`}>
      <a
        href="/dashboard"
        class="flex items-center justify-center gap-2 rounded-lg bg-cta px-8 py-3 text-base font-medium text-cta-fg hover:bg-cta-hover no-underline"
      >
        {t("auth.toDashboard")}
        <span class="material-symbols-outlined text-[20px]">arrow_forward</span>
      </a>
    </div>
  );
}
