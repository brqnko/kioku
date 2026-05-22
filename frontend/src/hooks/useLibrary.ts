import useSWRInfinite, { unstable_serialize } from "swr/infinite";
import { kyInstance } from "../api/mutator";
import {
  ListProjectsOrder,
  type ListProjects200,
} from "../api/generated/backend.schemas";

const PAGE_SIZE = 32;

type Cursor = NonNullable<ListProjects200["next_cursor"]>;

function getKey(
  pageIndex: number,
  prevPageData: ListProjects200 | null,
): readonly [string, Cursor | null] | null {
  if (prevPageData && !prevPageData.next_cursor) return null;
  if (pageIndex === 0) return ["library:page", null] as const;
  return ["library:page", prevPageData!.next_cursor!] as const;
}

export const LIBRARY_CACHE_KEY = unstable_serialize(getKey);

const fetcher = async ([, cursor]: readonly [string, Cursor | null]) => {
  const searchParams: Record<string, string | number> = {
    order: ListProjectsOrder.last_seen_at_desc,
    limit: PAGE_SIZE,
  };
  if (cursor) {
    searchParams.cursor_last_seen_at = cursor.last_seen_at;
    searchParams.cursor_project_id = cursor.project_id;
  }
  return kyInstance.get("projects", { searchParams }).json<ListProjects200>();
};

export function useLibrary() {
  const { data, error, isLoading, isValidating, size, setSize, mutate } =
    useSWRInfinite<ListProjects200>(getKey, fetcher);

  const pages = data ?? [];
  const items = pages.flatMap((p) => p.items);
  const hasMore =
    pages.length > 0 ? Boolean(pages[pages.length - 1]?.next_cursor) : false;
  const loadingMore = isValidating && pages.length > 0 && pages.length < size;

  const loadMore = () => setSize((s) => s + 1);

  return {
    items,
    error,
    isLoading,
    hasMore,
    loadingMore,
    loadMore,
    mutate,
  };
}
