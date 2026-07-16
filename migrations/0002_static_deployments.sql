CREATE TABLE deployments (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    slug text NOT NULL,
    spa boolean NOT NULL DEFAULT false,
    active_bundle_id uuid,
    published_at timestamptz NOT NULL DEFAULT now(),
    last_read_at timestamptz,
    pinned_at timestamptz,
    deletion_attempted_at timestamptz,
    status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'deleting')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, slug)
);

CREATE TABLE deployment_bundles (
    id uuid PRIMARY KEY,
    deployment_id uuid NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    object_prefix text NOT NULL UNIQUE,
    manifest jsonb NOT NULL,
    status text NOT NULL CHECK (status IN ('active', 'superseded')),
    created_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE deployments
    ADD CONSTRAINT deployments_active_bundle_fk
    FOREIGN KEY (active_bundle_id) REFERENCES deployment_bundles(id) ON DELETE SET NULL;

CREATE INDEX deployments_retention_idx
    ON deployments ((COALESCE(last_read_at, published_at)))
    WHERE pinned_at IS NULL;
