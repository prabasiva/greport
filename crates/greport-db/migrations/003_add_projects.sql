-- Add GitHub Projects V2 tables

-- Projects table
CREATE TABLE IF NOT EXISTS projects (
    node_id VARCHAR(255) PRIMARY KEY,
    number BIGINT NOT NULL,
    owner VARCHAR(255) NOT NULL,
    title VARCHAR(512) NOT NULL,
    description TEXT,
    url VARCHAR(1024) NOT NULL,
    closed BOOLEAN NOT NULL DEFAULT FALSE,
    total_items INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(owner, number)
);

CREATE INDEX IF NOT EXISTS idx_projects_owner ON projects(owner);
CREATE INDEX IF NOT EXISTS idx_projects_closed ON projects(closed);

-- Project field definitions
CREATE TABLE IF NOT EXISTS project_fields (
    node_id VARCHAR(255) PRIMARY KEY,
    project_id VARCHAR(255) NOT NULL REFERENCES projects(node_id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    field_type VARCHAR(50) NOT NULL,
    config_json JSONB,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_project_fields_project ON project_fields(project_id);

-- Project items (denormalized content)
CREATE TABLE IF NOT EXISTS project_items (
    node_id VARCHAR(255) PRIMARY KEY,
    project_id VARCHAR(255) NOT NULL REFERENCES projects(node_id) ON DELETE CASCADE,
    content_type VARCHAR(20) NOT NULL,
    content_number BIGINT,
    content_title TEXT NOT NULL,
    content_state VARCHAR(20),
    content_url VARCHAR(1024),
    content_repository VARCHAR(512),
    content_json JSONB,
    field_values_json JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_project_items_project ON project_items(project_id);
CREATE INDEX IF NOT EXISTS idx_project_items_content_type ON project_items(content_type);
CREATE INDEX IF NOT EXISTS idx_project_items_repository ON project_items(content_repository);
CREATE INDEX IF NOT EXISTS idx_project_items_state ON project_items(content_state);
