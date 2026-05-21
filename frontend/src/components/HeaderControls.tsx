import { useState, useRef, useEffect } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { Icon } from "./Icon";
import { GithubIcon } from "./GithubIcon";
import { useColorMode } from "../hooks/useColorMode";
import { useAuth } from "../hooks/useAuth";
import { kyInstance } from "../api/mutator";
import { PROFILE_KEY } from "../api/keys";
import { modeIcons, modeOrder, languages } from "../constants";
import type { UpdateUserProfileBody } from "../api/generated/backend.schemas";

export default function HeaderControls() {
  const { t, i18n } = useTranslation();
  const { mode, setMode } = useColorMode();
  const { isAuthenticated } = useAuth();

  const nextMode = modeOrder[(modeOrder.indexOf(mode) + 1) % modeOrder.length];
  const cycleMode = () => setMode(nextMode);

  const currentLang =
    languages.find((l) => l.code === i18n.language)?.code ?? "en";
  const [langOpen, setLangOpen] = useState(false);
  const langRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (langRef.current && !langRef.current.contains(e.target as Node)) {
        setLangOpen(false);
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") setLangOpen(false);
    };
    document.addEventListener("click", handleClick);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("click", handleClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  return (
    <div class="flex items-center gap-2">
      <button
        type="button"
        class="icon-button"
        onClick={cycleMode}
        title={t(`colorMode.${nextMode}`)}
        aria-label={`${t(`colorMode.${mode}`)} → ${t(`colorMode.${nextMode}`)}`}
      >
        <Icon name={modeIcons[mode]} class="text-xl" />
      </button>
      <div class="relative" ref={langRef}>
        <button
          type="button"
          class="icon-button"
          onClick={() => setLangOpen(!langOpen)}
          aria-label={t("language.label")}
          aria-expanded={langOpen}
          aria-haspopup="true"
        >
          <Icon name="translate" class="text-xl" />
        </button>
        {langOpen && (
          <div
            role="menu"
            class="menu-panel absolute right-0 top-full mt-2 py-2 min-w-[180px] z-50"
          >
            {languages.map((lang) => (
              <button
                role="menuitem"
                type="button"
                key={lang.code}
                class={`w-full flex items-center gap-2 px-4 py-2 text-left text-sm font-medium cursor-pointer bg-transparent border-none ${
                  currentLang === lang.code
                    ? "text-accent-blue"
                    : "text-text-secondary hover:text-text-primary hover:bg-overlay-faint"
                }`}
                onClick={() => {
                  void i18n.changeLanguage(lang.code);
                  localStorage.setItem("lang", lang.code);
                  if (isAuthenticated) {
                    const body: UpdateUserProfileBody = {
                      language_code: lang.code,
                    };
                    void kyInstance
                      .patch(PROFILE_KEY, { json: body })
                      .catch(() => {
                        // ignore — local preference still applied
                      });
                  }
                  setLangOpen(false);
                }}
              >
                <Icon
                  name="check"
                  class={`text-base ${currentLang === lang.code ? "opacity-100" : "opacity-0"}`}
                />
                {lang.label}
              </button>
            ))}
          </div>
        )}
      </div>
      <a
        class="icon-button no-underline"
        href="https://github.com/brqnko/kioku"
        target="_blank"
        rel="noopener noreferrer"
        aria-label="GitHub"
      >
        <GithubIcon />
      </a>
    </div>
  );
}
