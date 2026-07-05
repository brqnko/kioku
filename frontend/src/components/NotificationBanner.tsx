import { useTranslation } from "react-i18next";
import {
  dismissNotification,
  useNotifications,
  type Notification,
  type NotificationKind,
} from "../notifications/store";
import { Icon } from "./Icon";

const KIND_STYLES: Record<NotificationKind, { icon: string; accent: string }> =
  {
    warning: { icon: "warning", accent: "text-warning" },
    error: { icon: "error", accent: "text-danger" },
    success: { icon: "check_circle", accent: "text-success" },
  };

interface ItemProps {
  notification: Notification;
  dismissLabel: string;
}

function NotificationItem({ notification, dismissLabel }: ItemProps) {
  const { icon, accent } = KIND_STYLES[notification.kind];
  return (
    <div
      class="flex items-start gap-3 bg-surface-dark border border-border-subtle rounded-[12px] px-4 py-3 shadow-[0_4px_12px_rgba(0,0,0,0.15)]"
      style={{
        animation: notification.isExiting
          ? "notification-out 0.2s ease-in forwards"
          : "notification-in 0.25s ease-out both",
      }}
    >
      <Icon name={icon} class={`shrink-0 mt-0.5 ${accent}`} />
      <p class="flex-1 text-sm font-medium leading-5 text-text-primary">
        {notification.message}
      </p>
      <button
        type="button"
        class="shrink-0 -mr-1 -mt-0.5 p-1 rounded-md text-text-secondary hover:text-text-primary hover:bg-overlay-soft transition-colors"
        aria-label={dismissLabel}
        onClick={() => dismissNotification(notification.id)}
      >
        <Icon name="close" />
      </button>
    </div>
  );
}

export function NotificationBanner() {
  const { t } = useTranslation();
  const notifications = useNotifications();
  if (notifications.length === 0) return null;

  return (
    <div
      role="status"
      aria-live="polite"
      aria-label={t("notification.region", { defaultValue: "Notifications" })}
      class="fixed top-4 left-4 right-4 tablet:left-auto tablet:w-auto tablet:min-w-[18rem] tablet:max-w-sm z-[250] flex flex-col pointer-events-none"
    >
      {notifications.map((n) => (
        <div
          key={n.id}
          style={{
            display: "grid",
            gridTemplateRows: n.isExiting ? "0fr" : "1fr",
            paddingBottom: n.isExiting ? "0" : "0.5rem",
            transition: n.isExiting
              ? "grid-template-rows 0.2s ease-in 0.2s, padding-bottom 0.2s ease-in 0.2s"
              : "none",
            overflow: "hidden",
          }}
        >
          <div class="overflow-hidden pointer-events-auto">
            <NotificationItem
              notification={n}
              dismissLabel={t("notification.dismiss")}
            />
          </div>
        </div>
      ))}
    </div>
  );
}
