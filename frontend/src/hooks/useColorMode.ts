import { useEffect, useState } from "preact/hooks";

export type ColorMode = "light" | "dark" | "system";

const STORAGE_KEY = "color-mode";

function readInitial(): ColorMode {
  if (typeof window === "undefined") return "system";
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  return "system";
}

function getSystemTheme(): "light" | "dark" {
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function applyTheme(mode: ColorMode) {
  if (typeof document === "undefined") return;
  const resolved = mode === "system" ? getSystemTheme() : mode;
  document.documentElement.setAttribute("data-theme", resolved);
}

let currentMode: ColorMode = readInitial();
const subscribers = new Set<() => void>();

if (typeof window !== "undefined") {
  applyTheme(currentMode);
  const mq = window.matchMedia("(prefers-color-scheme: dark)");
  mq.addEventListener("change", () => {
    if (currentMode === "system") applyTheme("system");
  });
}

function setModeInternal(next: ColorMode) {
  currentMode = next;
  if (typeof window !== "undefined") {
    localStorage.setItem(STORAGE_KEY, next);
    applyTheme(next);
  }
  for (const fn of subscribers) fn();
}

export function useColorMode() {
  const [, setTick] = useState(0);

  useEffect(() => {
    const fn = () => setTick((t) => t + 1);
    subscribers.add(fn);
    return () => {
      subscribers.delete(fn);
    };
  }, []);

  return {
    mode: currentMode,
    setMode: setModeInternal,
  } as const;
}
