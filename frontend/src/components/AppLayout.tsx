import type { ComponentChildren } from "preact";
import TopAppBar from "./TopAppBar";

interface AppLayoutProps {
  children: ComponentChildren;
  className?: string;
  padding?: "default" | "none";
  scroll?: "auto" | "hidden";
}

export function AppLayout({
  children,
  className = "",
  padding = "default",
  scroll = "auto",
}: AppLayoutProps) {
  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <TopAppBar />
      <main
        class={`app-main ${padding === "none" ? "app-main--flush" : ""} h-[calc(100vh-3.5rem)] ${
          scroll === "hidden" ? "overflow-hidden" : "overflow-y-auto"
        } ${className}`}
      >
        {children}
      </main>
    </div>
  );
}
