import ky from "ky";

interface RequestConfig {
  url: string;
  method: string;
  params?: Record<string, unknown>;
  data?: unknown;
  headers?: Record<string, string>;
  signal?: AbortSignal;
  responseType?: "json" | "blob" | "text";
}

const UNSAFE_METHODS = new Set(["POST", "PUT", "PATCH", "DELETE"]);
const RETRY_MARKER = "x-auth-retried";

function readCookie(name: string): string | undefined {
  if (typeof document === "undefined") return undefined;
  const match = document.cookie.match(
    new RegExp(`(?:^|; )${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}=([^;]*)`),
  );
  return match ? decodeURIComponent(match[1]) : undefined;
}

let refreshing: Promise<boolean> | null = null;

function refreshTokens(): Promise<boolean> {
  if (refreshing) return refreshing;
  refreshing = (async () => {
    try {
      const csrf = readCookie("csrf");
      const headers: Record<string, string> = {};
      if (csrf) headers["x-csrf-token"] = csrf;
      const res = await ky.post("api/auth/refresh", {
        prefixUrl: "",
        credentials: "include",
        throwHttpErrors: false,
        headers,
      });
      return res.ok;
    } catch {
      return false;
    } finally {
      refreshing = null;
    }
  })();
  return refreshing;
}

export const kyInstance = ky.create({
  prefixUrl: "/api",
  credentials: "include",
  hooks: {
    beforeRequest: [
      (request) => {
        if (UNSAFE_METHODS.has(request.method)) {
          if (!request.headers.has("x-csrf-token")) {
            const csrf = readCookie("csrf");
            if (csrf) request.headers.set("x-csrf-token", csrf);
          }
        }
      },
    ],
    afterResponse: [
      async (request, _options, response) => {
        if (response.status !== 401) return;
        if (request.url.includes("/auth/refresh")) return;
        if (request.headers.get(RETRY_MARKER)) return;

        const ok = await refreshTokens();
        if (!ok) {
          if (typeof window !== "undefined") {
            window.dispatchEvent(new CustomEvent("auth:unauthenticated"));
          }
          return;
        }

        const retryHeaders = new Headers(request.headers);
        retryHeaders.set(RETRY_MARKER, "1");
        const csrf = readCookie("csrf");
        if (csrf && UNSAFE_METHODS.has(request.method)) {
          retryHeaders.set("x-csrf-token", csrf);
        }
        return kyInstance(request.url, {
          method: request.method,
          headers: retryHeaders,
          body:
            request.method === "GET" || request.method === "HEAD"
              ? undefined
              : await request.clone().arrayBuffer(),
          prefixUrl: "",
        });
      },
    ],
  },
});

export const customInstance = <T>(config: RequestConfig): Promise<T> => {
  const { url, method, params, data, headers, signal, responseType } = config;

  const promise = kyInstance(url.replace(/^\//, ""), {
    method: method as string,
    ...(data !== undefined && { json: data }),
    ...(params && { searchParams: params as Record<string, string> }),
    headers,
    signal,
  }).then((res) => {
    if (responseType === "blob") return res.blob() as Promise<T>;
    if (responseType === "text") return res.text() as Promise<T>;
    return res.json<T>();
  });

  // @ts-expect-error -- orval expects cancel property on promise
  promise.cancel = () => {};

  return promise;
};
