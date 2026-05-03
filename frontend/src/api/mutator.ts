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

export const kyInstance = ky.create({
  prefixUrl: "/api",
  credentials: "include",
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
