import useSWR from "swr";
import useSWRInfinite from "swr/infinite";
import { kyInstance } from "../api/mutator";
import type {
  GetPodcast200,
  ListPodcasts200,
} from "../api/generated/backend.schemas";

export function usePodcast(
  projectId: string | undefined,
  podcastId: string | undefined,
) {
  return useSWR<GetPodcast200>(
    projectId && podcastId
      ? `projects/${projectId}/podcasts/${podcastId}`
      : null,
    () =>
      kyInstance
        .get(`projects/${projectId}/podcasts/${podcastId}`)
        .json<GetPodcast200>(),
    { revalidateOnFocus: false },
  );
}

const PAGE_SIZE = 12;

type Cursor = NonNullable<ListPodcasts200["next_cursor"]>;

export function usePodcasts(projectId: string | undefined) {
  const getKey = (
    pageIndex: number,
    prev: ListPodcasts200 | null,
  ): readonly [string, string, Cursor | null] | null => {
    if (!projectId) return null;
    if (prev && !prev.next_cursor) return null;
    if (pageIndex === 0)
      return ["project-podcasts", projectId, null] as const;
    return ["project-podcasts", projectId, prev!.next_cursor!] as const;
  };

  const fetcher = async (
    [, id, cursor]: readonly [string, string, Cursor | null],
  ) => {
    const searchParams: Record<string, string | number> = { limit: PAGE_SIZE };
    if (cursor) {
      searchParams.cursor_created_at = cursor.created_at;
      searchParams.cursor_podcast_id = cursor.podcast_id;
    }
    return kyInstance
      .get(`projects/${id}/podcasts`, { searchParams })
      .json<ListPodcasts200>();
  };

  const { data, error, isLoading, isValidating, size, setSize, mutate } =
    useSWRInfinite<ListPodcasts200>(getKey, fetcher);

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
