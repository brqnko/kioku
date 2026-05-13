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
  if (el) hydrate(<App />, el);
}
