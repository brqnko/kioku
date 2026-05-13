import useSWR from "swr";
import { kyInstance } from "../api/mutator";
import type {
  ListCompilers200,
  ListCompilers200CompilersItem,
} from "../api/generated/backend.schemas";

export type Compiler = ListCompilers200CompilersItem;

const COMPILERS_KEY = "code/compilers";

export function useCompilers() {
  return useSWR<Compiler[]>(
    COMPILERS_KEY,
    async () => {
      const data = await kyInstance.get(COMPILERS_KEY).json<ListCompilers200>();
      return data.compilers;
    },
    {
      revalidateOnFocus: false,
      dedupingInterval: 24 * 60 * 60 * 1000,
    },
  );
}
