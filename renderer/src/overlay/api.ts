export type ReviewStatus = "open" | "finished" | "superseded";

export interface ReviewRoundState {
  id: string;
  number: number;
  status: ReviewStatus;
  thread_count: number;
  comment_count: number;
}

export interface OverlayState {
  enabled: boolean;
  owner: boolean;
  identified: boolean;
  review?: ReviewRoundState;
}

export interface ReviewComment {
  id: string;
  author_public_id: string | null;
  author_display_name: string | null;
  author_is_owner: boolean;
  body: string;
  created_at: string;
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
  threads: ReviewThread[];
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
    throw new Error(message);
  }
  return (await response.json()) as T;
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
