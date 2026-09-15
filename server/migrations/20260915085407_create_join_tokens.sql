CREATE TABLE IF NOT EXISTS project_join_tokens (
    token TEXT NOT NULL PRIMARY KEY UNIQUE,
    expires INTEGER NOT NULL,
    project_id TEXT NOT NULL,
    issuer_id INTEGER NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    FOREIGN KEY (issuer_id) REFERENCES users (id) ON DELETE CASCADE
);
