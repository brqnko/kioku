import LightMode from "~icons/material-symbols/light-mode-outline";
import DarkMode from "~icons/material-symbols/dark-mode-outline";
import Computer from "~icons/material-symbols/computer-outline";
import Translate from "~icons/material-symbols/translate";
import Check from "~icons/material-symbols/check";
import Warning from "~icons/material-symbols/warning-outline";
import ErrorOutline from "~icons/material-symbols/error-outline";
import CheckCircle from "~icons/material-symbols/check-circle-outline";
import Close from "~icons/material-symbols/close";

const iconMap: Record<string, string> = {
  light_mode: LightMode,
  dark_mode: DarkMode,
  computer: Computer,
  translate: Translate,
  check: Check,
  warning: Warning,
  error: ErrorOutline,
  check_circle: CheckCircle,
  close: Close,
};

interface IconProps {
  name: string;
  class?: string;
}

export function Icon({ name, class: className }: IconProps) {
  const svg = iconMap[name];
  if (!svg) return null;
  return (
    <span
      class={className}
      style={{ display: "inline-flex", verticalAlign: "middle" }}
      dangerouslySetInnerHTML={{ __html: svg }}
    />
  );
}
