import useSWR from "swr";
import { kyInstance } from "../api/mutator";
import type { GetFileContent200 } from "../api/generated/backend.schemas";

const fileContentKey = (id: string) => `files/${id}/content`;

export const fileContentKeyFor = fileContentKey;

export function useFileContent(fileId: string | undefined) {
  return useSWR<GetFileContent200>(
    fileId ? fileContentKey(fileId) : null,
    () =>
      kyInstance.get(`files/${fileId}/content`).json<GetFileContent200>(),
  );
}
