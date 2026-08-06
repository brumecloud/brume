export type ReviewStatus = "open" | "finished" | "superseded";

export interface ReviewRoundState {
  id: string;
  number: number;
  status: ReviewStatus;
  thread_count: number;
  comment_count: number;
}

export type AuthMode = "token" | "password" | "none";

export interface OverlayState {
  enabled: boolean;
  owner: boolean;
  identified: boolean;
  /** Present only for the owner: how the site is currently shared. */
  auth_mode?: AuthMode;
  review?: ReviewRoundState;
}

export interface ReviewComment {
  id: string;
  author_public_id: string | null;
  author_display_name: string | null;
  author_is_owner: boolean;
  body: string;
  created_at: string;
  edited_at: string | null;
}

export interface ReviewThread {
  id: string;
  page_path: string;
  anchor: unknown;
  created_at: string;
  comments: ReviewComment[];
}

export interface ThreadsResponse {
  round: ReviewRoundState | null;
  viewer_public_id: string | null;
  threads: ReviewThread[];
}

export class ApiRequestError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}

async function parseJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    let message = `Request failed (${response.status})`;
    try {
      const body = (await response.json()) as { message?: string };
      if (body.message) message = body.message;
    } catch {
      /* keep the generic message */
    }
    throw new ApiRequestError(message, response.status);
  }
  return (await response.json()) as T;
}

export interface GrantSummary {
  public_id: string;
  display_name: string | null;
  granted_at: string;
  last_used_at: string | null;
}

export interface ToolbarAccessState {
  auth_mode: AuthMode;
  has_password: boolean;
  grants: GrantSummary[];
}

export interface InvitationResponse {
  url: string;
  expires_at: string;
}

/**
 * Cross-origin client for the owner-only management endpoints on the auth
 * host. The browser attaches the auth-origin session cookie itself; the
 * server validates the request Origin against the site this control serves.
 */
export class ToolbarApi {
  constructor(
    readonly site: string,
    readonly authOrigin: string,
  ) {}

  private url(path: string): string {
    return this.authOrigin + "/toolbar/" + encodeURIComponent(this.site) + path;
  }

  private async post<T>(path: string, body: unknown): Promise<T> {
    const response = await fetch(this.url(path), {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    return parseJson<T>(response);
  }

  async ownerState(): Promise<{ owner: boolean }> {
    const response = await fetch(this.url("/owner-state"), { credentials: "include" });
    return parseJson(response);
  }

  async access(): Promise<ToolbarAccessState> {
    const response = await fetch(this.url("/access"), { credentials: "include" });
    return parseJson(response);
  }

  async updateSettings(auth: AuthMode, password?: string): Promise<ToolbarAccessState> {
    return this.post("/settings", { auth, password });
  }

  async createInvitation(returnTo: string): Promise<InvitationResponse> {
    return this.post("/invitations", { return_to: returnTo });
  }

  async revokeGrant(publicId: string): Promise<{ grants: GrantSummary[] }> {
    return this.post("/grants/revoke", { public_id: publicId });
  }
}

export async function fetchOverlayState(site: string): Promise<OverlayState> {
  const url =
    "/_brume/overlay-state?site=" +
    encodeURIComponent(site) +
    "&return_to=" +
    encodeURIComponent(location.href);
  const response = await fetch(url, { credentials: "same-origin" });
  return parseJson<OverlayState>(response);
}

export class ReviewApi {
  constructor(private readonly site: string) {}

  private url(path: string): string {
    return (
      path +
      "?site=" +
      encodeURIComponent(this.site) +
      "&return_to=" +
      encodeURIComponent(location.href)
    );
  }

  async threads(): Promise<ThreadsResponse> {
    const response = await fetch(this.url("/_brume/review/threads"), {
      credentials: "same-origin",
    });
    return parseJson<ThreadsResponse>(response);
  }

  async createThread(
    pagePath: string,
    anchor: unknown,
    body: string,
    displayName?: string,
  ): Promise<{ thread_id: string; comment_id: string }> {
    const response = await fetch(this.url("/_brume/review/threads"), {
      method: "POST",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        page_path: pagePath,
        anchor,
        body,
        display_name: displayName,
      }),
    });
    return parseJson(response);
  }

  async reply(
    threadId: string,
    body: string,
    displayName?: string,
  ): Promise<{ comment_id: string }> {
    const response = await fetch(
      this.url("/_brume/review/threads/" + encodeURIComponent(threadId) + "/comments"),
      {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ body, display_name: displayName }),
      },
    );
    return parseJson(response);
  }

  async editComment(commentId: string, body: string): Promise<void> {
    const response = await fetch(
      this.url("/_brume/review/comments/" + encodeURIComponent(commentId)),
      {
        method: "PATCH",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ body }),
      },
    );
    if (!response.ok) await parseJson(response);
  }

  async deleteThread(threadId: string): Promise<void> {
    const response = await fetch(
      this.url("/_brume/review/threads/" + encodeURIComponent(threadId)),
      {
        method: "DELETE",
        credentials: "same-origin",
      },
    );
    if (!response.ok) await parseJson(response);
  }

  async deleteComment(commentId: string): Promise<void> {
    const response = await fetch(
      this.url("/_brume/review/comments/" + encodeURIComponent(commentId)),
      {
        method: "DELETE",
        credentials: "same-origin",
      },
    );
    if (!response.ok) await parseJson(response);
  }

  async finish(): Promise<void> {
    const response = await fetch(this.url("/_brume/review/finish"), {
      method: "POST",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json" },
    });
    if (!response.ok) {
      throw new Error(`Could not finish the review (${response.status})`);
    }
  }
}
