import type { JSX } from "preact";
import { useTranslation } from "react-i18next";
import { SUPPORTED_LANGS, type SupportedLang } from "../i18n";

interface Props {
  class?: string;
}

export default function LanguageSwitcher({ class: className = "" }: Props) {
  const { t, i18n } = useTranslation("common");
  const current = (i18n.resolvedLanguage ?? i18n.language ?? "en").split(
    "-",
  )[0] as SupportedLang;

  const onChange = (e: JSX.TargetedEvent<HTMLSelectElement>) => {
    void i18n.changeLanguage(e.currentTarget.value);
  };

  return (
    <label class={`relative inline-flex items-center ${className}`}>
      <span class="sr-only">{t("language.label")}</span>
      <select
        value={current}
        onChange={onChange}
        aria-label={t("language.label")}
        class="bg-transparent border border-white/15 rounded-md text-sm text-white/80 px-2 py-1 hover:bg-white/10 focus:outline-none focus:ring-1 focus:ring-white/30 appearance-none cursor-pointer pr-6"
      >
        {SUPPORTED_LANGS.map((lng) => (
          <option key={lng} value={lng} class="bg-[#191918] text-white">
            {t(`language.names.${lng}`)}
          </option>
        ))}
      </select>
      <span class="material-symbols-outlined pointer-events-none absolute right-1 text-[16px] text-white/50">
        expand_more
      </span>
    </label>
  );
}
