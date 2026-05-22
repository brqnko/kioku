import { useState, useEffect, useRef } from "preact/hooks";
import { useRoute } from "preact-iso";
import { useTranslation } from "react-i18next";
import SideNavBar from "../components/SideNavBar";
import TopAppBar from "../components/TopAppBar";
import { Dialog } from "../components/Dialog";
import { RowActionMenu } from "../components/RowActionMenu";
import { MarkdownView } from "../components/MarkdownView";
import { useProject } from "../hooks/useProject";
import { useChats } from "../hooks/useChat";
import { useDocumentHead } from "../hooks/useDocumentHead";
import { kyInstance } from "../api/mutator";
import type {
  GetChat200,
  CreateChat200,
  SendMessage200,
} from "../api/generated/backend.schemas";

type LocalMessage = {
  role: "user" | "assistant";
  content: string;
  sent_at?: string;
  thinking?: boolean;
};

type ChatTarget = { id: string; name: string };

function ThinkingDots() {
  return (
    <div class="flex gap-1.5 items-center py-1">
      {[0, 150, 300].map((delay) => (
        <span
          key={delay}
          class="w-1.5 h-1.5 rounded-full bg-text-disabled animate-pulse"
          style={{ animationDelay: `${delay}ms` }}
        />
      ))}
    </div>
  );
}

