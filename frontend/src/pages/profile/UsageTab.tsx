import { useTranslation } from "react-i18next";
import useSWR from "swr";
import { kyInstance } from "../../api/mutator";
import { RATE_LIMITS_KEY } from "../../api/keys";
import { formatDateTime } from "../../utils/datetime";
import type { GetRateLimits200 } from "../../api/generated/backend.schemas";

const fetcher = (path: string) => kyInstance.get(path).json<GetRateLimits200>();

type UsageMetric = {
  key: "podcast" | "chatbot" | "fileUpload";
  label: string;
  used: number;
  limit: number;
  resetAt: string;
};

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
  const metrics: UsageMetric[] = data
    ? [
        {
          key: "podcast",
          label: t("profile.usage.items.podcast"),
          used: data.podcast.used,
          limit: data.podcast.limit,
          resetAt: data.podcast.reset_at,
        },
        {
          key: "chatbot",
          label: t("profile.usage.items.chatbot"),
          used: data.chatbot.used,
          limit: data.chatbot.limit,
          resetAt: data.chatbot.reset_at,
        },
        {
          key: "fileUpload",
          label: t("profile.usage.items.fileUpload"),
          used: data.file_upload.used,
          limit: data.file_upload.limit,
          resetAt: data.file_upload.reset_at,
        },
      ]
    : [];

  return (
    <>
      <div class="flex flex-col gap-2">
        <h1 class="heading-h2">{t("profile.usage.title")}</h1>
      </div>

      {isLoading && <UsageSkeleton />}
      {error && <p class="text-sm text-danger">{t("profile.usage.error")}</p>}

      {data && (
        <section class="divide-y divide-border-subtle">
          {metrics.map((metric) => (
            <UsageRow key={metric.key} item={metric} locale={i18n.language} />
          ))}
        </section>
      )}
    </>
  );
}

function UsageSkeleton() {
  return (
    <div class="divide-y divide-border-subtle">
      {[0, 1, 2].map((i) => (
        <div key={i} class="animate-pulse py-4">
          <div class="flex-1">
            <div class="flex items-start justify-between gap-4">
              <div class="space-y-2">
                <div class="h-4 w-36 rounded bg-overlay-soft" />
                <div class="h-3 w-44 rounded bg-overlay-faint" />
              </div>
              <div class="h-4 w-14 rounded bg-overlay-soft" />
            </div>
            <div class="mt-4 h-1.5 w-full rounded-full bg-overlay-faint" />
          </div>
        </div>
      ))}
    </div>
  );
}

function UsageRow({ item, locale }: { item: UsageMetric; locale: string }) {
  const { t } = useTranslation();
  const { label, used, limit, resetAt } = item;
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

  return (
    <div class="py-4">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <h3 class="truncate text-sm font-bold text-text-primary">{label}</h3>
          <p class="mt-1 text-xs text-text-disabled">
            {t("profile.usage.resetAt", {
              time: formatDateTime(nextResetIso(resetAt), locale),
            })}
          </p>
        </div>
        <p class="shrink-0 text-sm text-text-secondary tabular-nums whitespace-nowrap">
          <span class="font-medium text-text-primary">{used}</span>
          <span> / {limit}</span>
        </p>
      </div>
      <div
        class="mt-4 h-1.5 w-full overflow-hidden rounded-full bg-overlay-faint"
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={limit}
        aria-valuenow={safeUsed}
        aria-label={label}
      >
        <div
          class={`h-full ${barColor} rounded-full transition-[width] duration-300`}
          style={{ width: `${percent}%` }}
        />
      </div>
    </div>
  );
}
