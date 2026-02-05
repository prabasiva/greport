-- Initial database schema for greport

-- Repositories cache
CREATE TABLE IF NOT EXISTS repositories (
    id BIGINT PRIMARY KEY,
    owner VARCHAR(100) NOT NULL,
    name VARCHAR(100) NOT NULL,
    full_name VARCHAR(200) NOT NULL UNIQUE,
    description TEXT,
    private BOOLEAN NOT NULL DEFAULT false,
    default_branch VARCHAR(100) NOT NULL DEFAULT 'main',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_repositories_owner ON repositories(owner);
CREATE INDEX IF NOT EXISTS idx_repositories_full_name ON repositories(full_name);

-- Issues cache
CREATE TABLE IF NOT EXISTS issues (
    id BIGINT PRIMARY KEY,
    repo_id BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number INT NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    state VARCHAR(20) NOT NULL,
    author_login VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    comments_count INT NOT NULL DEFAULT 0,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, number)
);

CREATE INDEX IF NOT EXISTS idx_issues_repo_id ON issues(repo_id);
CREATE INDEX IF NOT EXISTS idx_issues_state ON issues(state);
CREATE INDEX IF NOT EXISTS idx_issues_created_at ON issues(created_at);
CREATE INDEX IF NOT EXISTS idx_issues_closed_at ON issues(closed_at);
CREATE INDEX IF NOT EXISTS idx_issues_author ON issues(author_login);

-- Issue labels (many-to-many)
CREATE TABLE IF NOT EXISTS issue_labels (
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    label_name VARCHAR(100) NOT NULL,
    label_color VARCHAR(10),
    PRIMARY KEY (issue_id, label_name)
);

CREATE INDEX IF NOT EXISTS idx_issue_labels_name ON issue_labels(label_name);

-- Issue assignees (many-to-many)
CREATE TABLE IF NOT EXISTS issue_assignees (
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    assignee_login VARCHAR(100) NOT NULL,
    PRIMARY KEY (issue_id, assignee_login)
);

CREATE INDEX IF NOT EXISTS idx_issue_assignees_login ON issue_assignees(assignee_login);

-- Issue events for timeline analysis
CREATE TABLE IF NOT EXISTS issue_events (
    id BIGINT PRIMARY KEY,
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    actor_login VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL,
    label_name VARCHAR(100),
    assignee_login VARCHAR(100),
    milestone_title VARCHAR(200)
);

CREATE INDEX IF NOT EXISTS idx_issue_events_issue_id ON issue_events(issue_id);
CREATE INDEX IF NOT EXISTS idx_issue_events_type ON issue_events(event_type);
CREATE INDEX IF NOT EXISTS idx_issue_events_created_at ON issue_events(created_at);

-- Pull requests cache
CREATE TABLE IF NOT EXISTS pull_requests (
    id BIGINT PRIMARY KEY,
    repo_id BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number INT NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    state VARCHAR(20) NOT NULL,
    draft BOOLEAN NOT NULL DEFAULT false,
    author_login VARCHAR(100) NOT NULL,
    head_ref VARCHAR(200) NOT NULL,
    base_ref VARCHAR(200) NOT NULL,
    additions INT NOT NULL DEFAULT 0,
    deletions INT NOT NULL DEFAULT 0,
    changed_files INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    merged_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, number)
);

CREATE INDEX IF NOT EXISTS idx_pull_requests_repo_id ON pull_requests(repo_id);
CREATE INDEX IF NOT EXISTS idx_pull_requests_state ON pull_requests(state);
CREATE INDEX IF NOT EXISTS idx_pull_requests_merged_at ON pull_requests(merged_at);
CREATE INDEX IF NOT EXISTS idx_pull_requests_author ON pull_requests(author_login);

-- Milestones
CREATE TABLE IF NOT EXISTS milestones (
    id BIGINT PRIMARY KEY,
    repo_id BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    number INT NOT NULL,
    title VARCHAR(200) NOT NULL,
    description TEXT,
    state VARCHAR(20) NOT NULL,
    open_issues INT NOT NULL DEFAULT 0,
    closed_issues INT NOT NULL DEFAULT 0,
    due_on TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, number)
);

-- Releases
CREATE TABLE IF NOT EXISTS releases (
    id BIGINT PRIMARY KEY,
    repo_id BIGINT NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    tag_name VARCHAR(100) NOT NULL,
    name VARCHAR(200),
    body TEXT,
    draft BOOLEAN NOT NULL DEFAULT false,
    prerelease BOOLEAN NOT NULL DEFAULT false,
    author_login VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    published_at TIMESTAMPTZ,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, tag_name)
);

-- Saved reports
CREATE TABLE IF NOT EXISTS saved_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(100) NOT NULL,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    report_type VARCHAR(50) NOT NULL,
    config JSONB NOT NULL,
    schedule VARCHAR(100),
    last_run_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_saved_reports_user_id ON saved_reports(user_id);

-- API keys
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(100) NOT NULL,
    name VARCHAR(100) NOT NULL,
    key_hash VARCHAR(64) NOT NULL UNIQUE,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);

-- SLA configurations
CREATE TABLE IF NOT EXISTS sla_configs (
    id SERIAL PRIMARY KEY,
    repo_id BIGINT REFERENCES repositories(id) ON DELETE CASCADE,
    priority_label VARCHAR(100),
    response_time_hours INT NOT NULL,
    resolution_time_hours INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(repo_id, priority_label)
);
