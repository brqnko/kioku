import { useEffect, useRef, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import useSWR from "swr";
import { kyInstance } from "../../api/mutator";
import { useColorMode, type ColorMode } from "../../hooks/useColorMode";
import { languages, modeIcons, modeOrder } from "../../constants";
import { Dialog } from "../../components/Dialog";
import type {
  GetUserProfile200,
  UpdateUserProfileBody,
} from "../../api/generated/backend.schemas";

const PROFILE_KEY = "users/me";

const profileFetcher = (path: string) =>
  kyInstance.get(path).json<GetUserProfile200>();

export default function AccountTab() {
  const { t, i18n } = useTranslation();
  const { mode, setMode } = useColorMode();
  const { data, error, isLoading, mutate } = useSWR<GetUserProfile200>(
    PROFILE_KEY,
    profileFetcher,
  );

  const [displayName, setDisplayName] = useState("");
  const [languageCode, setLanguageCode] = useState(i18n.language);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [showLogout, setShowLogout] = useState(false);
  const [showDelete, setShowDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [deleteInput, setDeleteInput] = useState("");
  const [langOpen, setLangOpen] = useState(false);
  const [themeOpen, setThemeOpen] = useState(false);
  const langRef = useRef<HTMLDivElement>(null);
  const themeRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (langRef.current && !langRef.current.contains(e.target as Node)) {
        setLangOpen(false);
      }
      if (themeRef.current && !themeRef.current.contains(e.target as Node)) {
        setThemeOpen(false);
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setLangOpen(false);
        setThemeOpen(false);
      }
    };
    document.addEventListener("click", handleClick);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("click", handleClick);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  const currentLanguage = languages.find((l) => l.code === languageCode);

  useEffect(() => {
    if (data) {
      setDisplayName(data.display_name ?? "");
      setLanguageCode(data.language_code ?? i18n.language);
    }
  }, [data]);

  const trimmed = displayName.trim();
  const nameLength = [...trimmed].length;
  const nameValid = nameLength >= 1 && nameLength <= 32;
  const dirty =
    !!data &&
    (trimmed !== (data.display_name ?? "") ||
      languageCode !== (data.language_code ?? ""));

  const handleSave = async () => {
    if (!nameValid || saving) return;
    setSaving(true);
    setSaveError(null);
    setSaved(false);
    try {
      const body: UpdateUserProfileBody = {
        display_name: trimmed,
        language_code: languageCode,
      };
      const updated = await kyInstance
        .patch(PROFILE_KEY, { json: body })
        .json<GetUserProfile200>();
      await mutate(updated, { revalidate: false });
      void i18n.changeLanguage(languageCode);
      localStorage.setItem("lang", languageCode);
      setSaved(true);
      setTimeout(() => setSaved(false), 2500);
    } catch {
      setSaveError(t("profile.errors.save"));
    } finally {
      setSaving(false);
    }
  };

  const handleLogout = async () => {
    try {
      await kyInstance.post("auth/logout");
    } catch {
      // ignore
    }
    window.location.href = "/";
  };

  const handleDelete = async () => {
    if (deleting) return;
    setDeleting(true);
    try {
      await kyInstance.delete(PROFILE_KEY);
      window.location.href = "/";
    } catch {
      setDeleting(false);
    }
  };

  if (isLoading) {
    return <p class="text-sm text-text-muted-dark">{t("profile.loading")}</p>;
  }
  if (error) {
    return <p class="text-sm text-danger">{t("profile.errors.load")}</p>;
  }

  return (
    <>
      <div class="flex flex-col gap-1">
        <h1 class="heading-h2">{t("profile.account.title")}</h1>
        <p class="text-body text-text-secondary">
          {t("profile.account.subtitle")}
        </p>
      </div>

      <section class="flex flex-col gap-4">
        <Field label={t("profile.account.displayName")}>
          <input
            id="displayName"
            type="text"
            value={displayName}
            maxLength={32}
            onInput={(e) =>
              setDisplayName((e.target as HTMLInputElement).value)
            }
            class="input-field"
          />
          <div class="flex items-center justify-between">
            <p class="text-xs text-text-muted-dark">
              {t("profile.account.displayNameHelp")}
            </p>
            <span
              class={`text-xs tabular-nums ${nameLength > 32 ? "text-danger" : "text-text-disabled"}`}
            >
              {nameLength}/32
            </span>
          </div>
        </Field>

        <Field label={t("profile.account.language")} bordered>
          <div class="relative w-full tablet:w-fit min-w-[200px]" ref={langRef}>
            <button
              type="button"
              onClick={() => setLangOpen(!langOpen)}
              aria-haspopup="true"
              aria-expanded={langOpen}
              class="input-field flex items-center justify-between gap-2 cursor-pointer text-sm"
            >
              <span>{currentLanguage?.label ?? languageCode}</span>
              <span class="material-symbols-outlined text-text-secondary text-[18px]">
                expand_more
              </span>
            </button>
            {langOpen && (
              <div
                role="menu"
                class="menu-panel absolute left-0 right-0 top-full mt-2 py-2 z-50"
              >
                {languages.map((lang) => {
                  const active = languageCode === lang.code;
                  return (
                    <button
                      role="menuitem"
                      type="button"
                      key={lang.code}
                      onClick={() => {
                        setLanguageCode(lang.code);
                        setLangOpen(false);
                      }}
                      class={`w-full flex items-center gap-2 px-4 py-2 text-left text-sm font-medium cursor-pointer bg-transparent border-none ${
                        active
                          ? "text-accent-blue"
                          : "text-text-secondary hover:bg-overlay-faint"
                      }`}
                    >
                      <span
                        class={`material-symbols-outlined text-base ${active ? "opacity-100" : "opacity-0"}`}
                      >
                        check
                      </span>
                      {lang.label}
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        </Field>

        <Field label={t("profile.account.theme")} bordered>
          <div class="relative w-full tablet:w-fit min-w-[200px]" ref={themeRef}>
            <button
              type="button"
              onClick={() => setThemeOpen(!themeOpen)}
              aria-haspopup="true"
              aria-expanded={themeOpen}
              class="input-field flex items-center justify-between gap-2 cursor-pointer text-sm"
            >
              <span class="flex items-center gap-2">
                <span
                  class="material-symbols-outlined"
                  style={{ fontSize: "16px", lineHeight: 1 }}
                >
                  {modeIcons[mode]}
                </span>
                {t(`colorMode.${mode}`)}
              </span>
              <span class="material-symbols-outlined text-text-secondary text-[18px]">
                expand_more
              </span>
            </button>
            {themeOpen && (
              <div
                role="menu"
                class="menu-panel absolute left-0 right-0 top-full mt-2 py-2 z-50"
              >
                {modeOrder.map((m) => {
                  const active = mode === m;
                  return (
                    <button
                      role="menuitem"
                      type="button"
                      key={m}
                      onClick={() => {
                        setMode(m as ColorMode);
                        setThemeOpen(false);
                      }}
                      class={`w-full flex items-center gap-2 px-4 py-2 text-left text-sm font-medium cursor-pointer bg-transparent border-none ${
                        active
                          ? "text-accent-blue"
                          : "text-text-secondary hover:bg-overlay-faint"
                      }`}
                    >
                      <span
                        class={`material-symbols-outlined text-base ${active ? "opacity-100" : "opacity-0"}`}
                      >
                        check
                      </span>
                      <span class="material-symbols-outlined text-[18px]">
                        {modeIcons[m as ColorMode]}
                      </span>
                      {t(`colorMode.${m}`)}
                    </button>
                  );
                })}
              </div>
            )}
          </div>
        </Field>

        <div class="flex items-center justify-end gap-3 pt-2">
          {saved && (
            <span class="text-sm text-success">{t("profile.saved")}</span>
          )}
          {saveError && (
            <span class="text-sm text-danger">{saveError}</span>
          )}
          <button
            type="button"
            onClick={handleSave}
            disabled={saving || !nameValid || !dirty}
            class="btn-primary"
          >
            {saving ? t("profile.saving") : t("profile.save")}
          </button>
        </div>
      </section>

      <section class="flex flex-col gap-2 pt-8 border-t border-border-dark mt-8">
        <a
          href="https://github.com/brqnko/kioku/issues/new"
          target="_blank"
          rel="noopener noreferrer"
          class="flex items-center gap-4 p-2 rounded-lg hover:bg-overlay-faint text-text-primary text-left w-full h-11 no-underline"
        >
          <span class="material-symbols-outlined text-text-muted-dark">flag</span>
          <span class="text-base">{t("profile.actions.report")}</span>
        </a>
        <button
          type="button"
          onClick={() => setShowLogout(true)}
          class="flex items-center gap-4 p-2 rounded-lg hover:bg-overlay-faint text-text-primary text-left w-full h-11 cursor-pointer bg-transparent border-none"
        >
          <span class="material-symbols-outlined text-text-muted-dark">logout</span>
          <span class="text-base">{t("profile.actions.logout")}</span>
        </button>
        <button
          type="button"
          onClick={() => setShowDelete(true)}
          class="flex items-center gap-4 p-2 rounded-lg hover:bg-danger/10 text-danger text-left w-full h-11 mt-4 cursor-pointer bg-transparent border-none"
        >
          <span class="material-symbols-outlined">delete</span>
          <span class="text-base font-medium">
            {t("profile.actions.delete")}
          </span>
        </button>
      </section>

      <Dialog
        open={showLogout}
        onClose={() => setShowLogout(false)}
        ariaLabel={t("profile.logoutConfirm.title")}
        maxWidth="max-w-md"
      >
        <div class="p-6 flex flex-col gap-4">
          <h3 class="heading-h2">{t("profile.logoutConfirm.title")}</h3>
          <p class="text-body text-text-secondary">
            {t("profile.logoutConfirm.body")}
          </p>
          <div class="flex justify-end gap-3 mt-2">
            <button
              type="button"
              onClick={() => setShowLogout(false)}
              class="btn-ghost"
            >
              {t("profile.cancel")}
            </button>
            <button type="button" onClick={handleLogout} class="btn-primary">
              {t("profile.actions.logout")}
            </button>
          </div>
        </div>
      </Dialog>

      <Dialog
        open={showDelete}
        onClose={() => {
          if (!deleting) {
            setShowDelete(false);
            setDeleteInput("");
          }
        }}
        ariaLabel={t("profile.deleteConfirm.title")}
        maxWidth="max-w-md"
      >
        <div class="p-6 flex flex-col gap-4">
          <h3 class="heading-h2 text-danger">
            {t("profile.deleteConfirm.title")}
          </h3>
          <p class="text-body text-text-secondary">
            {t("profile.deleteConfirm.body")}
          </p>
          <div class="flex flex-col gap-2">
            <label
              for="delete-confirm"
              class="text-caption font-medium text-text-secondary"
            >
              {t("profile.deleteConfirm.inputLabel", {
                phrase: t("profile.deleteConfirm.phrase"),
              })}
            </label>
            <input
              id="delete-confirm"
              type="text"
              value={deleteInput}
              onInput={(e) =>
                setDeleteInput((e.target as HTMLInputElement).value)
              }
              autocomplete="off"
              class="input-field"
            />
          </div>
          <div class="flex justify-end gap-3 mt-2">
            <button
              type="button"
              onClick={() => {
                setShowDelete(false);
                setDeleteInput("");
              }}
              disabled={deleting}
              class="btn-ghost"
            >
              {t("profile.cancel")}
            </button>
            <button
              type="button"
              onClick={handleDelete}
              disabled={
                deleting || deleteInput !== t("profile.deleteConfirm.phrase")
              }
              class="btn-danger"
            >
              {deleting
                ? t("profile.deleteConfirm.deleting")
                : t("profile.deleteConfirm.confirm")}
            </button>
          </div>
        </div>
      </Dialog>
    </>
  );
}

interface FieldProps {
  label: string;
  bordered?: boolean;
  children: preact.ComponentChildren;
}

function Field({ label, bordered, children }: FieldProps) {
  return (
    <div
      class={`flex flex-col gap-2 ${bordered ? "pt-4 border-t border-border-dark" : ""}`}
    >
      <label class="text-sm text-text-primary font-medium">{label}</label>
      {children}
    </div>
  );
}
