-- schema.sql: tables for the demo.
-- It has a two-line header, then the statements.

CREATE TABLE notes (
  id INTEGER PRIMARY KEY,
  body TEXT NOT NULL
);

-- Later run: an index note.
-- It runs over two lines.
CREATE INDEX notes_body ON notes (body);
