import { useTranslation } from "react-i18next";
import {
  dismissNotification,
  useNotifications,
  type Notification,
  type NotificationKind,
} from "../notifications/store";
import { Icon } from "./Icon";

const KIND_STYLES: Record<
  NotificationKind,
  { icon: string; accent: string }
> = {
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
    <div class="flex items-start gap-3 bg-surface-dark border border-border-subtle rounded-[12px] px-4 py-3 shadow-[0_4px_12px_rgba(0,0,0,0.15)] animate-fade-in-up">
      <Icon name={icon} class={`shrink-0 mt-0.5 ${accent}`} />
      <p class="flex-1 text-sm leading-6 text-text-primary">
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
      role="region"
      aria-live="polite"
      aria-label={t("notification.region", { defaultValue: "Notifications" })}
      class="fixed top-4 left-1/2 -translate-x-1/2 z-[200] flex flex-col gap-2 w-[calc(100%-32px)] max-w-[480px] pointer-events-none"
    >
      {notifications.map((n) => (
        <div key={n.id} class="pointer-events-auto">
          <NotificationItem
            notification={n}
            dismissLabel={t("notification.dismiss")}
          />
        </div>
      ))}
    </div>
  );
}
