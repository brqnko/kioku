import useSWRInfinite from "swr/infinite";
import { kyInstance } from "../api/mutator";
import type { ListChats200 } from "../api/generated/backend.schemas";

const PAGE_SIZE = 16;
type ChatCursor = NonNullable<ListChats200["next_cursor"]>;

export function useChats(projectId: string | undefined) {
  const getKey = (
    pageIndex: number,
    prev: ListChats200 | null,
  ): readonly [string, string, ChatCursor | null] | null => {
    if (!projectId) return null;
    if (prev && !prev.next_cursor) return null;
    if (pageIndex === 0) return ["project-chats", projectId, null] as const;
    return ["project-chats", projectId, prev!.next_cursor!] as const;
  };

  const fetcher = async ([, id, cursor]: readonly [
    string,
    string,
    ChatCursor | null,
  ]) => {
    const searchParams: Record<string, string | number> = { limit: PAGE_SIZE };
    if (cursor) {
      searchParams.cursor_last_activity_at = cursor.last_activity_at;
      searchParams.cursor_chat_id = cursor.chat_id;
    }
    return kyInstance
      .get(`projects/${id}/chats`, { searchParams })
      .json<ListChats200>();
  };

  const { data, error, isLoading, isValidating, size, setSize, mutate } =
    useSWRInfinite<ListChats200>(getKey, fetcher);

  const pages = data ?? [];
  const items = pages.flatMap((p) => p.items);
  const hasMore =
    pages.length > 0 ? Boolean(pages[pages.length - 1]?.next_cursor) : false;
  const loadingMore = isValidating && pages.length > 0 && pages.length < size;
  const loadMore = () => setSize((s) => s + 1);
  const refresh = async (): Promise<void> => {
    await mutate();
  };

  return { items, error, isLoading, hasMore, loadingMore, loadMore, refresh };
}
