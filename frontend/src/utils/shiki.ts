import {
  createHighlighter,
  type BundledLanguage,
  type Highlighter,
} from "shiki";

const ALIAS_TO_SHIKI: Record<string, BundledLanguage> = {
  py: "python",
  python: "python",
  cpp: "cpp",
  "c++": "cpp",
  cxx: "cpp",
  cc: "cpp",
  c: "c",
  h: "c",
  rs: "rust",
  rust: "rust",
  js: "javascript",
  javascript: "javascript",
  jsx: "jsx",
  ts: "typescript",
  typescript: "typescript",
  tsx: "tsx",
  go: "go",
  rb: "ruby",
  ruby: "ruby",
  java: "java",
  hs: "haskell",
  haskell: "haskell",
  lua: "lua",
  php: "php",
  swift: "swift",
  scala: "scala",
  bash: "bash",
  sh: "bash",
  zsh: "bash",
  perl: "perl",
  pl: "perl",
  zig: "zig",
  nim: "nim",
  crystal: "crystal",
  elixir: "elixir",
  ex: "elixir",
  exs: "elixir",
  d: "d",
  ocaml: "ocaml",
  ml: "ocaml",
  julia: "julia",
  jl: "julia",
  json: "json",
  yaml: "yaml",
  yml: "yaml",
  toml: "toml",
  html: "html",
  css: "css",
  scss: "scss",
  md: "markdown",
  markdown: "markdown",
  sql: "sql",
  dockerfile: "docker",
  docker: "docker",
  xml: "xml",
};

const HOT_LANGS: BundledLanguage[] = [
  "javascript",
  "typescript",
  "python",
  "bash",
  "json",
  "html",
  "css",
];

export function normalizeLang(input: string | undefined | null): string {
  if (!input) return "";
  const t = input.toLowerCase().trim();
  return ALIAS_TO_SHIKI[t] ?? "";
}

let highlighter: Highlighter | null = null;
let highlighterPromise: Promise<Highlighter> | null = null;

function ensureHighlighter(): Promise<Highlighter> {
  if (highlighter) return Promise.resolve(highlighter);
  if (highlighterPromise) return highlighterPromise;
  highlighterPromise = createHighlighter({
    themes: ["github-light", "github-dark"],
    langs: HOT_LANGS,
  }).then((h) => {
    highlighter = h;
    return h;
  });
  return highlighterPromise;
}

const langLoaders = new Map<string, Promise<boolean>>();

function ensureLanguage(lang: string): Promise<boolean> {
  if (!lang) return Promise.resolve(false);
  const cached = langLoaders.get(lang);
  if (cached) return cached;
  const p = (async () => {
    const h = await ensureHighlighter();
    if (h.getLoadedLanguages().includes(lang)) return true;
    try {
      await h.loadLanguage(lang as BundledLanguage);
      return true;
    } catch {
      return false;
    }
  })();
  langLoaders.set(lang, p);
  return p;
}

export async function highlightToHtml(
  code: string,
  lang: string,
): Promise<string> {
  const h = await ensureHighlighter();
  let target = "";
  if (lang) {
    const ok = await ensureLanguage(lang);
    if (ok) target = lang;
  }
  return h.codeToHtml(code, {
    lang: (target || "text") as BundledLanguage,
    themes: { light: "github-light", dark: "github-dark" },
    defaultColor: false,
  });
}
