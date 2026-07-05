import { useEffect, useState } from "preact/hooks";

export type NotificationKind = "warning" | "error" | "success";

export interface Notification {
  id: string;
  kind: NotificationKind;
  message: string;
  dedupeKey?: string;
  durationMs?: number;
  isExiting?: boolean;
}

export interface PushInput {
  kind: NotificationKind;
  message: string;
  dedupeKey?: string;
  durationMs?: number;
}

const DEFAULT_DURATION_MS = 5000;
const EXIT_DURATION_MS = 420;

const items = new Map<string, Notification>();
const autoDismissTimers = new Map<string, ReturnType<typeof setTimeout>>();
const removeTimers = new Map<string, ReturnType<typeof setTimeout>>();
const subscribers = new Set<() => void>();
let snapshot: Notification[] = [];

function rebuildSnapshot() {
  snapshot = Array.from(items.values());
}

function notify() {
  rebuildSnapshot();
  for (const sub of subscribers) sub();
}

function clearAutoDismissTimer(id: string) {
  const t = autoDismissTimers.get(id);
  if (t !== undefined) {
    clearTimeout(t);
    autoDismissTimers.delete(id);
  }
}

function clearRemoveTimer(id: string) {
  const t = removeTimers.get(id);
  if (t !== undefined) {
    clearTimeout(t);
    removeTimers.delete(id);
  }
}

function scheduleAutoDismiss(id: string, durationMs: number) {
  if (typeof window === "undefined") return;
  if (durationMs <= 0) return;
  const t = setTimeout(() => {
    autoDismissTimers.delete(id);
    dismissNotification(id);
  }, durationMs);
  autoDismissTimers.set(id, t);
}

function removeNotification(id: string) {
  clearAutoDismissTimer(id);
  clearRemoveTimer(id);
  if (items.delete(id)) notify();
}

function makeId(): string {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

export function pushNotification(input: PushInput): string {
  if (typeof window === "undefined") return "";

  const duration =
    input.durationMs === undefined ? DEFAULT_DURATION_MS : input.durationMs;

  let id: string | undefined;
  if (input.dedupeKey) {
    for (const [existingId, existing] of items) {
      if (existing.dedupeKey === input.dedupeKey) {
        id = existingId;
        break;
      }
    }
  }
  if (!id) id = makeId();
  clearAutoDismissTimer(id);
  clearRemoveTimer(id);

  items.set(id, {
    id,
    kind: input.kind,
    message: input.message,
    dedupeKey: input.dedupeKey,
    durationMs: duration,
    isExiting: false,
  });
  scheduleAutoDismiss(id, duration);
  notify();
  return id;
}

export function dismissNotification(id: string): void {
  clearAutoDismissTimer(id);
  const item = items.get(id);
  if (!item) return;
  if (item.isExiting) return;

  items.set(id, { ...item, isExiting: true });
  notify();

  if (typeof window === "undefined") {
    removeNotification(id);
    return;
  }

  const t = setTimeout(() => {
    removeTimers.delete(id);
    removeNotification(id);
  }, EXIT_DURATION_MS);
  removeTimers.set(id, t);
}

export function useNotifications(): Notification[] {
  const [value, setValue] = useState<Notification[]>(snapshot);
  useEffect(() => {
    const sub = () => setValue(snapshot);
    subscribers.add(sub);
    sub();
    return () => {
      subscribers.delete(sub);
    };
  }, []);
  return value;
}
