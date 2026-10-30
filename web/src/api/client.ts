import { apiErrorMessage } from "~/helpers/api-error";

// The only code that talks to the server. See archi/base/04-panel-admin.md (CSRF).

class ApiError extends Error {
  override name = "ApiError";
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

type ApiPath = `/api/${string}`;

export function apiGet<T>(path: ApiPath, decode: (raw: unknown) => T): Promise<T> {
  return request("GET", path, undefined, decode);
}

export function apiPost<T>(path: ApiPath, body: unknown, decode: (raw: unknown) => T): Promise<T> {
  return request("POST", path, body, decode);
}

export function apiDelete(path: ApiPath): Promise<void> {
  return request("DELETE", path, undefined, () => undefined);
}

async function request<T>(
  method: "GET" | "POST" | "DELETE",
  path: ApiPath,
  body: unknown,
  decode: (raw: unknown) => T,
): Promise<T> {
  const response = await fetch(path, {
    method,
    headers: { "X-Brume": "1", "Content-Type": "application/json" },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
  });
  const payload: unknown = await response.json().catch(() => undefined);
  if (!response.ok) {
    throw new ApiError(response.status, apiErrorMessage(response.status, payload));
  }
  return decode(payload);
}
