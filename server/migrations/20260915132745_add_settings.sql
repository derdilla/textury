ALTER TABLE users ADD COLUMN client_settings BLOB;

ALTER TABLE projects ADD COLUMN settings TEXT;

CREATE TABLE IF NOT EXISTS server_settings (
    key TEXT NOT NULL PRIMARY KEY UNIQUE,
    value BLOB NOT NULL
);
