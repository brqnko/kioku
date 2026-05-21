import useSWR from "swr";
import { kyInstance } from "../api/mutator";
import { DASHBOARD_KEY } from "../api/keys";
import type { GetDashboard200 } from "../api/generated/backend.schemas";

const fetcher = (path: string) => kyInstance.get(path).json<GetDashboard200>();

export function useDashboard() {
  return useSWR<GetDashboard200>(DASHBOARD_KEY, fetcher);
}
