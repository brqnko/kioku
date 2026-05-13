import { useState, useEffect } from "preact/hooks";

const STORAGE_KEY = "sidebar-collapsed";
const EXPANDED_WIDTH = "16rem";
const COLLAPSED_WIDTH = "0px";
const MOBILE_QUERY = "(max-width: 767.98px)";

function readStored(): boolean {
  if (typeof window === "undefined") return false;
  return localStorage.getItem(STORAGE_KEY) === "true";
}

function readIsMobile(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia(MOBILE_QUERY).matches;
}

function syncDom(isMobile: boolean, dockedCollapsed: boolean, overlayOpen: boolean) {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.setAttribute("data-sidebar-mode", isMobile ? "overlay" : "docked");
  root.setAttribute("data-sidebar-open", String(isMobile ? overlayOpen : !dockedCollapsed));
  const width = isMobile || dockedCollapsed ? COLLAPSED_WIDTH : EXPANDED_WIDTH;
  root.style.setProperty("--sidebar-width", width);
}

// Apply attributes/width synchronously on module load to avoid layout flash.
syncDom(readIsMobile(), readStored(), false);

export function useSidebar() {
  const [dockedCollapsed, setDockedCollapsed] = useState<boolean>(readStored);
  const [overlayOpen, setOverlayOpen] = useState<boolean>(false);
  const [isMobile, setIsMobile] = useState<boolean>(readIsMobile);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const mql = window.matchMedia(MOBILE_QUERY);
    const handler = (e: MediaQueryListEvent) => {
      setIsMobile(e.matches);
      if (!e.matches) setOverlayOpen(false);
    };
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  }, []);

  useEffect(() => {
    syncDom(isMobile, dockedCollapsed, overlayOpen);
    localStorage.setItem(STORAGE_KEY, String(dockedCollapsed));
  }, [isMobile, dockedCollapsed, overlayOpen]);

  useEffect(() => {
    if (!(isMobile && overlayOpen)) return;
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOverlayOpen(false);
    };
    document.addEventListener("keydown", handleKey);
    return () => {
      document.body.style.overflow = prevOverflow;
      document.removeEventListener("keydown", handleKey);
    };
  }, [isMobile, overlayOpen]);

  const toggle = () => {
    if (isMobile) {
      setOverlayOpen((o) => !o);
    } else {
      setDockedCollapsed((c) => !c);
    }
  };

  const close = () => {
    if (isMobile) setOverlayOpen(false);
  };

  const isOpen = isMobile ? overlayOpen : !dockedCollapsed;
  // Legacy "collapsed": true when sidebar is visually hidden.
  const collapsed = isMobile ? !overlayOpen : dockedCollapsed;

  return { collapsed, toggle, isMobile, isOpen, close } as const;
}
