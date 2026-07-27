CREATE TABLE access_controls (
    id uuid PRIMARY KEY,
    owner_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    auth_mode text NOT NULL CHECK (auth_mode IN ('token', 'password', 'none')),
    password_hash text,
    access_version bigint NOT NULL DEFAULT 1,
    overlay_enabled boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CHECK (
        (auth_mode = 'password' AND password_hash IS NOT NULL)
        OR (auth_mode <> 'password' AND password_hash IS NULL)
    )
);

ALTER TABLE plans ADD COLUMN access_control_id uuid;
ALTER TABLE deployments ADD COLUMN access_control_id uuid;

UPDATE plans SET access_control_id = gen_random_uuid();
UPDATE deployments SET access_control_id = gen_random_uuid();

INSERT INTO access_controls (
    id, owner_user_id, auth_mode, overlay_enabled
)
SELECT
    access_control_id,
    user_id,
    CASE WHEN visibility = 'public' THEN 'none' ELSE 'token' END,
    true
FROM plans;

INSERT INTO access_controls (
    id, owner_user_id, auth_mode, overlay_enabled
)
SELECT access_control_id, user_id, 'none', true
FROM deployments;

ALTER TABLE plans ALTER COLUMN access_control_id SET NOT NULL;
ALTER TABLE deployments ALTER COLUMN access_control_id SET NOT NULL;
ALTER TABLE plans
    ADD CONSTRAINT plans_access_control_fk
    FOREIGN KEY (access_control_id) REFERENCES access_controls(id);
ALTER TABLE deployments
    ADD CONSTRAINT deployments_access_control_fk
    FOREIGN KEY (access_control_id) REFERENCES access_controls(id);
ALTER TABLE plans ADD CONSTRAINT plans_access_control_unique UNIQUE (access_control_id);
ALTER TABLE deployments ADD CONSTRAINT deployments_access_control_unique UNIQUE (access_control_id);

ALTER TABLE plans DROP COLUMN visibility;
ALTER TABLE plans DROP COLUMN unlisted_token;
ALTER TABLE plans DROP COLUMN unlisted_token_hash;

CREATE TABLE access_identities (
    id uuid PRIMARY KEY,
    public_id text NOT NULL UNIQUE,
    token_hash bytea NOT NULL UNIQUE,
    revoked_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz
);

CREATE TABLE access_grants (
    access_control_id uuid NOT NULL REFERENCES access_controls(id) ON DELETE CASCADE,
    identity_id uuid NOT NULL REFERENCES access_identities(id) ON DELETE CASCADE,
    permission text NOT NULL DEFAULT 'view' CHECK (permission IN ('view')),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (access_control_id, identity_id)
);

CREATE TABLE access_invitations (
    id uuid PRIMARY KEY,
    access_control_id uuid NOT NULL REFERENCES access_controls(id) ON DELETE CASCADE,
    token_hash bytea NOT NULL UNIQUE,
    return_to text NOT NULL,
    expires_at timestamptz NOT NULL,
    consumed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE site_sessions (
    id uuid PRIMARY KEY,
    access_control_id uuid NOT NULL REFERENCES access_controls(id) ON DELETE CASCADE,
    identity_id uuid REFERENCES access_identities(id) ON DELETE CASCADE,
    subject_kind text NOT NULL CHECK (subject_kind IN ('owner', 'token', 'password')),
    session_hash bytea NOT NULL UNIQUE,
    access_version bigint NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE site_auth_tickets (
    id uuid PRIMARY KEY,
    access_control_id uuid NOT NULL REFERENCES access_controls(id) ON DELETE CASCADE,
    identity_id uuid REFERENCES access_identities(id) ON DELETE CASCADE,
    subject_kind text NOT NULL CHECK (subject_kind IN ('owner', 'token', 'password')),
    ticket_hash bytea NOT NULL UNIQUE,
    completion_origin text NOT NULL,
    return_to text NOT NULL,
    cookie_path text NOT NULL,
    expires_at timestamptz NOT NULL,
    consumed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE password_attempts (
    access_control_id uuid NOT NULL REFERENCES access_controls(id) ON DELETE CASCADE,
    client_hash bytea NOT NULL,
    failures integer NOT NULL DEFAULT 0,
    window_started_at timestamptz NOT NULL DEFAULT now(),
    blocked_until timestamptz,
    PRIMARY KEY (access_control_id, client_hash)
);

CREATE TABLE toolbar_action_tokens (
    id uuid PRIMARY KEY,
    access_control_id uuid NOT NULL REFERENCES access_controls(id) ON DELETE CASCADE,
    owner_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash bytea NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE oauth_states ADD COLUMN site_access_control_id uuid REFERENCES access_controls(id);
ALTER TABLE oauth_states ADD COLUMN site_return_to text;

CREATE INDEX access_grants_identity_idx ON access_grants(identity_id);
CREATE INDEX access_invitations_active_idx
    ON access_invitations(token_hash, expires_at)
    WHERE consumed_at IS NULL;
CREATE INDEX site_sessions_active_idx
    ON site_sessions(session_hash, expires_at);
CREATE INDEX site_auth_tickets_active_idx
    ON site_auth_tickets(ticket_hash, expires_at)
    WHERE consumed_at IS NULL;
