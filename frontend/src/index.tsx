import { LocationProvider, Router, Route, lazy, hydrate } from "preact-iso";
import { SWRConfig } from "swr";
import "./i18n";
import "./style.css";
import { NotificationBanner } from "./components/NotificationBanner";

const LandingPage = lazy(() => import("./pages/LandingPage.jsx"));
const TOSPage = lazy(() => import("./pages/TOSPage.jsx"));
const PrivacyPolicyPage = lazy(() => import("./pages/PrivacyPolicyPage.jsx"));
const DashboardPage = lazy(() => import("./pages/DashboardPage.jsx"));
const LibraryPage = lazy(() => import("./pages/LibraryPage.jsx"));
const ProjectPage = lazy(() => import("./pages/ProjectPage.jsx"));
const FolderPage = lazy(() => import("./pages/FolderPage.jsx"));
const FilePage = lazy(() => import("./pages/FilePage.jsx"));
const PodcastPage = lazy(() => import("./pages/PodcastPage.jsx"));
const ChatPage = lazy(() => import("./pages/ChatPage.jsx"));
const ProjectChatPage = lazy(() => import("./pages/ProjectChatPage.jsx"));
const ProjectPodcastsPage = lazy(() => import("./pages/ProjectPodcastsPage.jsx"));
const PodcastNewPage = lazy(() => import("./pages/PodcastNewPage.jsx"));
const PodcastDetailPage = lazy(() => import("./pages/PodcastDetailPage.jsx"));
const ProfilePage = lazy(() => import("./pages/ProfilePage.jsx"));
const NotFound = lazy(() => import("./_404.jsx"));

export function App() {
  return (
    <LocationProvider>
      <SWRConfig
        value={{
          revalidateOnFocus: true,
          revalidateOnReconnect: true,
          dedupingInterval: 30_000,
          focusThrottleInterval: 10_000,
          keepPreviousData: true,
          errorRetryCount: 1,
        }}
      >
        <NotificationBanner />
        <Router>
          <Route path="/" component={LandingPage} />
          <Route path="/tos" component={TOSPage} />
          <Route path="/privacy" component={PrivacyPolicyPage} />
          <Route path="/dashboard" component={DashboardPage} />
          <Route path="/library" component={LibraryPage} />
          <Route path="/projects/:projectId/chat" component={ProjectChatPage} />
          <Route path="/projects/:projectId" component={ProjectPage} />
          <Route path="/folders/:folderId" component={FolderPage} />
          <Route path="/files/:fileId" component={FilePage} />
          <Route path="/podcast" component={PodcastPage} />
          <Route path="/chat" component={ChatPage} />
          <Route
            path="/projects/:projectId/podcasts"
            component={ProjectPodcastsPage}
          />
          <Route
            path="/projects/:projectId/podcasts/new"
            component={PodcastNewPage}
          />
          <Route
            path="/projects/:projectId/podcasts/:podcastId"
            component={PodcastDetailPage}
          />
          <Route path="/profile" component={ProfilePage} />
          <Route default component={NotFound} />
        </Router>
      </SWRConfig>
    </LocationProvider>
  );
}

if (typeof window !== "undefined") {
  const el = document.getElementById("app");
  if (el) {
    // nginx serves the prerendered LandingPage HTML (/index.html) for SPA routes
    // that aren't prerendered. Hydrating that DOM against a different route's
    // VNodes leaves stale nodes from the LandingPage. Drop the prerendered
    // markup on non-prerendered routes so preact renders fresh.
    const PRERENDERED = new Set(["/", "/tos", "/privacy", "/404"]);
    if (!PRERENDERED.has(window.location.pathname)) {
      el.innerHTML = "";
    }
    hydrate(<App />, el);
  }
}

// --- Prerender ---------------------------------------------------------------
// Called by @preact/preset-vite (via vite-prerender-plugin) at build time for
// each route in `additionalPrerenderRoutes` plus any links it discovers while
// crawling.

const SITE_URL = "https://kioku.brqnko.rs";

type HeadElement = { type: string; props: Record<string, string> };

interface RouteHead {
  title: string;
  description: string;
  canonical?: string;
  robots: "index,follow" | "noindex,follow" | "noindex,nofollow";
}

const ROUTE_META: Record<string, RouteHead> = {
  "/": {
    title: "kioku — Folder structure for NotebookLM",
    description:
      "kioku adds folders, projects, and a tidy library to NotebookLM-style knowledge bases. Organize research, chat with your files, and learn through AI podcasts.",
    canonical: "/",
    robots: "index,follow",
  },
  "/tos": {
    title: "Terms of Service — kioku",
    description:
      "Terms of service for kioku, the folder-organized companion for NotebookLM-style knowledge bases.",
    canonical: "/tos",
    robots: "index,follow",
  },
  "/privacy": {
    title: "Privacy Policy — kioku",
    description:
      "How kioku handles your account data and uploaded materials.",
    canonical: "/privacy",
    robots: "index,follow",
  },
  "/404": {
    title: "Page not found — kioku",
    description: "The page you requested does not exist.",
    robots: "noindex,follow",
  },
  "/_shell": {
    title: "kioku",
    description: "Knowledge and learning management system",
    robots: "noindex,nofollow",
  },
};

function buildHeadFor(url: string): {
  title: string;
  lang: string;
  elements: Set<HeadElement>;
} {
  const meta = ROUTE_META[url] ?? ROUTE_META["/"];
  const elements = new Set<HeadElement>();

  elements.add({
    type: "meta",
    props: { name: "description", content: meta.description },
  });
  elements.add({
    type: "meta",
    props: { name: "robots", content: meta.robots },
  });
  elements.add({
    type: "meta",
    props: { property: "og:title", content: meta.title },
  });
  elements.add({
    type: "meta",
    props: { property: "og:description", content: meta.description },
  });
  elements.add({
    type: "meta",
    props: { name: "twitter:title", content: meta.title },
  });
  elements.add({
    type: "meta",
    props: { name: "twitter:description", content: meta.description },
  });
  if (meta.canonical) {
    const absolute = `${SITE_URL}${meta.canonical}`;
    elements.add({
      type: "link",
      props: { rel: "canonical", href: absolute },
    });
    elements.add({
      type: "meta",
      props: { property: "og:url", content: absolute },
    });
  }

  return { title: meta.title, lang: "en", elements };
}

export async function prerender(data: { url: string }) {
  // /_shell is an empty SPA shell. nginx serves it as the fallback for routes
  // that aren't prerendered (auth-only pages and dynamic routes like
  // /projects/:id). Returning empty html avoids the hydration mismatch that
  // would otherwise occur when the LandingPage prerender was served for a
  // different route.
  if (data.url === "/_shell") {
    return { html: "", head: buildHeadFor(data.url) };
  }
  const { default: ssr } = await import("preact-iso/prerender");
  const { html, links } = await ssr(<App />);
  return {
    html,
    links,
    head: buildHeadFor(data.url),
  };
}
