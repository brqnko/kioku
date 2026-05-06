import useSWR from "swr";
import { kyInstance } from "../api/mutator";
import type { GetUserProfile200 } from "../api/generated/backend.schemas";

const PROFILE_KEY = "users/me";

const fetcher = (path: string) =>
  kyInstance.get(path).json<GetUserProfile200>();

export function useAuth() {
  const { data, error, isLoading } = useSWR<GetUserProfile200>(
    PROFILE_KEY,
    fetcher,
    { shouldRetryOnError: false },
  );
  return {
    user: data,
    isAuthenticated: !!data && !error,
    isLoading,
  };
}
