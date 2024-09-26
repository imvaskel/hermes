CREATE VIEW v_entries
AS
SELECT * FROM entries
INNER JOIN notes ON entries.parent = notes.note_id
INNER JOIN users ON notes.note_owner = users.user_id;

CREATE VIEW v_notes
AS
SELECT * FROM notes
INNER JOIN users ON notes.note_owner = users.user_id;