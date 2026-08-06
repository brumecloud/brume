-- Toolbar writes are now JSON requests protected by Origin validation on the
-- auth host, so the short-lived per-form action tokens are no longer needed.
DROP TABLE toolbar_action_tokens;
