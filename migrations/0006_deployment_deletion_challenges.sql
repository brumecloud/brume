CREATE TABLE deployment_deletion_challenges (
    id uuid PRIMARY KEY,
    deployment_id uuid NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    challenge_hash bytea NOT NULL UNIQUE,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX deployment_deletion_challenges_expiry_idx
    ON deployment_deletion_challenges (expires_at);
