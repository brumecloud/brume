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

export async function apiGet<T>(path: `/api/${string}`, decode: (raw: unknown) => T): Promise<T> {
  const response = await fetch(path, {
    headers: { "X-Brume": "1", "Content-Type": "application/json" },
  });
  const body: unknown = await response.json().catch(() => undefined);
  if (!response.ok) {
    throw new ApiError(response.status, apiErrorMessage(response.status, body));
  }
  return decode(body);
}
