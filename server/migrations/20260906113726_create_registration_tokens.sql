CREATE TABLE IF NOT EXISTS registration_tokens (
    token TEXT NOT NULL PRIMARY KEY UNIQUE,
    expires INTEGER NOT NULL,
    issuer_id INTEGER,
    FOREIGN KEY (issuer_id) REFERENCES users (id) ON DELETE CASCADE
);
