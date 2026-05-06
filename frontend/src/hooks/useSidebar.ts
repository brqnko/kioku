import { useState, useEffect } from "preact/hooks";

const STORAGE_KEY = "sidebar-collapsed";
const EXPANDED_WIDTH = "16rem";
const COLLAPSED_WIDTH = "0px";

function readStored(): boolean {
  if (typeof window === "undefined") return false;
  return localStorage.getItem(STORAGE_KEY) === "true";
}

function applyWidth(collapsed: boolean) {
  if (typeof document === "undefined") return;
  document.documentElement.style.setProperty(
    "--sidebar-width",
    collapsed ? COLLAPSED_WIDTH : EXPANDED_WIDTH,
  );
}

// Apply width synchronously on module load to avoid layout flash.
applyWidth(readStored());

export function useSidebar() {
  const [collapsed, setCollapsed] = useState<boolean>(readStored);

  useEffect(() => {
    applyWidth(collapsed);
    localStorage.setItem(STORAGE_KEY, String(collapsed));
  }, [collapsed]);

  const toggle = () => setCollapsed((c) => !c);

  return { collapsed, toggle } as const;
}
