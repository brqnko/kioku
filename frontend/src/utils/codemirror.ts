import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
} from "@codemirror/autocomplete";
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import {
  bracketMatching,
  defaultHighlightStyle,
  indentOnInput,
  syntaxHighlighting,
} from "@codemirror/language";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import type { Extension } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import {
  EditorView,
  highlightActiveLine,
  keymap,
  lineNumbers,
} from "@codemirror/view";

type LangLoader = () => Promise<Extension>;

const LOADERS: Record<string, LangLoader> = {
  python: async () => (await import("@codemirror/lang-python")).python(),
  cpp: async () => (await import("@codemirror/lang-cpp")).cpp(),
  c: async () => (await import("@codemirror/lang-cpp")).cpp(),
  rust: async () => (await import("@codemirror/lang-rust")).rust(),
  javascript: async () =>
    (await import("@codemirror/lang-javascript")).javascript(),
  typescript: async () =>
    (await import("@codemirror/lang-javascript")).javascript({
      typescript: true,
    }),
  jsx: async () =>
    (await import("@codemirror/lang-javascript")).javascript({ jsx: true }),
  tsx: async () =>
    (await import("@codemirror/lang-javascript")).javascript({
      jsx: true,
      typescript: true,
    }),
  go: async () => (await import("@codemirror/lang-go")).go(),
  java: async () => (await import("@codemirror/lang-java")).java(),
  php: async () => (await import("@codemirror/lang-php")).php(),
  sql: async () => (await import("@codemirror/lang-sql")).sql(),
  json: async () => (await import("@codemirror/lang-json")).json(),
  yaml: async () => (await import("@codemirror/lang-yaml")).yaml(),
  html: async () => (await import("@codemirror/lang-html")).html(),
  css: async () => (await import("@codemirror/lang-css")).css(),
  markdown: async () => (await import("@codemirror/lang-markdown")).markdown(),
  xml: async () => (await import("@codemirror/lang-xml")).xml(),
  bash: async () => {
    const { StreamLanguage } = await import("@codemirror/language");
    const { shell } = await import("@codemirror/legacy-modes/mode/shell");
    return StreamLanguage.define(shell);
  },
  lua: async () => {
    const { StreamLanguage } = await import("@codemirror/language");
    const { lua } = await import("@codemirror/legacy-modes/mode/lua");
    return StreamLanguage.define(lua);
  },
  perl: async () => {
    const { StreamLanguage } = await import("@codemirror/language");
    const { perl } = await import("@codemirror/legacy-modes/mode/perl");
    return StreamLanguage.define(perl);
  },
  haskell: async () => {
    const { StreamLanguage } = await import("@codemirror/language");
    const { haskell } = await import("@codemirror/legacy-modes/mode/haskell");
    return StreamLanguage.define(haskell);
  },
  julia: async () => {
    const { StreamLanguage } = await import("@codemirror/language");
    const { julia } = await import("@codemirror/legacy-modes/mode/julia");
    return StreamLanguage.define(julia);
  },
  ocaml: async () => {
    const { StreamLanguage } = await import("@codemirror/language");
    const { oCaml } = await import("@codemirror/legacy-modes/mode/mllike");
    return StreamLanguage.define(oCaml);
  },
  ruby: async () => {
    const { StreamLanguage } = await import("@codemirror/language");
    const { ruby } = await import("@codemirror/legacy-modes/mode/ruby");
    return StreamLanguage.define(ruby);
  },
  d: async () => {
    const { StreamLanguage } = await import("@codemirror/language");
    const { d } = await import("@codemirror/legacy-modes/mode/d");
    return StreamLanguage.define(d);
  },
};

const ALIASES: Record<string, string> = {
  py: "python",
  "c++": "cpp",
  cxx: "cpp",
  cc: "cpp",
  h: "c",
  hpp: "cpp",
  rs: "rust",
  js: "javascript",
  ts: "typescript",
  rb: "ruby",
  hs: "haskell",
  pl: "perl",
  ml: "ocaml",
  jl: "julia",
  sh: "bash",
  zsh: "bash",
  yml: "yaml",
  md: "markdown",
  ex: "ruby",
  exs: "ruby",
};

export function resolveLang(input: string | undefined | null): string {
  if (!input) return "";
  const key = input.toLowerCase().trim();
  if (LOADERS[key]) return key;
  const alias = ALIASES[key];
  if (alias && LOADERS[alias]) return alias;
  return "";
}

const cache = new Map<string, Promise<Extension>>();

export function loadLanguageExtension(langKey: string): Promise<Extension> {
  if (!langKey || !LOADERS[langKey]) return Promise.resolve([]);
  let promise = cache.get(langKey);
  if (!promise) {
    promise = LOADERS[langKey]().catch(() => [] as Extension);
    cache.set(langKey, promise);
  }
  return promise;
}

export function createBaseExtensions(): Extension {
  return [
    lineNumbers(),
    highlightActiveLine(),
    history(),
    bracketMatching(),
    closeBrackets(),
    indentOnInput(),
    autocompletion(),
    highlightSelectionMatches(),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    EditorView.lineWrapping,
    keymap.of([
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...completionKeymap,
      indentWithTab,
    ]),
  ];
}

export type ThemeMode = "light" | "dark";

export function getCurrentTheme(): ThemeMode {
  if (typeof document === "undefined") return "dark";
  return document.documentElement.getAttribute("data-theme") === "light"
    ? "light"
    : "dark";
}

export function getThemeExtension(mode: ThemeMode): Extension {
  return mode === "dark" ? oneDark : [];
}

const themeSubscribers = new Set<(mode: ThemeMode) => void>();
let themeObserver: MutationObserver | null = null;

function ensureThemeObserver(): void {
  if (themeObserver || typeof document === "undefined") return;
  themeObserver = new MutationObserver(() => {
    const mode = getCurrentTheme();
    for (const callback of themeSubscribers) {
      callback(mode);
    }
  });
  themeObserver.observe(document.documentElement, {
    attributeFilter: ["data-theme"],
    attributes: true,
  });
}

export function onThemeChange(callback: (mode: ThemeMode) => void): () => void {
  ensureThemeObserver();
  themeSubscribers.add(callback);
  return () => {
    themeSubscribers.delete(callback);
  };
}
