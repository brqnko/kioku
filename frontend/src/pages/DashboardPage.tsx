import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { MarkdownView } from "../components/MarkdownView";
import { useDashboard } from "../hooks/useDashboard";
import { useDocumentHead } from "../hooks/useDocumentHead";
import type { GetDashboard200RecentSeenFilesItem } from "../api/generated/backend.schemas";

function fileIcon(name: string): { icon: string; tone: "danger" | "info" } {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  if (ext === "pdf") return { icon: "picture_as_pdf", tone: "danger" };
  return { icon: "description", tone: "info" };
}

const toneClass = {
  danger: "bg-danger/10 text-danger",
  info: "bg-accent-blue/10 text-accent-blue",
} as const;

const RELATIVE_THRESHOLDS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 60 * 60 * 24 * 365],
  ["month", 60 * 60 * 24 * 30],
  ["day", 60 * 60 * 24],
  ["hour", 60 * 60],
  ["minute", 60],
];

function formatRelative(iso: string, locale: string) {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const diffSec = Math.round((then - Date.now()) / 1000);
  const rtf = new Intl.RelativeTimeFormat(locale, { numeric: "auto" });
  for (const [unit, sec] of RELATIVE_THRESHOLDS) {
    if (Math.abs(diffSec) >= sec) {
      return rtf.format(Math.round(diffSec / sec), unit);
    }
  }
  return rtf.format(diffSec, "second");
}

export default function DashboardPage() {
  const { t, i18n } = useTranslation();
  useDocumentHead({ title: "Dashboard — kioku", robots: "noindex,nofollow" });
  const { data, error, isLoading } = useDashboard();

  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main class="ml-[var(--sidebar-width)] p-4 tablet:p-8 h-[calc(100vh-3.5rem)] overflow-y-auto flex flex-col gap-8 transition-[margin-left] duration-200 ease-in-out">
        <header class="flex flex-col gap-1">
          <h1 class="heading-h2">{t("dashboard.title")}</h1>
          <p class="text-caption text-text-secondary">
            {t("dashboard.subtitle")}
          </p>
        </header>

        <div class="grid grid-cols-12 gap-4 auto-rows-min">
          <section class="col-span-12 bg-surface-dark rounded-[12px] border border-border-subtle p-6">
            <div>
              <div class="flex items-center justify-between mb-4">
                <div class="flex items-center gap-2">
                  <div class="w-8 h-8 rounded-lg bg-overlay-faint border border-border-subtle flex items-center justify-center">
                    <span class="material-symbols-outlined text-text-primary text-[18px]">
                      auto_awesome
                    </span>
                  </div>
                  <h2 class="text-base font-bold">{t("dashboard.summary.title")}</h2>
                </div>
                {data?.ai_learning_summary_updated_at && (
                  <span class="text-xs text-text-disabled">
                    {formatRelative(data.ai_learning_summary_updated_at, i18n.language)}
                  </span>
                )}
              </div>

              {isLoading && (
                <p class="text-sm text-text-muted-dark">{t("dashboard.loading")}</p>
              )}
              {error && (
                <p class="text-sm text-danger">{t("dashboard.error")}</p>
              )}
              {data && !data.ai_learning_summary && (
                <p class="text-sm text-text-muted-dark">{t("dashboard.summary.empty")}</p>
              )}
              {data?.ai_learning_summary && (
                <MarkdownView
                  source={data.ai_learning_summary}
                  className="markdown-body text-sm text-text-muted-dark leading-relaxed"
                />
              )}
            </div>
          </section>

          <section class="col-span-12 mt-2">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-base font-bold flex items-center gap-2">
                <span class="material-symbols-outlined text-text-muted-dark text-[18px]">
                  history
                </span>
                {t("dashboard.recent.title")}
              </h2>
              <a
                href="/library"
                class="text-text-muted-dark text-xs hover:text-text-primary flex items-center gap-1 no-underline"
              >
                {t("dashboard.recent.toLibrary")}
                <span class="material-symbols-outlined text-[14px]">arrow_forward</span>
              </a>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
              {isLoading && (
                <p class="col-span-full text-sm text-text-muted-dark">
                  {t("dashboard.loading")}
                </p>
              )}
              {error && (
                <p class="col-span-full text-sm text-danger">
                  {t("dashboard.error")}
                </p>
              )}
              {data?.recent_seen_files?.length === 0 && (
                <p class="col-span-full text-sm text-text-muted-dark">
                  {t("dashboard.recent.empty")}
                </p>
              )}
              {data?.recent_seen_files?.map((file: GetDashboard200RecentSeenFilesItem) => {
                const { icon, tone } = fileIcon(file.name);
                return (
                  <a
                    key={file.id}
                    href={`/files/${file.id}`}
                    class="bg-surface-dark border border-border-subtle rounded-[12px] p-4 flex flex-col gap-4 hover:bg-overlay-faint hover:border-overlay-medium group no-underline text-inherit"
                  >
                    <div class="flex items-start">
                      <div class={`w-10 h-10 rounded-lg flex items-center justify-center ${toneClass[tone]}`}>
                        <span class="material-symbols-outlined text-[20px]">{icon}</span>
                      </div>
                    </div>
                    <div>
                      <h3 class="text-sm font-bold truncate mb-1 group-hover:text-accent-blue">
                        {file.name}
                      </h3>
                      <p class="text-xs text-text-disabled">
                        {formatRelative(file.changed_at, i18n.language)}
                      </p>
                    </div>
                  </a>
                );
              })}
            </div>
          </section>
        </div>
      </main>
    </div>
  );
}
