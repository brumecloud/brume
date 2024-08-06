import { Octokit } from "octokit";
import { useCallback, useEffect, useState } from "react";
import { Virtuoso } from "react-virtuoso";
import { toast } from "sonner";

import { LogLine } from "./logs-body.comp";

export const github_token = "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";
const octokit = new Octokit({
  auth: github_token,
});

export type GitHubEvent = {
  id: string;
  type: string;
  actor: {
    id: number;
    login: string;
    display_login: string;
    gravatar_id: string;
    url: string;
    avatar_url: string;
  };
  repo: {
    id: number;
    name: string;
    url: string;
  };
  payload: {
    repository_id: number;
    push_id: number;
    size: number;
    distinct_size: number;
    ref: string;
    head: string;
    description: string;
    before: string;
    commits: Array<{
      sha: string;
      author: {
        email: string;
        name: string;
      };
      message: string;
      distinct: boolean;
      url: string;
    }>;
    pull_request: {
      body: string;
    };
    issue: {
      body: string;
    };
  };
  public: boolean;
  created_at: string;
  org: {
    id: number;
    login: string;
    gravatar_id: string;
    url: string;
    avatar_url: string;
  };
};

export const LogsRender = () => {
  const [logs, setLogs] = useState<GitHubEvent[]>([]);
  const [lastPage, setLastPage] = useState(1);

  const loadMoreLogs = useCallback(async () => {
    // try {
    //   const newLogs = await octokit.request(`GET /events?per_page=100&page=${lastPage}`, {
    //     headers: {
    //       "X-GitHub-Api-Version": "2022-11-28",
    //     },
    //   });
    //   setLogs((oldLogs) => [...oldLogs, ...newLogs.data]);
    //   setLastPage((oldPage) => oldPage + 1);
    // } catch (e) {
    //   console.error({ ...e });
    //   toast.error(`Loading error (status ${e.response.status})`, {
    //     description: e.response.data.message as string,
    //   });
    // }
  }, [setLogs, lastPage]);

  // useEffect(() => {
  //   loadMoreLogs();
  // }, []);

  return (
    <div className="h-full max-h-full cursor-text overflow-x-auto font-mono">
      <div className="h-full w-full">
        <Virtuoso
          style={{ height: "100%" }}
          data={logs}
          endReached={loadMoreLogs}
          itemContent={(_, data) => <LogLine data={data} />}
        />
      </div>
    </div>
  );
};
