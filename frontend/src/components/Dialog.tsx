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
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  useEffect(() => {
    if (!open) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCloseRef.current();
    };
    document.addEventListener("keydown", handleKey);
    const prevOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.removeEventListener("keydown", handleKey);
      document.body.style.overflow = prevOverflow;
    };
  }, [open]);

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
        class={`w-full ${maxWidth} dialog-panel flex flex-col`}
      >
        {children}
      </div>
    </div>
  );
}
