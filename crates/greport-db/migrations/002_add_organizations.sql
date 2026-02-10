-- Add organization tracking

-- Add org_name column to repositories
ALTER TABLE repositories ADD COLUMN org_name VARCHAR(255);

-- Backfill org_name from owner
UPDATE repositories SET org_name = owner WHERE org_name IS NULL;

-- Index for efficient org-based queries
CREATE INDEX idx_repositories_org_name ON repositories(org_name);

-- Organizations table for tracking configured orgs
CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    base_url VARCHAR(1024),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ
);
