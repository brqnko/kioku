import { useEffect, useState } from "preact/hooks";

const STORAGE_KEY = "sidebar-collapsed";
const MOBILE_QUERY = "(max-width: 767.98px)";

function readStored(): boolean {
  if (typeof window === "undefined") return false;
  return localStorage.getItem(STORAGE_KEY) === "true";
}

function readIsMobile(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia(MOBILE_QUERY).matches;
}

function syncDom(
  isMobile: boolean,
  dockedCollapsed: boolean,
  overlayOpen: boolean,
) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.setAttribute("data-sidebar-mode", isMobile ? "overlay" : "docked");
  root.setAttribute(
    "data-sidebar-open",
    String(isMobile ? overlayOpen : !dockedCollapsed),
  );
  const width = isMobile || dockedCollapsed ? "0px" : "16rem";
  root.style.setProperty("--sidebar-width", width);
}

type State = {
  dockedCollapsed: boolean;
  overlayOpen: boolean;
  isMobile: boolean;
};

const state: State = {
  dockedCollapsed: readStored(),
  overlayOpen: false,
  isMobile: readIsMobile(),
};

const listeners = new Set<() => void>();

function emit() {
  for (const fn of listeners) fn();
}

function setState(partial: Partial<State>) {
  Object.assign(state, partial);
  syncDom(state.isMobile, state.dockedCollapsed, state.overlayOpen);
  if ("dockedCollapsed" in partial) {
    localStorage.setItem(STORAGE_KEY, String(state.dockedCollapsed));
  }
  emit();
}

// Apply attributes/width synchronously on module load to avoid layout flash.
syncDom(state.isMobile, state.dockedCollapsed, state.overlayOpen);

if (typeof window !== "undefined") {
  const mql = window.matchMedia(MOBILE_QUERY);
  mql.addEventListener("change", (e) => {
    const partial: Partial<State> = { isMobile: e.matches };
    if (!e.matches) partial.overlayOpen = false;
    setState(partial);
  });
}

export function useSidebar() {
  const [, setTick] = useState(0);

  useEffect(() => {
    const fn = () => setTick((t) => t + 1);
    listeners.add(fn);
    return () => {
      listeners.delete(fn);
    };
  }, []);

  const { isMobile, overlayOpen, dockedCollapsed } = state;

  useEffect(() => {
    if (!(isMobile && overlayOpen)) return;
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setState({ overlayOpen: false });
    };
    document.addEventListener("keydown", handleKey);
    return () => {
      document.body.style.overflow = prevOverflow;
      document.removeEventListener("keydown", handleKey);
    };
  }, [isMobile, overlayOpen]);

  const toggle = () => {
    if (state.isMobile) {
      setState({ overlayOpen: !state.overlayOpen });
    } else {
      setState({ dockedCollapsed: !state.dockedCollapsed });
    }
  };

  const close = () => {
    if (state.isMobile) setState({ overlayOpen: false });
  };

  const isOpen = isMobile ? overlayOpen : !dockedCollapsed;
  const collapsed = isMobile ? !overlayOpen : dockedCollapsed;

  return { collapsed, toggle, isMobile, isOpen, close } as const;
}
