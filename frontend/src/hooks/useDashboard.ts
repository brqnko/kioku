import useSWR from "swr";
import { kyInstance } from "../api/mutator";
import type { GetDashboard200 } from "../api/generated/backend.schemas";

const DASHBOARD_KEY = "users/me/dashboard";

const fetcher = (path: string) => kyInstance.get(path).json<GetDashboard200>();

export function useDashboard() {
  return useSWR<GetDashboard200>(DASHBOARD_KEY, fetcher);
}
