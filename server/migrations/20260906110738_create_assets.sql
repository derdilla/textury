CREATE TABLE IF NOT EXISTS assets (
    project_id TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    data BLOB NOT NULL,
    CONSTRAINT pk_asset PRIMARY KEY (project_id, asset_id)
);
