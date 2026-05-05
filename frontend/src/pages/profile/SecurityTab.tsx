import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import useSWR from "swr";
import { kyInstance } from "../../api/mutator";
import { Dialog } from "../../components/Dialog";
import type {
  ListSessions200,
  ListSessions200ItemsItem,
} from "../../api/generated/backend.schemas";

const SESSIONS_KEY = "users/me/sessions?limit=32";

const fetcher = (path: string) => kyInstance.get(path).json<ListSessions200>();

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

export default function SecurityTab() {
  const { t, i18n } = useTranslation();
  const { data, error, isLoading, mutate } = useSWR<ListSessions200>(
    SESSIONS_KEY,
    fetcher,
  );
  const [revoking, setRevoking] = useState<string | null>(null);
  const [revokingAll, setRevokingAll] = useState(false);
  const [pendingRevoke, setPendingRevoke] =
    useState<ListSessions200ItemsItem | null>(null);
  const [confirmAll, setConfirmAll] = useState(false);

  const revokeOne = async () => {
    if (!pendingRevoke) return;
    const id = pendingRevoke.id;
    setRevoking(id);
    try {
      await kyInstance.delete(`users/me/sessions/${id}`);
      await mutate();
      setPendingRevoke(null);
    } finally {
      setRevoking(null);
    }
  };

  const revokeAll = async () => {
    setRevokingAll(true);
    try {
      await kyInstance.delete("users/me/sessions");
      window.location.href = "/";
    } catch {
      setRevokingAll(false);
      setConfirmAll(false);
    }
  };

  return (
    <>
      <div class="flex flex-col gap-2">
        <h1 class="text-[22px] leading-[1.27] font-bold tracking-tight">
          {t("profile.security.title")}
        </h1>
        <p class="text-base text-text-muted-dark max-w-2xl">
          {t("profile.security.subtitle")}
        </p>
      </div>

      <div class="bg-surface-dark border border-border-dark rounded-lg overflow-hidden">
        <div class="grid grid-cols-12 gap-4 p-4 border-b border-border-dark bg-surface-container text-text-muted-dark text-sm">
          <div class="col-span-5 md:col-span-4">
            {t("profile.security.columns.device")}
          </div>
          <div class="col-span-4 md:col-span-3 hidden md:block">
            {t("profile.security.columns.ip")}
          </div>
          <div class="col-span-4 md:col-span-3">
            {t("profile.security.columns.lastAccessed")}
          </div>
          <div class="col-span-3 md:col-span-2 text-right">
            {t("profile.security.columns.action")}
          </div>
        </div>

        {isLoading && (
          <div class="p-4 text-sm text-text-muted-dark">
            {t("profile.loading")}
          </div>
        )}
        {error && (
          <div class="p-4 text-sm text-danger">
            {t("profile.security.error")}
          </div>
        )}
        {data?.items.length === 0 && (
          <div class="p-4 text-sm text-text-muted-dark">
            {t("profile.security.empty")}
          </div>
        )}

        {data?.items.map((session, i) => (
          <SessionRow
            key={session.id}
            session={session}
            locale={i18n.language}
            isLast={i === data.items.length - 1}
            onRevoke={() => setPendingRevoke(session)}
            revoking={revoking === session.id}
          />
        ))}

        {data && data.items.length > 0 && (
          <div class="p-4 bg-surface-container border-t border-border-dark text-right">
            <button
              type="button"
              onClick={() => setConfirmAll(true)}
              disabled={revokingAll}
              class="text-sm text-danger hover:text-danger/80 py-2 px-4 rounded border border-danger/30 hover:border-danger transition-colors cursor-pointer bg-transparent disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {t("profile.security.revokeAll")}
            </button>
          </div>
        )}
      </div>

      <Dialog
        open={!!pendingRevoke}
        onClose={() => {
          if (!revoking) setPendingRevoke(null);
        }}
        ariaLabel={t("profile.security.revokeConfirm.title")}
        maxWidth="max-w-md"
      >
        <div class="p-6 flex flex-col gap-4">
          <h3 class="text-lg font-bold">
            {t("profile.security.revokeConfirm.title")}
          </h3>
          <p class="text-sm text-text-muted-dark leading-relaxed">
            {t("profile.security.revokeConfirm.body")}
          </p>
          {pendingRevoke && (
            <div class="bg-surface-container-high border border-border-dark rounded-lg p-3 text-sm">
              <div class="text-text-primary truncate" title={pendingRevoke.user_agent}>
                {pendingRevoke.user_agent ||
                  t("profile.security.unknownAgent")}
              </div>
              <div class="text-xs text-text-muted-dark mt-1">
                {pendingRevoke.ip_address}
              </div>
            </div>
          )}
          <div class="flex justify-end gap-3 mt-2">
            <button
              type="button"
              onClick={() => setPendingRevoke(null)}
              disabled={!!revoking}
              class="px-5 py-2.5 rounded-lg font-bold text-sm text-text-muted-dark hover:bg-overlay-faint transition-colors cursor-pointer bg-transparent border-none disabled:opacity-50"
            >
              {t("profile.cancel")}
            </button>
            <button
              type="button"
              onClick={revokeOne}
              disabled={!!revoking}
              class="px-5 py-2.5 bg-danger text-white text-sm font-bold rounded-lg hover:bg-danger/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors cursor-pointer border-none"
            >
              {revoking
                ? t("profile.security.revoking")
                : t("profile.security.revoke")}
            </button>
          </div>
        </div>
      </Dialog>

      <Dialog
        open={confirmAll}
        onClose={() => {
          if (!revokingAll) setConfirmAll(false);
        }}
        ariaLabel={t("profile.security.revokeAllConfirm.title")}
        maxWidth="max-w-md"
      >
        <div class="p-6 flex flex-col gap-4">
          <h3 class="text-lg font-bold text-danger">
            {t("profile.security.revokeAllConfirm.title")}
          </h3>
          <p class="text-sm text-text-muted-dark leading-relaxed">
            {t("profile.security.revokeAllConfirm.body")}
          </p>
          <div class="flex justify-end gap-3 mt-2">
            <button
              type="button"
              onClick={() => setConfirmAll(false)}
              disabled={revokingAll}
              class="px-5 py-2.5 rounded-lg font-bold text-sm text-text-muted-dark hover:bg-overlay-faint transition-colors cursor-pointer bg-transparent border-none disabled:opacity-50"
            >
              {t("profile.cancel")}
            </button>
            <button
              type="button"
              onClick={revokeAll}
              disabled={revokingAll}
              class="px-5 py-2.5 bg-danger text-white text-sm font-bold rounded-lg hover:bg-danger/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors cursor-pointer border-none"
            >
              {revokingAll
                ? t("profile.security.revokingAll")
                : t("profile.security.revokeAll")}
            </button>
          </div>
        </div>
      </Dialog>
    </>
  );
}

