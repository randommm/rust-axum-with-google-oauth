CREATE TABLE "users" (
    "id" integer NOT NULL PRIMARY KEY AUTOINCREMENT,
    "email" text NOT NULL UNIQUE
);
CREATE TABLE "user_sessions" (
    "id" integer NOT NULL PRIMARY KEY AUTOINCREMENT,
    "user_id" integer NOT NULL,
    "session_token_p1" text NOT NULL,
    "session_token_p2" text NOT NULL,
    "created_at" integer NOT NULL,
    "expires_at" integer NOT NULL
);
CREATE TABLE "oauth2_state_storage" (
    "id" integer NOT NULL PRIMARY KEY AUTOINCREMENT,
    "csrf_state" text NOT NULL,
    "pkce_code_verifier" text NOT NULL,
    "return_url" text NOT NULL
);
PRAGMA journal_mode=WAL;
