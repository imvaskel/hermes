CREATE TABLE IF NOT EXISTS entries(
    entry_id TEXT PRIMARY KEY NOT NULL, -- The ID of the note
    created_at INT NOT NULL, -- When the note was created, unix timestamp
    content TEXT NOT NULL, -- The content of the note, supports markdown.
    parent TEXT NOT NULL, -- The ID of the "parent"
    FOREIGN KEY(parent) REFERENCES notes(note_id)
    ON DELETE CASCADE
)