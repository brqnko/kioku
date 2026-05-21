import useSWR from "swr";
import useSWRInfinite from "swr/infinite";
import { kyInstance } from "../api/mutator";
import { folderKey } from "../api/keys";
import type {
  GetFileAncestors200,
  GetFolder200,
  GetFolderAncestors200,
  ListFolderChildren200,
} from "../api/generated/backend.schemas";

const PAGE_SIZE = 32;

type FolderChildCursor = NonNullable<ListFolderChildren200["next_cursor"]>;

export function useFolder(folderId: string | undefined) {
  return useSWR<GetFolder200>(
    folderId ? folderKey(folderId) : null,
    () => kyInstance.get(`folders/${folderId}`).json<GetFolder200>(),
  );
}

export function useFolderChildren(folderId: string | undefined) {
  const getKey = (
    pageIndex: number,
    prev: ListFolderChildren200 | null,
  ): readonly [string, string, FolderChildCursor | null] | null => {
    if (!folderId) return null;
    if (prev && !prev.next_cursor) return null;
    if (pageIndex === 0) return ["folder-children", folderId, null] as const;
    return ["folder-children", folderId, prev!.next_cursor!] as const;
  };

  const fetcher = async (
    [, id, cursor]: readonly [string, string, FolderChildCursor | null],
  ) => {
    const searchParams: Record<string, string | number> = { limit: PAGE_SIZE };
    if (cursor) {
      searchParams.cursor_phase = cursor.phase;
      searchParams.cursor_name = cursor.name;
      searchParams.cursor_id = cursor.id;
    }
    return kyInstance
      .get(`folders/${id}/children`, { searchParams })
      .json<ListFolderChildren200>();
  };

  const { data, error, isLoading, isValidating, size, setSize, mutate } =
    useSWRInfinite<ListFolderChildren200>(getKey, fetcher);

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

export interface BreadcrumbAncestor {
  kind: "project" | "folder";
  id: string;
  name: string;
}

export async function fetchFolderAncestors(
  folderId: string,
): Promise<BreadcrumbAncestor[]> {
  const res = await kyInstance
    .get(`folders/${folderId}/ancestors`)
    .json<GetFolderAncestors200>();
  return res.ancestors;
}

export async function fetchFileAncestors(
  fileId: string,
): Promise<BreadcrumbAncestor[]> {
  const res = await kyInstance
    .get(`files/${fileId}/ancestors`)
    .json<GetFileAncestors200>();
  return res.ancestors;
}
