import { useEffect, useMemo, useRef, useState } from "preact/hooks";
import type { Compiler } from "../hooks/useCompilers";
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
  list: Compiler[],
  preferredLanguage: string,
  query: string,
): Group[] {
  const q = query.trim().toLowerCase();
  const filtered = q
    ? list.filter((c) => {
        const hay = [
          c.language ?? "",
          c.display_name ?? "",
          c.name ?? "",
          c.version ?? "",
        ]
          .join(" ")
          .toLowerCase();
        return hay.includes(q);
      })
    : list;

  const buckets = new Map<string, Compiler[]>();
  for (const c of filtered) {
    const key = (c.language ?? "other").toLowerCase();
    let arr = buckets.get(key);
    if (!arr) {
      arr = [];
      buckets.set(key, arr);
    }
    arr.push(c);
  }

  const pref = preferredLanguage.toLowerCase().replace("c++", "cpp");
  const groups: Group[] = [];
  if (pref && buckets.has(pref)) {
    groups.push({ language: pref, items: buckets.get(pref)! });
    buckets.delete(pref);
  }
  const sorted = [...buckets.entries()].sort((a, b) =>
    a[0].localeCompare(b[0]),
  );
  for (const [lang, items] of sorted) {
    groups.push({ language: lang, items });
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
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const groups = useMemo(
    () => buildGroups(compilers, preferredLanguage, query),
    [compilers, preferredLanguage, query],
  );

  const flat = useMemo(() => {
    const out: Compiler[] = [];
    for (const g of groups) for (const c of g.items) out.push(c);
    return out;
  }, [groups]);

  const [activeIdx, setActiveIdx] = useState(0);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    setActiveIdx(0);
    setTimeout(() => inputRef.current?.focus(), 30);
  }, [open]);

  useEffect(() => {
    setActiveIdx(0);
  }, [query]);

  const total = flat.length;

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveIdx((i) => (total === 0 ? 0 : (i + 1) % total));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveIdx((i) => (total === 0 ? 0 : (i - 1 + total) % total));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const c = flat[activeIdx];
      if (c) {
        onSelect(c.name);
        onClose();
      }
    }
  };

  return (
    <Dialog
      open={open}
      onClose={onClose}
      ariaLabel="コンパイラを選択"
      maxWidth="max-w-[560px]"
    >
      <div class="flex flex-col max-h-[70vh]">
        <div class="p-4 border-b border-border-subtle shrink-0">
          <input
            ref={inputRef}
            type="text"
            value={query}
            onInput={(e) => setQuery((e.target as HTMLInputElement).value)}
            onKeyDown={handleKeyDown}
            placeholder="言語名やコンパイラ名で検索…"
            class="w-full h-10 bg-surface-dark border border-border-subtle rounded-md px-3 text-sm text-text-primary placeholder:text-text-disabled focus:outline-none focus:border-accent-blue"
          />
          {preferredLanguage && (
            <p class="text-[11px] text-text-disabled mt-2">
              現在の言語タグ: <span class="font-mono">{preferredLanguage}</span>
            </p>
          )}
        </div>
        <div class="flex-1 overflow-y-auto py-2">
          {total === 0 ? (
            <p class="text-sm text-text-disabled text-center py-8 px-4">
              {compilers.length === 0
                ? "コンパイラ一覧を読み込み中…"
                : "一致するコンパイラがありません"}
            </p>
          ) : (
            (() => {
              let runningIdx = 0;
              return groups.map((g) => (
                <div key={g.language} class="mb-2">
                  <div class="text-[10px] uppercase tracking-wider text-text-disabled px-4 py-1">
                    {g.language}
                  </div>
                  {g.items.map((c) => {
                    const idx = runningIdx++;
                    const active = idx === activeIdx;
                    const isSelected = c.name === selected;
                    return (
                      <button
                        key={c.name}
                        type="button"
                        onClick={() => {
                          onSelect(c.name);
                          onClose();
                        }}
                        onMouseEnter={() => setActiveIdx(idx)}
                        class={`w-full text-left px-4 py-2 cursor-pointer flex items-center gap-2 border-none bg-transparent ${
                          active
                            ? "bg-overlay-soft"
                            : "hover:bg-overlay-faint"
                        }`}
                      >
                        <span class="flex-1 min-w-0">
                          <span class="text-sm text-text-primary truncate block">
                            {c.display_name || c.name}
                            {c.version ? ` (${c.version})` : ""}
                          </span>
                          <span class="text-[11px] text-text-disabled font-mono truncate block">
                            {c.name}
                          </span>
                        </span>
                        {isSelected && (
                          <span class="material-symbols-outlined text-[16px] text-accent-blue">
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
        <div class="px-4 py-2 border-t border-border-subtle text-[11px] text-text-disabled shrink-0 flex items-center justify-between">
          <span>{total} 件</span>
          <span>↑↓ で移動 / Enter で選択 / Esc で閉じる</span>
        </div>
      </div>
    </Dialog>
  );
}
