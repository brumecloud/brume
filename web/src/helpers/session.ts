import { decodeRecord, readString } from "./decode";

// `brume dev` prints `…/login#token=<base64url>`: the hash never reaches a server log.
const LOGIN_TOKEN = /^token=([A-Za-z0-9_-]{16,128})$/;

const SESSION_PATH = "/api/session";

export function parseLoginHash(hash: string): string | null {
  return LOGIN_TOKEN.exec(hash.replace(/^#/, ""))?.[1] ?? null;
}

// The session routes answer 401 to say "not signed in": the login page handles it.
export function redirectsToLogin(status: number, path: string): boolean {
  return status === 401 && path !== SESSION_PATH;
}

export function parseSession(raw: unknown): { readonly admin: string } {
  return { admin: readString(decodeRecord(raw, "session"), "admin", "session") };
}
