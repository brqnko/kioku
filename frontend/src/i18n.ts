import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import enCommon from "./locales/en/common.json";
import enLanding from "./locales/en/landing.json";
import enTos from "./locales/en/tos.json";
import enPrivacy from "./locales/en/privacy.json";
import jaCommon from "./locales/ja/common.json";
import jaLanding from "./locales/ja/landing.json";
import jaTos from "./locales/ja/tos.json";
import jaPrivacy from "./locales/ja/privacy.json";

export const SUPPORTED_LANGS = ["en", "ja"] as const;
export type SupportedLang = (typeof SUPPORTED_LANGS)[number];

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { common: enCommon, landing: enLanding, tos: enTos, privacy: enPrivacy },
      ja: { common: jaCommon, landing: jaLanding, tos: jaTos, privacy: jaPrivacy },
    },
    fallbackLng: "en",
    supportedLngs: SUPPORTED_LANGS,
    nonExplicitSupportedLngs: true,
    load: "languageOnly",
    ns: ["common", "landing", "tos", "privacy"],
    defaultNS: "common",
    detection: {
      order: ["localStorage", "navigator"],
      caches: ["localStorage"],
      lookupLocalStorage: "i18nextLng",
    },
    interpolation: { escapeValue: false },
    react: { useSuspense: false },
    returnNull: false,
  });

if (typeof document !== "undefined") {
  const apply = (lng: string) => {
    document.documentElement.lang = lng.split("-")[0];
  };
  apply(i18n.resolvedLanguage ?? i18n.language ?? "en");
  i18n.on("languageChanged", apply);
}

export default i18n;
