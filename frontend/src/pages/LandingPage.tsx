import { useTranslation, Trans } from "react-i18next";
import DashboardButton from "../components/DashboardButton";
import GoogleSignInButton from "../components/GoogleSignInButton";
import HeaderControls from "../components/HeaderControls";
import { Reveal } from "../components/Reveal";
import { useAuth } from "../hooks/useAuth";
import { useDocumentHead } from "../hooks/useDocumentHead";

const principles = ["synthesis", "library", "research"] as const;

function MockWindow({ children }: { children: preact.ComponentChildren }) {
  return (
    <div
      class="rounded-2xl bg-card-light dark:bg-card-dark border border-border-light dark:border-border-dark shadow-xl overflow-hidden select-none"
      aria-hidden="true"
    >
      <div class="flex items-center gap-3 px-4 py-3 border-b border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark">
        <div class="flex items-center gap-1.5">
          <span class="size-3 rounded-full bg-black/15 dark:bg-white/20" />
          <span class="size-3 rounded-full bg-black/15 dark:bg-white/20" />
          <span class="size-3 rounded-full bg-black/15 dark:bg-white/20" />
        </div>
      </div>
      <div class="bg-background-light dark:bg-background-dark">{children}</div>
    </div>
  );
}

function ExplorerMock({ t }: { t: (key: string) => string }) {
  const rows = [
    {
      kind: "folder" as const,
      icon: "folder",
      iconTone: "text-primary",
      name: t("mock.explorer.folder1"),
      type: t("mock.explorer.type.folder"),
      time: t("mock.explorer.folder1Time"),
    },
    {
      kind: "folder" as const,
      icon: "folder",
      iconTone: "text-primary",
      name: t("mock.explorer.folder2"),
      type: t("mock.explorer.type.folder"),
      time: t("mock.explorer.folder2Time"),
    },
    {
      kind: "file" as const,
      icon: "picture_as_pdf",
      iconTone: "text-rose-400",
      name: t("mock.explorer.file1"),
      type: t("mock.explorer.type.pdf"),
      time: t("mock.explorer.file1Time"),
    },
    {
      kind: "file" as const,
      icon: "description",
      iconTone: "text-sky-400",
      name: t("mock.explorer.file2"),
      type: t("mock.explorer.type.note"),
      time: t("mock.explorer.file2Time"),
    },
    {
      kind: "file" as const,
      icon: "article",
      iconTone: "text-emerald-400",
      name: t("mock.explorer.file3"),
      type: t("mock.explorer.type.markdown"),
      time: t("mock.explorer.file3Time"),
    },
  ];
  return (
    <MockWindow>
      <div class="p-6 md:p-8 bg-background-light dark:bg-background-dark">
        <div class="flex flex-col gap-2 mb-6">
          <nav class="flex items-center gap-1.5 text-xs text-text-muted-light dark:text-text-muted-dark">
            <span>{t("mock.explorer.breadcrumb")}</span>
            <span class="material-symbols-outlined text-[14px]">
              chevron_right
            </span>
            <span class="text-charcoal dark:text-white font-medium">
              {t("mock.explorer.project")}
            </span>
          </nav>
          <h4 class="text-xl md:text-2xl font-bold text-charcoal dark:text-white tracking-tight">
            {t("mock.explorer.project")}
          </h4>
          <p class="text-sm text-text-muted-light dark:text-text-muted-dark">
            {t("mock.explorer.subtitle")}
          </p>
        </div>

        <div class="flex items-center gap-2 mb-3 text-charcoal dark:text-white">
          <span class="material-symbols-outlined text-text-muted-light dark:text-text-muted-dark text-[18px]">
            folder_open
          </span>
          <span class="text-sm font-bold">{t("mock.explorer.allFiles")}</span>
        </div>

        <div class="rounded-xl border border-border-light dark:border-border-dark bg-card-light dark:bg-surface-dark overflow-hidden">
          <div class="grid grid-cols-[1fr_auto_auto] gap-4 px-4 py-2.5 text-[11px] font-semibold uppercase tracking-wider text-text-muted-light dark:text-text-muted-dark border-b border-border-light dark:border-border-dark bg-background-light dark:bg-background-dark/60">
            <span>{t("mock.explorer.column.name")}</span>
            <span>{t("mock.explorer.column.type")}</span>
            <span>{t("mock.explorer.column.modified")}</span>
          </div>
          <ul class="divide-y divide-border-light dark:divide-border-dark list-none p-0 m-0">
            {rows.map((row) => (
              <li
                key={row.name}
                class="grid grid-cols-[1fr_auto_auto] gap-4 items-center px-4 py-2.5"
              >
                <div class="flex items-center gap-2.5 min-w-0">
                  <span
                    class={`material-symbols-outlined text-[18px] ${row.iconTone}`}
                    style={{ fontVariationSettings: "'FILL' 1" }}
                  >
                    {row.icon}
                  </span>
                  <span class="text-sm font-medium text-charcoal dark:text-white truncate">
                    {row.name}
                  </span>
                </div>
                <span class="text-xs text-text-muted-light dark:text-text-muted-dark whitespace-nowrap">
                  {row.type}
                </span>
                <span class="text-xs text-text-muted-light dark:text-text-muted-dark whitespace-nowrap">
                  {row.time}
                </span>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </MockWindow>
  );
}

function ChatMock({ t }: { t: (key: string) => string }) {
  const aiLines = t("mock.chat.ai").split("\n");
  return (
    <MockWindow>
      <div class="flex flex-col gap-6 p-6 md:p-8">
        <h3 class="text-xl md:text-2xl font-bold text-charcoal dark:text-white tracking-tight">
          {t("mock.chat.title")}
        </h3>
        <div class="flex flex-col gap-4">
          <div class="self-end max-w-[80%] rounded-2xl rounded-br-md bg-primary/10 border border-primary/20 px-4 py-3">
            <p class="text-sm text-charcoal dark:text-white leading-relaxed">
              {t("mock.chat.user")}
            </p>
          </div>
          <div class="self-start max-w-[88%] rounded-2xl rounded-bl-md bg-card-light dark:bg-card-dark border border-border-light dark:border-border-dark px-4 py-3">
            <div class="text-sm text-charcoal dark:text-white leading-relaxed flex flex-col gap-1">
              {aiLines.map((line, i) => (
                <p key={i}>{line}</p>
              ))}
            </div>
          </div>
        </div>
        <div class="flex items-center gap-3 rounded-xl bg-card-light dark:bg-card-dark border border-border-light dark:border-border-dark px-4 py-3">
          <div class="h-2 flex-1 bg-black/5 dark:bg-white/5 rounded-full" />
          <div class="size-8 rounded-full bg-primary/80 shrink-0" />
        </div>
      </div>
    </MockWindow>
  );
}

function PodcastMock({ t }: { t: (key: string) => string }) {
  const transcript = [
    { speaker: t("mock.podcast.speaker1"), text: t("mock.podcast.text1") },
    { speaker: t("mock.podcast.speaker2"), text: t("mock.podcast.text2") },
  ];
  return (
    <MockWindow>
      <div class="p-6 md:p-8 bg-background-light dark:bg-background-dark flex flex-col gap-5">
        {/* Title */}
        <div class="flex flex-col items-center text-center">
          <h4 class="text-2xl font-bold text-charcoal dark:text-white leading-tight tracking-tight">
            {t("mock.podcast.name")}
          </h4>
        </div>

        {/* Player card */}
        <div class="rounded-xl bg-card-light dark:bg-surface-dark border border-border-light dark:border-border-dark p-5 flex flex-col gap-4 shadow-[0_4px_24px_rgba(0,0,0,0.15)]">
          <div class="flex flex-col gap-2">
            <div class="flex justify-between text-xs text-text-muted-light dark:text-text-muted-dark">
              <span>4:42</span>
              <span>12:18</span>
            </div>
            <div class="relative h-2 w-full bg-black/5 dark:bg-white/10 rounded-full overflow-hidden">
              <div
                class="absolute inset-y-0 left-0 bg-charcoal dark:bg-white rounded-full"
                style={{ width: "38%" }}
              />
            </div>
          </div>
          <div class="flex items-center justify-between">
            <span class="material-symbols-outlined text-[20px] text-text-muted-light dark:text-text-muted-dark">
              volume_up
            </span>
            <div class="flex items-center gap-4">
              <span class="material-symbols-outlined text-[24px] text-text-muted-light dark:text-text-muted-dark">
                replay_10
              </span>
              <span class="size-12 rounded-full bg-cta text-cta-fg flex items-center justify-center shadow-md">
                <span
                  class="material-symbols-outlined text-[24px]"
                  style={{ fontVariationSettings: "'FILL' 1" }}
                >
                  play_arrow
                </span>
              </span>
              <span class="material-symbols-outlined text-[24px] text-text-muted-light dark:text-text-muted-dark">
                forward_10
              </span>
            </div>
            <span class="px-2.5 py-1 rounded border border-border-light dark:border-border-dark text-xs font-medium text-text-muted-light dark:text-text-muted-dark">
              1x
            </span>
          </div>
        </div>

        {/* Transcript */}
        <div class="flex flex-col gap-3">
          <div class="flex items-center gap-2 pb-2 border-b border-border-light dark:border-border-dark text-text-muted-light dark:text-text-muted-dark">
            <span class="material-symbols-outlined text-[16px]">subject</span>
            <span class="text-xs font-bold text-charcoal dark:text-white">
              {t("podcast.detail.transcript")}
            </span>
          </div>
          <ol class="flex flex-col gap-3 list-none p-0 m-0">
            {transcript.map((entry, i) => (
              <li key={i} class="flex gap-3">
                <span class="shrink-0 w-16 text-[10px] font-bold uppercase tracking-widest text-primary pt-0.5">
                  {entry.speaker}
                </span>
                <p class="text-xs text-charcoal dark:text-white leading-snug flex-1">
                  {entry.text}
                </p>
              </li>
            ))}
          </ol>
        </div>
      </div>
    </MockWindow>
  );
}

export default function LandingPage() {
  const { t } = useTranslation();
  const { isAuthenticated } = useAuth();
  useDocumentHead({
    title: "kioku — Folder structure for NotebookLM",
    description:
      "kioku adds folders, projects, and a tidy library to NotebookLM-style knowledge bases. Organize research, chat with your files, and learn through AI podcasts.",
    canonical: "/",
    robots: "index,follow",
    ogTitle: "kioku — Folder structure for NotebookLM",
    ogDescription:
      "kioku adds folders, projects, and a tidy library to NotebookLM-style knowledge bases. Organize research, chat with your files, and learn through AI podcasts.",
    ogUrl: "/",
  });

  return (
    <div class="min-h-dvh bg-background-light dark:bg-background-dark text-charcoal dark:text-white/95">
      <header class="sticky top-0 z-50 border-b border-border-light dark:border-border-dark bg-background-light/90 dark:bg-background-dark/90 backdrop-blur">
        <div class="max-w-7xl mx-auto px-6 md:px-8 py-4 flex items-center justify-between">
          <a href="/" class="no-underline text-inherit">
            <span class="text-xl font-bold tracking-tight">kioku</span>
          </a>
          <HeaderControls />
        </div>
      </header>

      <main class="pt-8 md:pt-10">
        <section class="max-w-7xl mx-auto px-6 md:px-8 mb-14 md:mb-20">
          <div class="grid lg:grid-cols-[1.05fr_1fr] items-center gap-12 lg:gap-20">
            <Reveal class="flex flex-col gap-6 md:gap-8 lg:pl-16 xl:pl-24">
              <h1 class="text-4xl md:text-6xl xl:text-7xl font-bold text-charcoal dark:text-white leading-[1.05] tracking-tight">
                {t("hero.title")}
              </h1>
              {isAuthenticated ? (
                <DashboardButton align="start" />
              ) : (
                <GoogleSignInButton align="start" />
              )}
            </Reveal>
            <Reveal
              as="ul"
              delay={150}
              class="flex flex-col gap-5 md:gap-6 list-disc pl-7 text-xl md:text-2xl font-bold text-charcoal dark:text-white marker:text-primary leading-snug"
            >
              <li>{t("hero.feature1")}</li>
              <li>{t("hero.feature2")}</li>
              <li>{t("hero.feature3")}</li>
            </Reveal>
          </div>
        </section>

        <section class="max-w-7xl mx-auto px-6 md:px-8 mb-14 md:mb-20">
          <div class="lg:pl-16 xl:pl-24">
            <Reveal class="max-w-2xl border-l-4 border-primary pl-6 md:pl-10 py-4 md:py-6">
              <h2 class="text-2xl md:text-4xl font-bold text-charcoal dark:text-white mb-6 md:mb-8 leading-tight">
                {t("statement.heading")}
              </h2>
              <div class="space-y-4 md:space-y-8 text-base md:text-xl text-taupe dark:text-text-muted-dark leading-relaxed">
                <p>{t("statement.body1")}</p>
                <p class="text-xl md:text-3xl font-bold text-charcoal dark:text-white leading-snug tracking-tight">
                  {t("statement.body2")}
                </p>
              </div>
            </Reveal>
          </div>
        </section>

        <section class="max-w-7xl mx-auto px-6 md:px-8 overflow-x-clip mb-14 md:mb-20 pb-16 md:pb-32">
          <div class="lg:pl-16 xl:pl-24 flex flex-col gap-12 md:gap-20">
            {principles.map((key, idx) => {
              const num = String(idx + 1).padStart(2, "0");

              const blurClass =
                key === "synthesis"
                  ? "bg-primary/20"
                  : key === "library"
                    ? "bg-primary/15"
                    : "bg-primary/20";
              const mockNode =
                key === "synthesis" ? (
                  <ExplorerMock t={t} />
                ) : key === "library" ? (
                  <ChatMock t={t} />
                ) : (
                  <PodcastMock t={t} />
                );
              const mockBlock = (
                <div class={`relative w-full max-w-2xl ${key === "library" ? "mr-auto" : "mx-auto"}`}>
                  <div
                    aria-hidden="true"
                    class="pointer-events-none absolute inset-0 flex items-center justify-center -z-10"
                  >
                    <div
                      class={`w-[120%] max-w-[750px] aspect-[5/3] rounded-[100%] blur-3xl ${blurClass}`}
                    />
                  </div>
                  <div class="relative w-full">{mockNode}</div>
                </div>
              );

              if (key === "library") {
                return (
                  <Reveal
                    key={key}
                    class="md:grid md:grid-cols-12 md:items-start md:gap-y-8 flex flex-col gap-8 md:mt-32 md:mb-16"
                  >
                    <div class="md:col-start-2 md:col-span-6 md:row-start-1 space-y-4 md:space-y-6 md:text-right order-2 md:order-1">
                      <h3 class="text-2xl md:text-4xl font-bold text-charcoal dark:text-white leading-tight">
                        {t(`features.${key}.title`)}
                      </h3>
                      <p class="text-base md:text-lg text-taupe dark:text-text-muted-dark leading-relaxed md:ml-auto md:max-w-md">
                        {t(`features.${key}.body`)}
                      </p>
                    </div>
                    <div class="md:col-start-9 md:col-span-2 md:row-start-1 order-1 md:order-2">
                      <span class="block text-6xl sm:text-7xl md:text-9xl font-bold text-primary tracking-tighter leading-none select-none">
                        {num}
                      </span>
                    </div>
                    <div class="md:col-start-1 md:col-span-9 md:row-start-2 order-3">
                      {mockBlock}
                    </div>
                  </Reveal>
                );
              }
              return (
                <Reveal
                  key={key}
                  class="md:grid md:grid-cols-12 md:items-start md:gap-y-8 flex flex-col gap-8"
                >
                  <div class="md:col-start-1 md:col-span-2 md:row-start-1 order-1">
                    <span class="block text-6xl sm:text-7xl md:text-9xl font-bold text-primary tracking-tighter leading-none select-none">
                      {num}
                    </span>
                  </div>
                  <div
                    class={`md:col-span-6 md:row-start-1 space-y-4 md:space-y-6 order-2 ${
                      idx === 0 ? "md:col-start-4" : "md:col-start-5"
                    }`}
                  >
                    <h3 class="text-2xl md:text-4xl font-bold text-charcoal dark:text-white leading-tight">
                      {t(`features.${key}.title`)}
                    </h3>
                    <p class="text-base md:text-lg text-taupe dark:text-text-muted-dark leading-relaxed md:max-w-md">
                      {t(`features.${key}.body`)}
                    </p>
                  </div>
                  <div class="md:col-start-4 md:col-span-9 md:row-start-2 order-3">
                    {mockBlock}
                  </div>
                </Reveal>
              );
            })}
          </div>
        </section>

        <section class="bg-card-light dark:bg-card-dark px-6 md:px-8 py-14 md:py-20">
          <div class="text-center max-w-5xl mx-auto">
            <div class="space-y-8 md:space-y-10">
              <p class="whitespace-pre-line text-xl md:text-3xl font-bold text-charcoal dark:text-white max-w-2xl mx-auto leading-snug">
                <Trans
                  i18nKey="cta.description"
                  components={{ b: <strong class="font-extrabold text-primary" /> }}
                />
              </p>
              <div class="flex justify-center">
                {isAuthenticated ? (
                  <DashboardButton align="center" />
                ) : (
                  <GoogleSignInButton align="center" />
                )}
              </div>
            </div>
          </div>
        </section>

        <footer class="border-t border-border-light dark:border-border-dark">
          <div class="max-w-7xl mx-auto px-6 md:px-8 py-6 flex flex-col md:flex-row justify-between items-center gap-4 text-sm text-taupe dark:text-text-muted-dark">
            <div class="flex items-center gap-6">
              <a
                href="/privacy"
                class="font-medium no-underline text-inherit hover:text-charcoal dark:hover:text-white"
              >
                {t("footer.privacy")}
              </a>
              <a
                href="/tos"
                class="font-medium no-underline text-inherit hover:text-charcoal dark:hover:text-white"
              >
                {t("footer.terms")}
              </a>
            </div>
            <span>{t("footer.copyright")}</span>
          </div>
        </footer>
      </main>
    </div>
  );
}
