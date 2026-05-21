import useSWR from "swr";
import { kyInstance } from "../api/mutator";
import { fileContentKey } from "../api/keys";
import type { GetFileContent200 } from "../api/generated/backend.schemas";

export const fileContentKeyFor = fileContentKey;

export function useFileContent(fileId: string | undefined) {
  return useSWR<GetFileContent200>(
    fileId ? fileContentKey(fileId) : null,
    () =>
      kyInstance.get(`files/${fileId}/content`).json<GetFileContent200>(),
  );
}
