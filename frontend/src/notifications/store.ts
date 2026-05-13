import { useEffect, useState } from "preact/hooks";

export type NotificationKind = "warning" | "error" | "success";

export interface Notification {
  id: string;
  kind: NotificationKind;
  message: string;
  dedupeKey?: string;
  durationMs?: number;
}

export interface PushInput {
  kind: NotificationKind;
  message: string;
  dedupeKey?: string;
  durationMs?: number;
}

const DEFAULT_DURATION_MS = 5000;

const items = new Map<string, Notification>();
const timers = new Map<string, ReturnType<typeof setTimeout>>();
const subscribers = new Set<() => void>();
let snapshot: Notification[] = [];

function rebuildSnapshot() {
  snapshot = Array.from(items.values());
}

function notify() {
  rebuildSnapshot();
  for (const sub of subscribers) sub();
}

function clearTimer(id: string) {
  const t = timers.get(id);
  if (t !== undefined) {
    clearTimeout(t);
    timers.delete(id);
  }
}

function scheduleAutoDismiss(id: string, durationMs: number) {
  if (typeof window === "undefined") return;
  if (durationMs <= 0) return;
  const t = setTimeout(() => {
    timers.delete(id);
    if (items.delete(id)) notify();
  }, durationMs);
  timers.set(id, t);
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
  clearTimer(id);

  items.set(id, {
    id,
    kind: input.kind,
    message: input.message,
    dedupeKey: input.dedupeKey,
    durationMs: duration,
  });
  scheduleAutoDismiss(id, duration);
  notify();
  return id;
}

export function dismissNotification(id: string): void {
  clearTimer(id);
  if (items.delete(id)) notify();
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