function formatSentAt(sentAt: string | undefined, locale: string): string {
  if (!sentAt) return "";
  return new Date(sentAt).toLocaleTimeString(locale, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function MessageBubble({
  msg,
  locale,
  onCopy,
}: {
  msg: LocalMessage;
  locale: string;
  onCopy: (content: string) => void;
}) {
  const { t } = useTranslation();
  if (msg.thinking) {
    return (
      <div class="flex gap-3 max-w-3xl mx-auto w-full">
        <div class="w-7 h-7 rounded-full bg-surface-dark border border-border-subtle flex items-center justify-center shrink-0 mt-0.5">
          <span
            class="material-symbols-outlined text-[16px] text-text-primary"
            style={{ fontVariationSettings: "'FILL' 1" }}
          >
            smart_toy
          </span>
        </div>
        <ThinkingDots />
      </div>
    );
  }

  if (msg.role === "user") {
    return (
      <div class="flex gap-3 max-w-3xl mx-auto w-full flex-row-reverse">
        <div class="w-7 h-7 rounded-full bg-accent-blue flex items-center justify-center shrink-0 mt-0.5">
          <span class="text-white font-bold text-xs select-none">U</span>
        </div>
        <div class="flex flex-col items-end gap-1 max-w-xl min-w-0">
          <div class="bg-surface-dark border border-border-subtle px-4 py-2.5 rounded-xl rounded-tr-none text-sm text-text-primary whitespace-pre-wrap break-words">
            {msg.content}
          </div>
          <div class="flex items-center gap-1">
            {msg.sent_at && (
              <span class="text-[10px] text-text-disabled">
                {formatSentAt(msg.sent_at, locale)}
              </span>
            )}
            <button
              type="button"
              onClick={() => onCopy(msg.content)}
              title={t("projectChat.copy")}
              class="flex items-center justify-center w-7 h-7 rounded text-text-disabled hover:text-text-secondary hover:bg-overlay-faint cursor-pointer bg-transparent border-none"
            >
              <span class="material-symbols-outlined text-[15px]">
                content_copy
              </span>
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div class="flex gap-3 max-w-3xl mx-auto w-full">
      <div class="w-7 h-7 rounded-full bg-surface-dark border border-border-subtle flex items-center justify-center shrink-0 mt-0.5">
        <span
          class="material-symbols-outlined text-[16px] text-text-primary"
          style={{ fontVariationSettings: "'FILL' 1" }}
        >
          smart_toy
        </span>
      </div>
      <div class="flex flex-col gap-2 flex-1 min-w-0 pt-0.5">
        <MarkdownView
          source={msg.content}
          className="text-sm text-text-primary markdown-body leading-relaxed"
        />
        <div class="flex items-center gap-1 -ml-1">
          <button
            type="button"
            onClick={() => onCopy(msg.content)}
            title={t("projectChat.copy")}
            class="flex items-center justify-center w-7 h-7 rounded text-text-disabled hover:text-text-secondary hover:bg-overlay-faint cursor-pointer bg-transparent border-none"
          >
            <span class="material-symbols-outlined text-[15px]">
              content_copy
            </span>
          </button>
          {msg.sent_at && (
            <span class="text-[10px] text-text-disabled ml-1">
              {formatSentAt(msg.sent_at, locale)}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

export default function ProjectChatPage() {
  const { t, i18n } = useTranslation();
  useDocumentHead({
    title: "Project chat — kioku",
    robots: "noindex,nofollow",
  });
  const route = useRoute();
  const projectId = route.params.projectId as string;

  const { data: project } = useProject(projectId);
  const {
    items: chats,
    isLoading: chatsLoading,
    hasMore,
    loadMore,
    loadingMore,
    refresh: refreshChats,
  } = useChats(projectId);

  const [activeChatId, setActiveChatId] = useState<string | null>(null);
  const [messages, setMessages] = useState<LocalMessage[]>([]);
  const [chatLoading, setChatLoading] = useState(false);
  const [input, setInput] = useState("");
  const [sending, setSending] = useState(false);
  const [sendError, setSendError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  // rename dialog
  const [renamingChat, setRenamingChat] = useState<ChatTarget | null>(null);
  const [renameInput, setRenameInput] = useState("");
  const [renameSubmitting, setRenameSubmitting] = useState(false);

  // delete dialog
  const [deletingChat, setDeletingChat] = useState<ChatTarget | null>(null);
  const [deleteSubmitting, setDeleteSubmitting] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const renameInputRef = useRef<HTMLInputElement>(null);

  // Auto-select the most recently active chat once the list loads
  useEffect(() => {
    if (!activeChatId && !chatsLoading && chats.length > 0) {
      setActiveChatId(chats[0].id);
    }
  }, [chatsLoading, chats.length]); // eslint-disable-line react-hooks/exhaustive-deps

  // Load messages when the active chat changes
  useEffect(() => {
    if (!activeChatId || !projectId) {
      setMessages([]);
      return;
    }
    let cancelled = false;
    setChatLoading(true);
    setSendError(null);
    kyInstance
      .get(`projects/${projectId}/chats/${activeChatId}`)
      .json<GetChat200>()
      .then((data) => {
        if (!cancelled)
          setMessages(
            (
              data.messages as Array<
                (typeof data.messages)[0] & { sent_at?: string }
              >
            ).map((m) => ({
              role: m.role,
              content: m.content,
              sent_at: m.sent_at,
            })),
          );
      })
      .catch(() => {})
      .finally(() => {
        if (!cancelled) setChatLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [activeChatId, projectId]);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages.length]);

  // Focus rename input when dialog opens
  useEffect(() => {
    if (renamingChat) {
      setTimeout(() => renameInputRef.current?.focus(), 50);
    }
  }, [renamingChat]);

  const handleInputChange = (e: Event) => {
    const el = e.target as HTMLTextAreaElement;
    setInput(el.value);
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 128)}px`;
  };

  const handleCreateChat = async () => {
    if (creating || !projectId) return;
    setCreating(true);
    try {
      const date = new Date().toLocaleString(i18n.language, {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      });
      const newChat = await kyInstance
        .post(`projects/${projectId}/chats`, {
          json: { name: t("projectChat.newChatName", { date }) },
        })
        .json<CreateChat200>();
      await refreshChats();
      setActiveChatId(newChat.id);
    } catch {
      // silently ignore; user can retry
    } finally {
      setCreating(false);
    }
  };

  const handleRenameOpen = (chat: ChatTarget) => {
    setRenamingChat(chat);
    setRenameInput(chat.name);
  };

  const handleRenameSubmit = async () => {
    if (!renamingChat || !renameInput.trim() || renameSubmitting) return;
    setRenameSubmitting(true);
    try {
      await kyInstance.patch(`projects/${projectId}/chats/${renamingChat.id}`, {
        json: { name: renameInput.trim() },
      });
      await refreshChats();
      setRenamingChat(null);
    } catch {
      // keep dialog open; user can retry
    } finally {
      setRenameSubmitting(false);
    }
  };

  const handleDeleteOpen = (chat: ChatTarget) => {
    setDeletingChat(chat);
  };

  const handleDeleteConfirm = async () => {
    if (!deletingChat || deleteSubmitting) return;
    setDeleteSubmitting(true);
    try {
      await kyInstance.delete(`projects/${projectId}/chats/${deletingChat.id}`);
      if (activeChatId === deletingChat.id) {
        const remaining = chats.filter((c) => c.id !== deletingChat.id);
        setActiveChatId(remaining.length > 0 ? remaining[0].id : null);
        if (remaining.length === 0) setMessages([]);
      }
      await refreshChats();
      setDeletingChat(null);
    } catch {
      // keep dialog open; user can retry
    } finally {
      setDeleteSubmitting(false);
    }
  };

  const handleSend = async () => {
    if (!activeChatId || !input.trim() || sending) return;
    const content = input.trim();
    setInput("");
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
    setSending(true);
    setSendError(null);

    setMessages((prev) => [
      ...prev,
      { role: "user", content, sent_at: new Date().toISOString() },
      { role: "assistant", content: "", thinking: true },
    ]);

    try {
      const result = await kyInstance
        .post(`projects/${projectId}/chats/${activeChatId}/messages`, {
          json: { content },
        })
        .json<SendMessage200>();

      const am = result.assistant_message as typeof result.assistant_message & {
        sent_at?: string;
      };
      setMessages((prev) => [
        ...prev.filter((m) => !m.thinking),
        {
          role: am.role,
          content: am.content,
          sent_at: am.sent_at,
        },
      ]);
      refreshChats();
    } catch {
      setMessages((prev) => prev.filter((m) => !m.thinking));
      setSendError(t("projectChat.errors.send"));
    } finally {
      setSending(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleCopy = (content: string) => {
    navigator.clipboard.writeText(content).catch(() => {});
  };

  const activeChat = chats.find((c) => c.id === activeChatId);

  return (
    <div class="bg-background-dark text-text-primary overflow-hidden">
      <SideNavBar />
      <TopAppBar />

      <div class="ml-[var(--sidebar-width)] flex h-[calc(100vh-3.5rem)] overflow-hidden transition-[margin-left] duration-200 ease-in-out">
        {/* ── Sessions panel ── */}
        <aside
          class={`${activeChatId ? "hidden tablet:flex" : "flex"} w-full tablet:w-64 tablet:shrink-0 border-r border-border-subtle flex-col bg-surface-container-low overflow-hidden`}
        >
          <div class="shrink-0 p-3 border-b border-border-subtle">
            <button
              type="button"
              onClick={handleCreateChat}
              disabled={creating}
              class="btn-primary w-full"
            >
              {creating ? (
                <span>{t("projectChat.creating")}</span>
              ) : (
                <>
                  <span class="material-symbols-outlined text-[18px]">add</span>
                  {t("projectChat.newChat")}
                </>
              )}
            </button>
          </div>

          <div class="flex-1 overflow-y-auto p-2">
            {chatsLoading && chats.length === 0 && (
              <p class="text-xs text-text-disabled text-center px-2 py-4">
                {t("chat.loading")}
              </p>
            )}
            {!chatsLoading && chats.length === 0 && (
              <p class="text-xs text-text-disabled text-center px-2 py-4">
                {t("projectChat.sessionList.empty")}
              </p>
            )}

            <div class="flex flex-col gap-0.5">
              {chats.map((chat) => {
                const active = chat.id === activeChatId;
                return (
                  <div
                    key={chat.id}
                    class={`group flex items-center rounded-lg ${
                      active ? "bg-overlay-soft" : "hover:bg-overlay-faint"
                    }`}
                  >
                    <button
                      type="button"
                      onClick={() => setActiveChatId(chat.id)}
                      class={`flex-1 min-w-0 text-left pl-3 pr-1 py-2.5 text-sm cursor-pointer bg-transparent border-none ${
                        active
                          ? "text-text-primary font-medium"
                          : "text-text-secondary group-hover:text-text-primary"
                      }`}
                    >
                      <p class="truncate leading-snug">{chat.name}</p>
                      <p class="text-xs text-text-disabled mt-0.5">
                        {new Date(chat.last_activity_at).toLocaleDateString(
                          i18n.language,
                        )}
                      </p>
                    </button>
                    <div class="shrink-0 pr-1 opacity-0 group-hover:opacity-100">
                      <RowActionMenu
                        icon="more_vert"
                        ariaLabel={
                          t("renameItem.menu") + " / " + t("deleteItem.menu")
                        }
                        onEdit={() =>
                          handleRenameOpen({ id: chat.id, name: chat.name })
                        }
                        onDelete={() =>
                          handleDeleteOpen({ id: chat.id, name: chat.name })
                        }
                      />
                    </div>
                  </div>
                );
              })}
            </div>

            {hasMore && (
              <button
                type="button"
                onClick={loadMore}
                disabled={loadingMore}
                class="w-full mt-1 py-2 text-xs text-text-secondary hover:text-text-primary disabled:opacity-50 cursor-pointer bg-transparent border-none"
              >
                {t("chat.loadMore")}
              </button>
            )}
          </div>
        </aside>

        {/* ── Chat panel ── */}
        <section
          class={`${!activeChatId ? "hidden tablet:flex" : "flex"} flex-1 flex-col min-w-0 bg-background-dark overflow-hidden`}
        >
          {activeChatId ? (
            <>
              {/* Header */}
              <div class="shrink-0 h-12 border-b border-border-subtle flex items-center px-3 tablet:px-6 gap-2 overflow-hidden">
                <button
                  type="button"
                  onClick={() => setActiveChatId(null)}
                  aria-label={t("projectChat.back")}
                  class="tablet:hidden flex items-center justify-center w-8 h-8 rounded hover:bg-overlay-faint cursor-pointer bg-transparent border-none text-text-secondary shrink-0"
                >
                  <span class="material-symbols-outlined text-[20px]">
                    arrow_back
                  </span>
                </button>
                <a
                  href="/chat"
                  class="hidden tablet:inline text-xs text-text-disabled hover:text-text-secondary no-underline whitespace-nowrap shrink-0"
                >
                  {t("nav.chat")}
                </a>
                <span class="hidden tablet:inline material-symbols-outlined text-text-disabled text-[14px] select-none shrink-0">
                  chevron_right
                </span>
                <a
                  href={`/projects/${projectId}`}
                  class="text-xs text-text-secondary hover:text-text-primary no-underline truncate min-w-0"
                >
                  {project?.name ?? "…"}
                </a>
                {activeChat && (
                  <>
                    <span class="material-symbols-outlined text-text-disabled text-[14px] select-none shrink-0">
                      chevron_right
                    </span>
                    <span class="text-xs text-text-primary font-medium truncate">
                      {activeChat.name}
                    </span>
                  </>
                )}
              </div>

              {/* Messages */}
              <div class="flex-1 overflow-y-auto px-3 py-4 tablet:px-6 tablet:py-6 flex flex-col gap-6">
                {chatLoading ? (
                  <p class="text-sm text-text-disabled text-center py-8">
                    {t("chat.loading")}
                  </p>
                ) : messages.length === 0 ? (
                  <div class="flex flex-1 items-center justify-center">
                    <p class="text-sm text-text-disabled">
                      {t("projectChat.welcome.emptyChat")}
                    </p>
                  </div>
                ) : (
                  messages.map((msg, i) => (
                    <MessageBubble
                      key={i}
                      msg={msg}
                      locale={i18n.language}
                      onCopy={handleCopy}
                    />
                  ))
                )}
                <div ref={messagesEndRef} />
              </div>

              {/* Input area */}
              <div class="shrink-0 px-3 tablet:px-6 pb-4 pt-2">
                <div class="max-w-3xl mx-auto">
                  {sendError && (
                    <p class="text-xs text-danger mb-2 text-center">
                      {sendError}
                    </p>
                  )}
                  <div class="bg-surface-dark border border-border-subtle focus-within:border-accent-blue rounded-xl p-3 shadow-sm">
                    <textarea
                      ref={textareaRef}
                      value={input}
                      onInput={handleInputChange}
                      onKeyDown={handleKeyDown}
                      disabled={sending}
                      placeholder={t("projectChat.input.placeholder")}
                      rows={1}
                      class="w-full bg-transparent border-none outline-none resize-none text-sm text-text-primary placeholder:text-text-disabled p-0 disabled:opacity-60 leading-6"
                      style={{ minHeight: "1.5rem", maxHeight: "8rem" }}
                    />
                    <div class="flex items-center justify-end mt-2 pt-2 border-t border-border-subtle">
                      <button
                        type="button"
                        onClick={handleSend}
                        disabled={sending || !input.trim()}
                        class="btn-primary text-sm"
                      >
                        {sending
                          ? t("projectChat.input.sending")
                          : t("projectChat.input.send")}
                        <span class="material-symbols-outlined text-[18px]">
                          send
                        </span>
                      </button>
                    </div>
                  </div>
                  <p class="text-center mt-2 text-[10px] text-text-disabled leading-snug">
                    {t("projectChat.input.disclaimer")}
                  </p>
                </div>
              </div>
            </>
          ) : (
            /* Welcome / no-chat-selected screen */
            <div class="flex flex-1 flex-col items-center justify-center gap-6 p-8">
              <div class="w-14 h-14 rounded-full bg-surface-dark border border-border-subtle flex items-center justify-center">
                <span
                  class="material-symbols-outlined text-[28px] text-text-primary"
                  style={{ fontVariationSettings: "'FILL' 1" }}
                >
                  smart_toy
                </span>
              </div>
              <div class="text-center max-w-sm">
                <h2 class="heading-h2 mb-2">
                  {t("projectChat.welcome.title")}
                </h2>
                {project && (
                  <a
                    href={`/projects/${projectId}`}
                    class="text-sm text-text-disabled hover:text-text-secondary no-underline hover:underline mb-1 inline-block"
                  >
                    {project.name}
                  </a>
                )}
                <p class="text-sm text-text-secondary">
                  {t("projectChat.welcome.subtitle")}
                </p>
              </div>
              {!chatsLoading && (
                <button
                  type="button"
                  onClick={handleCreateChat}
                  disabled={creating}
                  class="btn-primary"
                >
                  <span class="material-symbols-outlined text-[18px]">add</span>
                  {creating
                    ? t("projectChat.creating")
                    : t("projectChat.welcome.cta")}
                </button>
              )}
            </div>
          )}
        </section>
      </div>

      {/* ── Rename dialog ── */}
      <Dialog
        open={!!renamingChat}
        onClose={() => setRenamingChat(null)}
        ariaLabel={t("renameItem.title")}
        maxWidth="max-w-[420px]"
      >
        <div class="p-6 flex flex-col gap-5">
          <h2 class="heading-h2">{t("renameItem.title")}</h2>
          <div class="flex flex-col gap-1.5">
            <label class="text-caption font-medium text-text-secondary">
              {t("renameItem.label")}
            </label>
            <input
              ref={renameInputRef}
              type="text"
              value={renameInput}
              onInput={(e) =>
                setRenameInput((e.target as HTMLInputElement).value)
              }
              onKeyDown={(e) => e.key === "Enter" && handleRenameSubmit()}
              class="input-field"
            />
          </div>
          <div class="flex justify-end gap-3">
            <button
              type="button"
              onClick={() => setRenamingChat(null)}
              class="btn-secondary"
            >
              {t("renameItem.cancel")}
            </button>
            <button
              type="button"
              onClick={handleRenameSubmit}
              disabled={renameSubmitting || !renameInput.trim()}
              class="btn-primary"
            >
              {renameSubmitting
                ? t("renameItem.submitting")
                : t("renameItem.submit")}
            </button>
          </div>
        </div>
      </Dialog>

      {/* ── Delete confirmation dialog ── */}
      <Dialog
        open={!!deletingChat}
        onClose={() => setDeletingChat(null)}
        ariaLabel={t("deleteItem.title")}
        maxWidth="max-w-[420px]"
      >
        <div class="p-6 flex flex-col gap-5">
          <h2 class="heading-h2">{t("deleteItem.title")}</h2>
          <p class="text-body text-text-secondary">
            {t("deleteItem.body", { name: deletingChat?.name ?? "" })}
          </p>
          <div class="flex justify-end gap-3">
            <button
              type="button"
              onClick={() => setDeletingChat(null)}
              class="btn-secondary"
            >
              {t("deleteItem.cancel")}
            </button>
            <button
              type="button"
              onClick={handleDeleteConfirm}
              disabled={deleteSubmitting}
              class="btn-danger"
            >
              {deleteSubmitting
                ? t("deleteItem.submitting")
                : t("deleteItem.submit")}
            </button>
          </div>
        </div>
      </Dialog>
    </div>
  );
}
