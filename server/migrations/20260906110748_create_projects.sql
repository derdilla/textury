CREATE TABLE IF NOT EXISTS projects (
    id INTEGER NOT NULL PRIMARY KEY UNIQUE,
    title TEXT NOT NULL,
    owner_id INTEGER NOT NULL,
    latest_compilation BLOB,
    last_modified INTEGER NOT NULL,
    last_modified_by INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    last_state BLOB,
    settings TEXT,
    FOREIGN KEY (owner_id) REFERENCES users(id),
    FOREIGN KEY (last_modified_by) REFERENCES users(id)
);
