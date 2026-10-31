import { parseSession } from "~/helpers/session";

import { ApiError, apiGet, apiPost } from "./client";

// Trades the one-time token of a login link for a session cookie.
function apiLogin(token: string): Promise<void> {
  return apiPost("/api/session", { token }, () => undefined);
}

function apiSession(): Promise<{ readonly admin: string }> {
  return apiGet("/api/session", parseSession);
}

export type LoginState = "signed-in" | "signed-out" | "link-refused";

// With a token, opens a session from it; without, checks for a session already open.
export async function loginState(token: string | null): Promise<LoginState> {
  try {
    await (token ? apiLogin(token) : apiSession());
    return "signed-in";
  } catch (error) {
    if (error instanceof ApiError && error.status === 401) {
      return token ? "link-refused" : "signed-out";
    }
    throw error;
  }
}
