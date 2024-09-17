CREATE TABLE IF NOT EXISTS users(
    user_id TEXT PRIMARY KEY NOT NULL,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL
)
