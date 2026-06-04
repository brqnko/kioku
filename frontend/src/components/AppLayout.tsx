import type { ComponentChildren } from "preact";
import SideNavBar from "./SideNavBar";
import TopAppBar from "./TopAppBar";

interface AppLayoutProps {
  children: ComponentChildren;
  className?: string;
}

export function AppLayout({ children, className = "" }: AppLayoutProps) {
  return (
    <div class="min-h-screen bg-background-dark text-text-primary">
      <SideNavBar />
      <TopAppBar />
      <main
        class={`app-main ml-[var(--sidebar-width)] h-[calc(100vh-3.5rem)] overflow-y-auto transition-[margin-left] duration-200 ease-in-out ${className}`}
      >
        {children}
      </main>
    </div>
  );
}
