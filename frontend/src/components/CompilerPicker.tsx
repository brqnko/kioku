import { useEffect, useMemo, useRef, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import type { Compiler } from "../hooks/useCompilers";
import { normalizeLanguageKey } from "../utils/codeRunner";
import { Dialog } from "./Dialog";

interface Props {
  open: boolean;
  onClose: () => void;
  compilers: Compiler[];
  selected: string | null;
  preferredLanguage: string;
  onSelect: (name: string) => void;
}

interface Group {
  language: string;
  items: Compiler[];
}

function buildGroups(
  compilers: Compiler[],
  preferredLanguage: string,
  query: string,
): Group[] {
  const q = query.trim().toLowerCase();
  const filtered = q
    ? compilers.filter((compiler) =>
        [
          compiler.language,
          compiler.display_name,
          compiler.name,
          compiler.version,
        ]
          .join(" ")
          .toLowerCase()
          .includes(q),
      )
    : compilers;

  const buckets = new Map<string, Compiler[]>();
  for (const compiler of filtered) {
    const key = normalizeLanguageKey(compiler.language) || "other";
    const bucket = buckets.get(key);
    if (bucket) {
      bucket.push(compiler);
    } else {
      buckets.set(key, [compiler]);
    }
  }

  const preferredKey = normalizeLanguageKey(preferredLanguage);
  const groups: Group[] = [];
  if (preferredKey && buckets.has(preferredKey)) {
    groups.push({ language: preferredKey, items: buckets.get(preferredKey)! });
    buckets.delete(preferredKey);
  }

  for (const [language, items] of [...buckets.entries()].sort((a, b) =>
    a[0].localeCompare(b[0]),
  )) {
    groups.push({ language, items });
  }

  return groups;
}

export function CompilerPicker({
  open,
  onClose,
  compilers,
  selected,
  preferredLanguage,
  onSelect,
}: Props) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const groups = useMemo(
    () => buildGroups(compilers, preferredLanguage, query),
    [compilers, preferredLanguage, query],
  );
  const flat = useMemo(() => groups.flatMap((group) => group.items), [groups]);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    setActiveIndex(0);
    const timer = window.setTimeout(() => inputRef.current?.focus(), 30);
    return () => window.clearTimeout(timer);
  }, [open]);

  useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setActiveIndex((index) =>
        flat.length === 0 ? 0 : (index + 1) % flat.length,
      );
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setActiveIndex((index) =>
        flat.length === 0 ? 0 : (index - 1 + flat.length) % flat.length,
      );
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      const compiler = flat[activeIndex];
      if (compiler) {
        onSelect(compiler.name);
        onClose();
      }
    }
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      ariaLabel={t("codeBlock.picker.title")}
      maxWidth="max-w-[620px]"
    >
      <div class="compiler-picker">
        <div class="compiler-picker-header">
          <div>
            <div class="compiler-picker-title">
              {t("codeBlock.picker.title")}
            </div>
          </div>
          <button
            type="button"
            class="icon-button"
            aria-label={t("codeBlock.picker.close")}
            onClick={onClose}
          >
            <span class="material-symbols-outlined text-[20px]">close</span>
          </button>
        </div>
        <div class="compiler-picker-search">
          <span class="material-symbols-outlined text-[18px]">search</span>
          <input
            ref={inputRef}
            type="text"
            value={query}
            onInput={(event) =>
              setQuery((event.target as HTMLInputElement).value)
            }
            onKeyDown={handleKeyDown}
            placeholder={t("codeBlock.picker.search")}
          />
        </div>
        <div class="compiler-picker-list">
          {flat.length === 0 ? (
            <div class="compiler-picker-empty">
              {compilers.length === 0
                ? t("codeBlock.picker.loading")
                : t("codeBlock.picker.empty")}
            </div>
          ) : (
            (() => {
              let runningIndex = 0;
              return groups.map((group) => (
                <div class="compiler-picker-group" key={group.language}>
                  <div class="compiler-picker-group-label">
                    {group.language}
                  </div>
                  {group.items.map((compiler) => {
                    const index = runningIndex++;
                    const isActive = index === activeIndex;
                    const isSelected = compiler.name === selected;
                    return (
                      <button
                        type="button"
                        key={compiler.name}
                        class={`compiler-picker-option ${
                          isActive ? "is-active" : ""
                        } ${isSelected ? "is-selected" : ""}`}
                        onMouseEnter={() => setActiveIndex(index)}
                        onClick={() => {
                          onSelect(compiler.name);
                          onClose();
                        }}
                      >
                        <span class="compiler-picker-option-main">
                          <span class="compiler-picker-option-name">
                            {compiler.display_name || compiler.name}
                          </span>
                          <span class="compiler-picker-option-meta">
                            {compiler.name}
                            {compiler.version ? ` / ${compiler.version}` : ""}
                          </span>
                        </span>
                        {isSelected && (
                          <span class="material-symbols-outlined text-[18px]">
                            check
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>
              ));
            })()
          )}
        </div>
      </div>
    </Dialog>
  );
}
