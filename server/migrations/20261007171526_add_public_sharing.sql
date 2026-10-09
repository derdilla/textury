CREATE TABLE IF NOT EXISTS public_shares (
    share_id TEXT NOT NULL PRIMARY KEY UNIQUE,
    password TEXT NOT NULL,
    project_id INTEGER NOT NULL,
    issuer_id INTEGER NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    FOREIGN KEY (issuer_id) REFERENCES users (id) ON DELETE CASCADE
);

ALTER TABLE users ADD COLUMN tied_to_invite TEXT DEFAULT NULL REFERENCES public_shares (share_id) ON DELETE CASCADE;
