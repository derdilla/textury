CREATE TABLE IF NOT EXISTS project_memberships (
    user_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    user_meta TEXT,
    CONSTRAINT pk_project_memberships PRIMARY KEY (user_id, project_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
);
