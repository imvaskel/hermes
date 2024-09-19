CREATE TABLE IF NOT EXISTS notes(
    note_id TEXT PRIMARY KEY NOT NULL,
    note_name TEXT NOT NULL,
    note_owner TEXT NOT NULL,
    FOREIGN KEY(note_owner) REFERENCES users(user_id) ON DELETE CASCADE,
    UNIQUE(note_name, note_owner)
)