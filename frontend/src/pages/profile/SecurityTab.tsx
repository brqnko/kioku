import { useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import useSWR from "swr";
import { kyInstance } from "../../api/mutator";
import { SESSIONS_KEY } from "../../api/keys";
import { Dialog } from "../../components/Dialog";
import { pushNotification } from "../../notifications/store";
import { formatRelative } from "../../utils/datetime";
import type {
  ListSessions200,
  ListSessions200ItemsItem,
} from "../../api/generated/backend.schemas";

const fetcher = (path: string) => kyInstance.get(path).json<ListSessions200>();

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
      pushNotification({
        kind: "success",
        message: t("notification.actions.sessionRevoked"),
      });
    } catch {
      pushNotification({
        kind: "error",
        message: t("profile.security.errors.revoke"),
      });
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
      pushNotification({
        kind: "error",
        message: t("profile.security.errors.revokeAll"),
      });
      setRevokingAll(false);
      setConfirmAll(false);
    }
  };

  return (
    <>
      <div class="flex flex-col gap-2">
        <h1 class="heading-h2">{t("profile.security.title")}</h1>
      </div>

      <div>
        {isLoading && (
          <div class="divide-y divide-border-subtle">
            {[0, 1, 2].map((i) => (
              <div key={i} class="animate-pulse py-4">
                <div class="flex items-start justify-between gap-4">
                  <div class="min-w-0 flex-1 space-y-2">
                    <div class="h-4 w-2/3 rounded bg-overlay-soft" />
                    <div class="h-3 w-44 rounded bg-overlay-faint" />
                  </div>
                  <div class="h-9 w-20 rounded bg-overlay-faint" />
                </div>
              </div>
            ))}
          </div>
        )}
        {error && (
          <div class="py-4 text-sm text-danger">
            {t("profile.security.error")}
          </div>
        )}
        {data?.items.length === 0 && (
          <div class="py-4 text-sm text-text-muted-dark">
            {t("profile.security.empty")}
          </div>
        )}

        {data && data.items.length > 0 && (
          <div class="divide-y divide-border-subtle">
            {data.items.map((session) => (
              <SessionRow
                key={session.id}
                session={session}
                locale={i18n.language}
                onRevoke={() => setPendingRevoke(session)}
                revoking={revoking === session.id}
              />
            ))}
          </div>
        )}

        {data && data.items.length > 0 && (
          <div class="mt-4 text-right">
            <button
              type="button"
              onClick={() => setConfirmAll(true)}
              disabled={revokingAll}
              class="btn-danger"
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
          <h3 class="heading-h2">
            {t("profile.security.revokeConfirm.title")}
          </h3>
          <p class="text-body text-text-secondary">
            {t("profile.security.revokeConfirm.body")}
          </p>
          {pendingRevoke && (
            <div class="bg-surface-container-high border border-border-subtle rounded-lg p-3 text-sm">
              <div
                class="text-text-primary truncate"
                title={pendingRevoke.user_agent}
              >
                {pendingRevoke.user_agent || t("profile.security.unknownAgent")}
              </div>
              <div class="text-xs text-text-secondary mt-1">
                {pendingRevoke.ip_address}
              </div>
            </div>
          )}
          <div class="flex justify-end gap-3 mt-2">
            <button
              type="button"
              onClick={() => setPendingRevoke(null)}
              disabled={!!revoking}
              class="btn-ghost"
            >
              {t("profile.cancel")}
            </button>
            <button
              type="button"
              onClick={revokeOne}
              disabled={!!revoking}
              class="btn-danger"
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
          <h3 class="heading-h2 text-danger">
            {t("profile.security.revokeAllConfirm.title")}
          </h3>
          <p class="text-body text-text-secondary">
            {t("profile.security.revokeAllConfirm.body")}
          </p>
          <div class="flex justify-end gap-3 mt-2">
            <button
              type="button"
              onClick={() => setConfirmAll(false)}
              disabled={revokingAll}
              class="btn-ghost"
            >
              {t("profile.cancel")}
            </button>
            <button
              type="button"
              onClick={revokeAll}
              disabled={revokingAll}
              class="btn-danger"
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
  onRevoke: () => void;
  revoking: boolean;
}

function SessionRow({ session, locale, onRevoke, revoking }: SessionRowProps) {
  const { t } = useTranslation();
  const lastUsed = formatRelative(session.last_used_at, locale);

  return (
    <div class="py-4">
      <div class="flex items-start justify-between gap-4">
        <div class="min-w-0">
          <div
            class="truncate text-sm font-bold text-text-primary"
            title={session.user_agent}
          >
            {session.user_agent || t("profile.security.unknownAgent")}
          </div>
          <div class="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-text-muted-dark">
            <span class="truncate">{session.ip_address}</span>
            <span>{lastUsed}</span>
          </div>
        </div>
        <button
          type="button"
          onClick={onRevoke}
          disabled={revoking}
          class="btn-secondary shrink-0"
        >
          {revoking
            ? t("profile.security.revoking")
            : t("profile.security.revoke")}
        </button>
      </div>
    </div>
  );
}
