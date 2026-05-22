import { useEffect } from "preact/hooks";

const SITE_URL = "https://kioku.brqnko.rs";

export type RobotsDirective =
  | "index,follow"
  | "noindex,follow"
  | "noindex,nofollow";

export interface DocumentHeadOptions {
  title?: string;
  description?: string;
  /** Absolute URL or path. If a path, SITE_URL is prepended. */
  canonical?: string;
  robots?: RobotsDirective;
  ogTitle?: string;
  ogDescription?: string;
  /** Absolute URL or path. If a path, SITE_URL is prepended. */
  ogUrl?: string;
}

const MANAGED_ATTR = "data-managed";
const MANAGED_VAL = "useDocumentHead";

function absoluteUrl(value: string): string {
  if (/^https?:\/\//i.test(value)) return value;
  return `${SITE_URL}${value.startsWith("/") ? value : `/${value}`}`;
}

function upsertMeta(
  selector: string,
  attrName: string,
  attrValue: string,
  content: string,
) {
  let el = document.head.querySelector<HTMLMetaElement>(selector);
  if (!el) {
    el = document.createElement("meta");
    el.setAttribute(attrName, attrValue);
    el.setAttribute(MANAGED_ATTR, MANAGED_VAL);
    document.head.appendChild(el);
  }
  el.setAttribute("content", content);
}

function upsertLink(rel: string, href: string) {
  let el = document.head.querySelector<HTMLLinkElement>(`link[rel="${rel}"]`);
  if (!el) {
    el = document.createElement("link");
    el.setAttribute("rel", rel);
    el.setAttribute(MANAGED_ATTR, MANAGED_VAL);
    document.head.appendChild(el);
  }
  el.setAttribute("href", href);
}

export function useDocumentHead(opts: DocumentHeadOptions): void {
  useEffect(() => {
    if (typeof document === "undefined") return;

    if (opts.title) document.title = opts.title;
    if (opts.description) {
      upsertMeta(
        'meta[name="description"]',
        "name",
        "description",
        opts.description,
      );
    }
    if (opts.canonical) {
      upsertLink("canonical", absoluteUrl(opts.canonical));
    }
    if (opts.robots) {
      upsertMeta('meta[name="robots"]', "name", "robots", opts.robots);
    }
    if (opts.ogTitle) {
      upsertMeta(
        'meta[property="og:title"]',
        "property",
        "og:title",
        opts.ogTitle,
      );
      upsertMeta(
        'meta[name="twitter:title"]',
        "name",
        "twitter:title",
        opts.ogTitle,
      );
    }
    if (opts.ogDescription) {
      upsertMeta(
        'meta[property="og:description"]',
        "property",
        "og:description",
        opts.ogDescription,
      );
      upsertMeta(
        'meta[name="twitter:description"]',
        "name",
        "twitter:description",
        opts.ogDescription,
      );
    }
    if (opts.ogUrl) {
      upsertMeta(
        'meta[property="og:url"]',
        "property",
        "og:url",
        absoluteUrl(opts.ogUrl),
      );
    }

    return () => {
      // Restore the default robots directive when leaving a noindex page so SPA
      // back-navigation to public routes does not inherit noindex.
      if (opts.robots && opts.robots !== "index,follow") {
        upsertMeta('meta[name="robots"]', "name", "robots", "index,follow");
      }
    };
  }, [
    opts.title,
    opts.description,
    opts.canonical,
    opts.robots,
    opts.ogTitle,
    opts.ogDescription,
    opts.ogUrl,
  ]);
}
