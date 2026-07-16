CREATE TABLE users (
    id uuid PRIMARY KEY,
    github_id bigint NOT NULL UNIQUE,
    github_login text NOT NULL,
    handle text NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE cli_login_sessions (
    id uuid PRIMARY KEY,
    poll_secret_hash bytea NOT NULL,
    user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    issued_token text,
    expires_at timestamptz NOT NULL,
    authorized_at timestamptz,
    consumed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE oauth_states (
    id uuid PRIMARY KEY,
    state_hash bytea NOT NULL UNIQUE,
    cli_session_id uuid REFERENCES cli_login_sessions(id) ON DELETE CASCADE,
    return_to text,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE api_tokens (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash bytea NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz,
    revoked_at timestamptz
);

CREATE TABLE web_sessions (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_hash bytea NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE plans (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    slug text NOT NULL,
    title text NOT NULL,
    visibility text NOT NULL CHECK (visibility IN ('private', 'unlisted', 'public')),
    active_bundle_id uuid,
    unlisted_token text,
    unlisted_token_hash bytea,
    published_at timestamptz NOT NULL DEFAULT now(),
    last_read_at timestamptz,
    pinned_at timestamptz,
    deletion_attempted_at timestamptz,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'deleting')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, slug)
);

CREATE TABLE plan_bundles (
    id uuid PRIMARY KEY,
    plan_id uuid NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    object_prefix text NOT NULL UNIQUE,
    renderer_version text NOT NULL,
    html_contract_version integer NOT NULL,
    manifest jsonb NOT NULL,
    status text NOT NULL CHECK (status IN ('active', 'superseded')),
    created_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE plans
    ADD CONSTRAINT plans_active_bundle_fk
    FOREIGN KEY (active_bundle_id) REFERENCES plan_bundles(id) ON DELETE SET NULL;

CREATE TABLE deletion_challenges (
    id uuid PRIMARY KEY,
    plan_id uuid NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    challenge_hash bytea NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX plans_retention_idx
    ON plans ((COALESCE(last_read_at, published_at)))
    WHERE pinned_at IS NULL;

CREATE INDEX api_tokens_active_idx ON api_tokens(token_hash) WHERE revoked_at IS NULL;
CREATE INDEX web_sessions_active_idx ON web_sessions(session_hash, expires_at);
