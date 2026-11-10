ALTER TABLE public_shares ADD COLUMN initial_perms TEXT NOT NULL DEFAULT "View";
ALTER TABLE project_join_tokens ADD COLUMN initial_perms TEXT NOT NULL DEFAULT "View";
ALTER TABLE project_memberships ADD COLUMN perms TEXT NOT NULL DEFAULT "View";
