import useSWR from "swr";
import useSWRInfinite from "swr/infinite";
import { kyInstance } from "../api/mutator";
import type {
  GetProject200,
  ListProjectChildren200,
} from "../api/generated/backend.schemas";

const PAGE_SIZE = 32;
const projectKey = (id: string) => `projects/${id}`;

type ChildCursor = NonNullable<ListProjectChildren200["next_cursor"]>;

export function useProject(projectId: string | undefined) {
  return useSWR<GetProject200>(
    projectId ? projectKey(projectId) : null,
    () => kyInstance.get(`projects/${projectId}`).json<GetProject200>(),
  );
}

export function useProjectChildren(projectId: string | undefined) {
  const getKey = (
    pageIndex: number,
    prev: ListProjectChildren200 | null,
  ): readonly [string, string, ChildCursor | null] | null => {
    if (!projectId) return null;
    if (prev && !prev.next_cursor) return null;
    if (pageIndex === 0)
      return ["project-children", projectId, null] as const;
    return ["project-children", projectId, prev!.next_cursor!] as const;
  };

  const fetcher = async (
    [, id, cursor]: readonly [string, string, ChildCursor | null],
  ) => {
    const searchParams: Record<string, string | number> = { limit: PAGE_SIZE };
    if (cursor) {
      searchParams.cursor_phase = cursor.phase;
      searchParams.cursor_name = cursor.name;
      searchParams.cursor_id = cursor.id;
    }
    return kyInstance
      .get(`projects/${id}/children`, { searchParams })
      .json<ListProjectChildren200>();
  };

  const { data, error, isLoading, isValidating, size, setSize, mutate } =
    useSWRInfinite<ListProjectChildren200>(getKey, fetcher);

  const pages = data ?? [];
  const items = pages.flatMap((p) => p.items);
  const hasMore = pages.length > 0
    ? Boolean(pages[pages.length - 1]?.next_cursor)
    : false;
  const loadingMore = isValidating && pages.length > 0 && pages.length < size;

  const loadMore = () => setSize((s) => s + 1);
  const refresh = async (): Promise<void> => {
    await mutate();
  };

  return {
    items,
    error,
    isLoading,
    hasMore,
    loadingMore,
    loadMore,
    refresh,
  };
}
