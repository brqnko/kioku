import type { ScopedMutator } from "swr";
import { unstable_serialize } from "swr/infinite";
import { LIBRARY_CACHE_KEY } from "../hooks/useLibrary";

const DASHBOARD_KEY = "users/me/dashboard";

function makeInfiniteKey(
  marker: string,
): string {
  return unstable_serialize(() => [marker, null] as const);
}

export interface MutationInvalidation {
  childListings?: boolean;
  library?: boolean;
  dashboard?: boolean;
}

export function invalidateAfterMutation(
  mutate: ScopedMutator,
  flags: MutationInvalidation,
): Promise<unknown> {
  const tasks: Promise<unknown>[] = [];
  if (flags.childListings) {
    tasks.push(mutate(makeInfiniteKey("project-children")));
    tasks.push(mutate(makeInfiniteKey("folder-children")));
  }
  if (flags.library) {
    tasks.push(mutate(LIBRARY_CACHE_KEY));
  }
  if (flags.dashboard) {
    tasks.push(mutate(DASHBOARD_KEY));
  }
  return Promise.all(tasks);
}
