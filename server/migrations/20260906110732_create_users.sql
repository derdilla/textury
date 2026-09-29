CREATE TABLE IF NOT EXISTS users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    invited_by INTEGER,
    is_admin BOOLEAN NOT NULL,
    FOREIGN KEY (invited_by) REFERENCES users(id)
    CONSTRAINT no_self_invite CHECK (invited_by <> id)
);
