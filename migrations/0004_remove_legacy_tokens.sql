DROP TABLE api_tokens;
DROP TABLE web_sessions;
ALTER TABLE cli_login_sessions DROP COLUMN issued_token;
