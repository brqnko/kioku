import useSWR from "swr";
import { COMPILERS_KEY } from "../api/keys";
import type {
  ListCompilers200,
  ListCompilers200CompilersItem,
} from "../api/generated/backend.schemas";
import { kyInstance } from "../api/mutator";

export type Compiler = ListCompilers200CompilersItem;

export function useCompilers() {
  return useSWR<Compiler[]>(
    COMPILERS_KEY,
    async () => {
      const data = await kyInstance.get(COMPILERS_KEY).json<ListCompilers200>();
      return data.compilers;
    },
    {
      dedupingInterval: 24 * 60 * 60 * 1000,
      revalidateOnFocus: false,
    },
  );
}
