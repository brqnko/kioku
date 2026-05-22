import { useTranslation } from "react-i18next";
import useSWR from "swr";
import { kyInstance } from "../../api/mutator";
import { RATE_LIMITS_KEY } from "../../api/keys";
import { formatRelative } from "../../utils/datetime";
import type { GetRateLimits200 } from "../../api/generated/backend.schemas";

const fetcher = (path: string) =>
  kyInstance.get(path).json<GetRateLimits200>();

function nextResetIso(resetAt: string): string {
  const last = new Date(resetAt);
  if (Number.isNaN(last.getTime())) return resetAt;
  const next = new Date(last.getTime() + 24 * 60 * 60 * 1000);
  const now = Date.now();
  while (next.getTime() <= now) {
    next.setUTCDate(next.getUTCDate() + 1);
  }
  return next.toISOString();
}

export default function UsageTab() {
  const { t, i18n } = useTranslation();
  const { data, error, isLoading } = useSWR<GetRateLimits200>(
    RATE_LIMITS_KEY,
    fetcher,
  );

  return (
    <>
      <div class="flex flex-col gap-2">
        <h1 class="heading-h2">{t("profile.usage.title")}</h1>
        <p class="text-body text-text-secondary max-w-2xl">
          {t("profile.usage.subtitle")}
        </p>
      </div>

      {isLoading && (
        <p class="text-sm text-text-muted-dark">{t("profile.loading")}</p>
      )}
      {error && <p class="text-sm text-danger">{t("profile.usage.error")}</p>}

      {data && (
        <div class="flex flex-col gap-4">
          <UsageRow
            label={t("profile.usage.items.podcast")}
            icon="podcasts"
            used={data.podcast.used}
            limit={data.podcast.limit}
            resetAt={data.podcast.reset_at}
            locale={i18n.language}
          />
          <UsageRow
            label={t("profile.usage.items.chatbot")}
            icon="forum"
            used={data.chatbot.used}
            limit={data.chatbot.limit}
            resetAt={data.chatbot.reset_at}
            locale={i18n.language}
          />
          <UsageRow
            label={t("profile.usage.items.fileUpload")}
            icon="upload_file"
            used={data.file_upload.used}
            limit={data.file_upload.limit}
            resetAt={data.file_upload.reset_at}
            locale={i18n.language}
          />
        </div>
      )}
    </>
  );
}

interface UsageRowProps {
  label: string;
  icon: string;
  used: number;
  limit: number;
  resetAt: string;
  locale: string;
}

function UsageRow({
  label,
  icon,
  used,
  limit,
  resetAt,
  locale,
}: UsageRowProps) {
  const { t } = useTranslation();
  const safeUsed = Math.min(used, limit);
  const ratio = limit === 0 ? 0 : safeUsed / limit;
  const percent = Math.round(ratio * 100);
  const exhausted = used >= limit;
  const warning = !exhausted && ratio >= 0.8;
  const barColor = exhausted
    ? "bg-danger"
    : warning
      ? "bg-warning"
      : "bg-accent-blue";
  const remaining = Math.max(limit - used, 0);

  return (
    <div class="bg-surface-dark border border-border-subtle rounded-[12px] p-5 flex flex-col gap-3">
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-3 min-w-0">
          <span class="material-symbols-outlined text-text-muted-dark text-[20px]">
            {icon}
          </span>
          <span class="text-base font-medium text-text-primary truncate">
            {label}
          </span>
        </div>
        <span class="text-sm text-text-muted-dark tabular-nums whitespace-nowrap">
          <span class="text-text-primary font-medium">{used}</span>
          <span> / {limit}</span>
        </span>
      </div>

      <div
        class="h-2 w-full bg-overlay-faint rounded-full overflow-hidden"
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={limit}
        aria-valuenow={safeUsed}
      >
        <div
          class={`h-full ${barColor} rounded-full transition-[width] duration-300`}
          style={{ width: `${percent}%` }}
        />
      </div>

      <div class="flex items-center justify-between text-xs text-text-muted-dark">
        <span>
          {exhausted
            ? t("profile.usage.exhausted")
            : t("profile.usage.remaining", { count: remaining })}
        </span>
        <span>
          {t("profile.usage.resetAt", {
            time: formatRelative(nextResetIso(resetAt), locale),
          })}
        </span>
      </div>
    </div>
  );
}
