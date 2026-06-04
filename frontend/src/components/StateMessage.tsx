import type { ComponentChildren } from "preact";

interface StateMessageProps {
  children: ComponentChildren;
  tone?: "muted" | "danger";
  className?: string;
}

export function StateMessage({
  children,
  tone = "muted",
  className = "",
}: StateMessageProps) {
  const toneClass =
    tone === "danger" ? "text-danger" : "text-text-secondary";
  return <p class={`text-sm ${toneClass} ${className}`}>{children}</p>;
}
