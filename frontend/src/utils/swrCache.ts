import type { ScopedMutator } from "swr";

const INFINITE_PREFIX = "$inf$";
const DASHBOARD_KEY = "users/me/dashboard";

function invalidateInfiniteByMarker(
  mutate: ScopedMutator,
  marker: string,
): Promise<unknown> {
  return mutate(
    (key) =>
      typeof key === "string" &&
      key.startsWith(INFINITE_PREFIX) &&
      key.includes(`"${marker}"`),
  );
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
    tasks.push(invalidateInfiniteByMarker(mutate, "project-children"));
    tasks.push(invalidateInfiniteByMarker(mutate, "folder-children"));
  }
  if (flags.library) {
    tasks.push(invalidateInfiniteByMarker(mutate, "library:page"));
  }
  if (flags.dashboard) {
    tasks.push(mutate(DASHBOARD_KEY));
  }
  return Promise.all(tasks);
}
