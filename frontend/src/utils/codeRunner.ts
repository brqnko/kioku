import type { Compiler } from "../hooks/useCompilers";

const PREFERRED: Record<string, string> = {
  python: "cpython-head",
  py: "cpython-head",
  cpp: "gcc-head",
  "c++": "gcc-head",
  cxx: "gcc-head",
  cc: "gcc-head",
  c: "gcc-head-c",
  rust: "rust-head",
  rs: "rust-head",
  javascript: "nodejs-head",
  js: "nodejs-head",
  typescript: "typescript-3.9.7",
  ts: "typescript-3.9.7",
  go: "go-head",
  ruby: "ruby-head",
  rb: "ruby-head",
  java: "openjdk-head",
  haskell: "ghc-head",
  hs: "ghc-head",
  lua: "lua-5.4.4",
  php: "php-head",
  swift: "swift-5.5.1",
  scala: "scala-3.1.0",
  bash: "bash",
  sh: "bash",
  perl: "perl-head",
  pl: "perl-head",
  zig: "zig-head",
  nim: "nim-head",
  crystal: "crystal-head",
  elixir: "elixir-head",
  ex: "elixir-head",
  d: "dmd-head",
  ocaml: "ocaml-head",
  ml: "ocaml-head",
  julia: "julia-head",
  jl: "julia-head",
};

export function normalizeLanguageKey(lang: string | undefined | null): string {
  return (lang ?? "").toLowerCase().trim().replace("c++", "cpp");
}

function buildLanguageBuckets(list: Compiler[]): Map<string, Compiler[]> {
  const buckets = new Map<string, Compiler[]>();
  for (const compiler of list) {
    const key = normalizeLanguageKey(compiler.language);
    let bucket = buckets.get(key);
    if (!bucket) {
      bucket = [];
      buckets.set(key, bucket);
    }
    bucket.push(compiler);
  }
  return buckets;
}

export function pickCompiler(
  tag: string | undefined,
  list: Compiler[],
): string | null {
  const key = normalizeLanguageKey(tag);
  const buckets = buildLanguageBuckets(list);
  const candidates = buckets.get(key) ?? [];
  const preferredId = PREFERRED[key] ?? PREFERRED[(tag ?? "").toLowerCase()];
  const preferred =
    preferredId && list.find((compiler) => compiler.name === preferredId)?.name;

  return preferred ?? candidates[0]?.name ?? null;
}