interface SessionRowProps {
  session: ListSessions200ItemsItem;
  locale: string;
  isLast: boolean;
  onRevoke: () => void;
  revoking: boolean;
}

function SessionRow({
  session,
  locale,
  isLast,
  onRevoke,
  revoking,
}: SessionRowProps) {
  const { t } = useTranslation();
  const lastUsed = formatRelative(session.last_used_at, locale);

  return (
    <div
      class={`grid grid-cols-12 gap-4 p-4 items-center hover:bg-overlay-faint transition-colors ${
        isLast ? "" : "border-b border-border-dark"
      }`}
    >
      <div class="col-span-5 md:col-span-4 min-w-0">
        <div
          class="text-text-primary truncate"
          title={session.user_agent}
        >
          {session.user_agent || t("profile.security.unknownAgent")}
        </div>
        <div class="text-xs text-text-muted-dark mt-1 md:hidden truncate">
          {session.ip_address}
        </div>
      </div>
      <div class="col-span-4 md:col-span-3 hidden md:block text-base text-text-muted-dark truncate">
        {session.ip_address}
      </div>
      <div class="col-span-4 md:col-span-3 text-base text-text-muted-dark">
        {lastUsed}
      </div>
      <div class="col-span-3 md:col-span-2 text-right">
        <button
          type="button"
          onClick={onRevoke}
          disabled={revoking}
          class="text-sm text-text-primary py-2 px-4 rounded border border-border-dark hover:border-text-muted-dark transition-colors cursor-pointer bg-transparent disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {revoking ? t("profile.security.revoking") : t("profile.security.revoke")}
        </button>
      </div>
    </div>
  );
}
