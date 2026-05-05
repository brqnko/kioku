import { LocationProvider, Router, Route, lazy, hydrate, prerender as ssr } from "preact-iso";
import { SWRConfig } from "swr";
import "./i18n";
import "./style.css";

const LandingPage = lazy(() => import("./pages/LandingPage.jsx"));
const TOSPage = lazy(() => import("./pages/TOSPage.jsx"));
const PrivacyPolicyPage = lazy(() => import("./pages/PrivacyPolicyPage.jsx"));
const DashboardPage = lazy(() => import("./pages/DashboardPage.jsx"));
const LibraryPage = lazy(() => import("./pages/LibraryPage.jsx"));
const ProjectPage = lazy(() => import("./pages/ProjectPage.jsx"));
const FolderPage = lazy(() => import("./pages/FolderPage.jsx"));
const FilePage = lazy(() => import("./pages/FilePage.jsx"));
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
        <Router>
          <Route path="/" component={LandingPage} />
          <Route path="/tos" component={TOSPage} />
          <Route path="/privacy" component={PrivacyPolicyPage} />
          <Route path="/dashboard" component={DashboardPage} />
          <Route path="/library" component={LibraryPage} />
          <Route path="/projects/:projectId" component={ProjectPage} />
          <Route path="/folders/:folderId" component={FolderPage} />
          <Route path="/files/:fileId" component={FilePage} />
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

export async function prerender(data: Record<string, unknown>) {
  return await ssr(<App {...data} />);
}
