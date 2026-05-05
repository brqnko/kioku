import { useEffect, useRef } from "preact/hooks";
import type { ComponentChildren } from "preact";

interface DialogProps {
  open: boolean;
  onClose: () => void;
  ariaLabel: string;
  children: ComponentChildren;
  maxWidth?: string;
}

export function Dialog({
  open,
  onClose,
  ariaLabel,
  children,
  maxWidth = "max-w-[500px]",
}: DialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", handleKey);
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", handleKey);
      document.body.style.overflow = prevOverflow;
    };
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 p-4"
      role="dialog"
      aria-modal="true"
      aria-label={ariaLabel}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        ref={dialogRef}
        class={`w-full ${maxWidth} bg-surface-container border border-border-dark rounded-xl shadow-2xl flex flex-col overflow-hidden`}
      >
        {children}
      </div>
    </div>
  );
}
