import { LocationProvider, Router, Route, lazy, hydrate, prerender as ssr } from "preact-iso";
import { SWRConfig } from "swr";
import "./style.css";

const LandingPage = lazy(() => import("./pages/LandingPage.jsx"));
const TOSPage = lazy(() => import("./pages/TOSPage.jsx"));
const PrivacyPolicyPage = lazy(() => import("./pages/PrivacyPolicyPage.jsx"));
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
