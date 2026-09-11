CREATE TABLE IF NOT EXISTS projects (
    id TEXT NOT NULL PRIMARY KEY UNIQUE,
    title TEXT NOT NULL,
    owner_id INTEGER NOT NULL,
    latest_compilation BLOB,
    last_modified INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    last_state BLOB,
    FOREIGN KEY (owner_id) REFERENCES users(id)
);
