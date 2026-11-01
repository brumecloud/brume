import { apiErrorMessage } from "~/helpers/api-error";
import { redirectsToLogin } from "~/helpers/session";

// The only code that talks to the server. See archi/base/04-panel-admin.md (CSRF).

export class ApiError extends Error {
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

export function apiDelete(path: ApiPath): Promise<void>;
export function apiDelete<T>(path: ApiPath, decode: (raw: unknown) => T): Promise<T>;
export function apiDelete(path: ApiPath, decode: (raw: unknown) => unknown = () => undefined) {
  return request("DELETE", path, undefined, decode);
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
    if (redirectsToLogin(response.status, path)) {
      redirectToLogin();
    }
    throw new ApiError(response.status, apiErrorMessage(response.status, payload));
  }
  return decode(payload);
}

// Every query fails at once when the session expires: the first one leaves for the login page,
// which reloads the app and its caches.
let leaving = false;

export function redirectToLogin(): void {
  if (!leaving) {
    leaving = true;
    window.location.assign("/login");
  }
}
