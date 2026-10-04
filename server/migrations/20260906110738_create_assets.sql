CREATE TABLE IF NOT EXISTS assets (
    project_id INTEGER NOT NULL,
    asset_id TEXT NOT NULL,
    data BLOB NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
    CONSTRAINT pk_asset PRIMARY KEY (project_id, asset_id)
);
