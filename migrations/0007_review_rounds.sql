CREATE TABLE review_rounds (
    id uuid PRIMARY KEY,
    access_control_id uuid NOT NULL REFERENCES access_controls(id) ON DELETE CASCADE,
    round_number integer NOT NULL,
    status text NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'finished', 'superseded')),
    created_at timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz,
    UNIQUE (access_control_id, round_number)
);

CREATE UNIQUE INDEX review_rounds_one_open_idx
    ON review_rounds(access_control_id)
    WHERE status = 'open';

CREATE TABLE review_threads (
    id uuid PRIMARY KEY,
    round_id uuid NOT NULL REFERENCES review_rounds(id) ON DELETE CASCADE,
    page_path text NOT NULL,
    anchor jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX review_threads_round_idx ON review_threads(round_id, created_at);

CREATE TABLE review_comments (
    id uuid PRIMARY KEY,
    thread_id uuid NOT NULL REFERENCES review_threads(id) ON DELETE CASCADE,
    author_identity_id uuid REFERENCES access_identities(id) ON DELETE SET NULL,
    author_public_id text,
    author_display_name text,
    author_is_owner boolean NOT NULL DEFAULT false,
    body text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX review_comments_thread_idx ON review_comments(thread_id, created_at);
