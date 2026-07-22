ALTER TABLE deployments ADD COLUMN public_label text;

UPDATE deployments
SET public_label = deployments.slug || '-' || users.handle
FROM users
WHERE users.id = deployments.user_id;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM deployments
        WHERE public_label IS NULL
           OR length(public_label) > 63
           OR public_label !~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$'
    ) THEN
        RAISE EXCEPTION 'existing deployment produces an invalid wildcard DNS label';
    END IF;

    IF EXISTS (
        SELECT public_label
        FROM deployments
        GROUP BY public_label
        HAVING count(*) > 1
    ) THEN
        RAISE EXCEPTION 'existing deployments produce duplicate wildcard DNS labels';
    END IF;
END $$;

ALTER TABLE deployments ALTER COLUMN public_label SET NOT NULL;
ALTER TABLE deployments ADD CONSTRAINT deployments_public_label_unique UNIQUE (public_label);

CREATE TABLE token_families (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_type text NOT NULL CHECK (client_type IN ('cli', 'browser', 'development')),
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE access_tokens (
    id uuid PRIMARY KEY,
    family_id uuid NOT NULL REFERENCES token_families(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash bytea NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    last_used_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE refresh_tokens (
    id uuid PRIMARY KEY,
    family_id uuid NOT NULL REFERENCES token_families(id) ON DELETE CASCADE,
    token_hash bytea NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    consumed_at timestamptz,
    replaced_by uuid REFERENCES refresh_tokens(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE auth_tickets (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    family_id uuid NOT NULL REFERENCES token_families(id) ON DELETE CASCADE,
    ticket_hash bytea NOT NULL UNIQUE,
    return_to text NOT NULL,
    expires_at timestamptz NOT NULL,
    consumed_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX access_tokens_active_idx
    ON access_tokens(token_hash, expires_at)
    WHERE revoked_at IS NULL;
CREATE INDEX refresh_tokens_active_idx
    ON refresh_tokens(token_hash, expires_at)
    WHERE consumed_at IS NULL;
CREATE INDEX token_families_expiry_idx ON token_families(expires_at);
CREATE INDEX auth_tickets_active_idx ON auth_tickets(ticket_hash, expires_at)
    WHERE consumed_at IS NULL;
