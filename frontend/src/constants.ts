import type { ColorMode } from "./hooks/useColorMode";

export const modeIcons: Record<ColorMode, string> = {
  light: "light_mode",
  dark: "dark_mode",
  system: "computer",
};

export const modeOrder: ColorMode[] = ["light", "dark", "system"];

export const languages = [
  { code: "ja", label: "日本語" },
  { code: "en", label: "English" },
] as const;

export const REPORT_FORM_URL =
  "https://docs.google.com/forms/d/e/1FAIpQLSfbAk6J4jSW83eKe1CeGCCqqHI517STCPh3hE2zRbWedbdcCQ/viewform?usp=dialog";
