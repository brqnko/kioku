import { useEffect, useState } from "preact/hooks";
import { marked, Renderer, type Tokens } from "marked";
import DOMPurify from "isomorphic-dompurify";
import { highlightToHtml, normalizeLang } from "../utils/shiki";

interface Props {
  source: string;
  className?: string;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

export function MarkdownView({ source, className }: Props) {
  const [html, setHtml] = useState<string>("");

  useEffect(() => {
    let cancelled = false;
    if (!source) {
      setHtml("");
      return;
    }
    const renderer = new Renderer();
    renderer.code = ({ text, lang }: Tokens.Code): string => {
      const normalized = normalizeLang(lang);
      return highlightToHtml(text, normalized).catch(
        () => `<pre><code>${escapeHtml(text)}</code></pre>`,
      ) as unknown as string;
    };
    marked
      .parse(source, { async: true, breaks: true, renderer })
      .then((raw) => {
        const safe = DOMPurify.sanitize(raw, {
          ADD_ATTR: ["target", "rel"],
        });
        if (!cancelled) setHtml(safe);
      })
      .catch(() => {
        if (!cancelled) setHtml(escapeHtml(source));
      });
    return () => {
      cancelled = true;
    };
  }, [source]);

  return (
    <div class={className} dangerouslySetInnerHTML={{ __html: html }} />
  );
}
