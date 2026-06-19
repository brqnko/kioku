import { useTranslation } from "react-i18next";
import type { ComponentChildren } from "preact";

type LegalBlock = string | { text: string; sub?: string[] };

interface LegalArticle {
  title: string;
  items: LegalBlock[];
}

// split keeps the captured email as its own element; test is anchored so the
// global-regex `lastIndex` statefulness of `.test()` can't bite us.
const EMAIL_SPLIT = /([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})/;
const EMAIL_MATCH = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;

function linkify(text: string): ComponentChildren {
  const parts = text.split(EMAIL_SPLIT);
  if (parts.length === 1) return text;
  return parts.map((part, i) =>
    EMAIL_MATCH.test(part) ? (
      <a
        key={i}
        href={`mailto:${part}`}
        class="text-primary underline font-medium"
      >
        {part}
      </a>
    ) : (
      part
    ),
  );
}

function SubList({ items }: { items: string[] }) {
  return (
    <ul class="mt-3 space-y-2 list-none m-0 p-0">
      {items.map((s, i) => (
        <li key={i} class="flex gap-2">
          <span class="shrink-0 tabular-nums">({i + 1})</span>
          <span>{linkify(s)}</span>
        </li>
      ))}
    </ul>
  );
}

function BlockBody({ block }: { block: LegalBlock }) {
  const text = typeof block === "string" ? block : block.text;
  const sub = typeof block === "string" ? undefined : block.sub;
  return (
    <>
      {linkify(text)}
      {sub && sub.length > 0 && <SubList items={sub} />}
    </>
  );
}

function Article({ article }: { article: LegalArticle }) {
  const numbered = article.items.length > 1;
  return (
    <section class="mb-10 last:mb-0">
      <h2 class="font-bold text-xl mb-4 text-charcoal dark:text-white m-0 flex items-center gap-3">
        <span class="text-primary">§</span>
        {article.title}
      </h2>
      {numbered ? (
        <ol class="list-decimal pl-6 space-y-3 m-0">
          {article.items.map((block, i) => (
            <li key={i} class="pl-1">
              <BlockBody block={block} />
            </li>
          ))}
        </ol>
      ) : (
        <div class="space-y-3">
          {article.items.map((block, i) => (
            <div key={i}>
              <BlockBody block={block} />
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

export function LegalArticles({ baseKey }: { baseKey: "tos" | "privacy" }) {
  const { t } = useTranslation();
  const intro = t(`${baseKey}.intro`);
  const updated = t(`${baseKey}.updated`);
  const articles = t(`${baseKey}.articles`, {
    returnObjects: true,
  }) as LegalArticle[];

  return (
    <>
      <div class="mb-16 text-center">
        <h1 class="text-4xl md:text-6xl font-bold leading-tight tracking-tight mb-6 text-charcoal dark:text-white">
          {t(`${baseKey}.title`)}
        </h1>
      </div>

      <div class="max-w-4xl mx-auto">
        <div class="bg-card-light dark:bg-card-dark border border-border-light dark:border-border-dark rounded-[12px] overflow-hidden mb-12 shadow-[0_1px_3px_rgba(0,0,0,0.1)]">
          <div class="p-10 md:p-16 text-lg text-taupe dark:text-text-muted-dark leading-8 tracking-[0.01em]">
            {intro && (
              <p class="m-0 mb-10 pb-10 border-b border-border-light dark:border-border-dark">
                {linkify(intro)}
              </p>
            )}

            {articles.map((article, i) => (
              <Article key={i} article={article} />
            ))}

            {updated && (
              <p class="m-0 mt-12 pt-8 border-t border-border-light dark:border-border-dark text-right text-base">
                {updated}
              </p>
            )}
          </div>
        </div>
      </div>
    </>
  );
}
