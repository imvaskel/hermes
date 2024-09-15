-- Add migration script here
CREATE TABLE notes(
    created_at NUMBER NOT NULL,
    content TEXT NOT NULL,
    id TEXT NOT NULL
);