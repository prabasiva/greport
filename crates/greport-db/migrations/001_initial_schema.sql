-- Initial database schema for greport

-- Repositories table
CREATE TABLE IF NOT EXISTS repositories (
    id BIGINT PRIMARY KEY,
    owner VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    full_name VARCHAR(512) NOT NULL UNIQUE,
    description TEXT,
    private BOOLEAN NOT NULL DEFAULT FALSE,
    default_branch VARCHAR(255) NOT NULL DEFAULT 'main',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_repositories_owner ON repositories(owner);
CREATE INDEX idx_repositories_full_name ON repositories(full_name);

-- Milestones table
CREATE TABLE IF NOT EXISTS milestones (
    id BIGINT PRIMARY KEY,
    repository_id BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number BIGINT NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    state VARCHAR(20) NOT NULL DEFAULT 'open',
    open_issues INTEGER NOT NULL DEFAULT 0,
    closed_issues INTEGER NOT NULL DEFAULT 0,
    due_on TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_milestones_repository ON milestones(repository_id);
CREATE INDEX idx_milestones_state ON milestones(state);

-- Issues table
CREATE TABLE IF NOT EXISTS issues (
    id BIGINT PRIMARY KEY,
    repository_id BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number BIGINT NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    state VARCHAR(20) NOT NULL DEFAULT 'open',
    milestone_id BIGINT REFERENCES milestones(id) ON DELETE SET NULL,
    author_login VARCHAR(255) NOT NULL,
    author_id BIGINT NOT NULL,
    comments_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    closed_by_login VARCHAR(255),
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repository_id, number)
);

CREATE INDEX idx_issues_repository ON issues(repository_id);
CREATE INDEX idx_issues_state ON issues(state);
CREATE INDEX idx_issues_author ON issues(author_login);
CREATE INDEX idx_issues_milestone ON issues(milestone_id);
CREATE INDEX idx_issues_created_at ON issues(created_at);

-- Issue labels (many-to-many)
CREATE TABLE IF NOT EXISTS issue_labels (
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    label_id BIGINT NOT NULL,
    label_name VARCHAR(255) NOT NULL,
    label_color VARCHAR(10),
    PRIMARY KEY (issue_id, label_id)
);

CREATE INDEX idx_issue_labels_name ON issue_labels(label_name);

-- Issue assignees (many-to-many)
CREATE TABLE IF NOT EXISTS issue_assignees (
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL,
    user_login VARCHAR(255) NOT NULL,
    PRIMARY KEY (issue_id, user_id)
);

CREATE INDEX idx_issue_assignees_login ON issue_assignees(user_login);

-- Pull requests table
CREATE TABLE IF NOT EXISTS pull_requests (
    id BIGINT PRIMARY KEY,
    repository_id BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number BIGINT NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    state VARCHAR(20) NOT NULL DEFAULT 'open',
    draft BOOLEAN NOT NULL DEFAULT FALSE,
    milestone_id BIGINT REFERENCES milestones(id) ON DELETE SET NULL,
    author_login VARCHAR(255) NOT NULL,
    author_id BIGINT NOT NULL,
    head_ref VARCHAR(255) NOT NULL,
    base_ref VARCHAR(255) NOT NULL,
    merged BOOLEAN NOT NULL DEFAULT FALSE,
    merged_at TIMESTAMPTZ,
    additions INTEGER NOT NULL DEFAULT 0,
    deletions INTEGER NOT NULL DEFAULT 0,
    changed_files INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repository_id, number)
);

CREATE INDEX idx_pull_requests_repository ON pull_requests(repository_id);
CREATE INDEX idx_pull_requests_state ON pull_requests(state);
CREATE INDEX idx_pull_requests_author ON pull_requests(author_login);
CREATE INDEX idx_pull_requests_merged ON pull_requests(merged);

-- Releases table
CREATE TABLE IF NOT EXISTS releases (
    id BIGINT PRIMARY KEY,
    repository_id BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    tag_name VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    body TEXT,
    draft BOOLEAN NOT NULL DEFAULT FALSE,
    prerelease BOOLEAN NOT NULL DEFAULT FALSE,
    author_login VARCHAR(255) NOT NULL,
    author_id BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    published_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repository_id, tag_name)
);

CREATE INDEX idx_releases_repository ON releases(repository_id);
CREATE INDEX idx_releases_tag ON releases(tag_name);

-- API keys table
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    owner VARCHAR(255) NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    rate_limit INTEGER NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    revoked BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_owner ON api_keys(owner);

-- Sync status table
CREATE TABLE IF NOT EXISTS sync_status (
    repository_id BIGINT PRIMARY KEY REFERENCES repositories(id) ON DELETE CASCADE,
    issues_synced_at TIMESTAMPTZ,
    pulls_synced_at TIMESTAMPTZ,
    releases_synced_at TIMESTAMPTZ,
    milestones_synced_at TIMESTAMPTZ,
    last_error TEXT,
    last_error_at TIMESTAMPTZ
);

-- Cache metadata table
CREATE TABLE IF NOT EXISTS cache_metadata (
    key VARCHAR(512) PRIMARY KEY,
    data_type VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    hit_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_cache_metadata_expires ON cache_metadata(expires_at);
