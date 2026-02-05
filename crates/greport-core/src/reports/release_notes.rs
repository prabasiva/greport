//! Release notes generation

use crate::models::{Issue, Label, PullRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generated release notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseNotes {
    /// Version/release name
    pub version: String,
    /// Release date
    pub date: String,
    /// Summary text
    pub summary: String,
    /// Categorized sections
    pub sections: Vec<ReleaseSection>,
    /// List of contributors
    pub contributors: Vec<String>,
    /// Release statistics
    pub stats: ReleaseStats,
}

/// Section within release notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseSection {
    /// Section title
    pub title: String,
    /// Items in this section
    pub items: Vec<ReleaseItem>,
}

/// Single item in release notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseItem {
    /// Issue/PR number
    pub number: u64,
    /// Title
    pub title: String,
    /// Author
    pub author: String,
    /// Labels
    pub labels: Vec<String>,
}

/// Release statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseStats {
    /// Number of issues closed
    pub issues_closed: usize,
    /// Number of PRs merged
    pub prs_merged: usize,
    /// Number of contributors
    pub contributors_count: usize,
}

/// Configuration for release notes generation
#[derive(Debug, Clone)]
pub struct ReleaseNotesConfig {
    /// Mapping from label to section title
    pub section_mappings: HashMap<String, String>,
}

impl Default for ReleaseNotesConfig {
    fn default() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert("bug".to_string(), "Bug Fixes".to_string());
        mappings.insert("feature".to_string(), "New Features".to_string());
        mappings.insert("enhancement".to_string(), "Enhancements".to_string());
        mappings.insert("documentation".to_string(), "Documentation".to_string());
        mappings.insert("docs".to_string(), "Documentation".to_string());
        mappings.insert("breaking".to_string(), "Breaking Changes".to_string());
        mappings.insert("security".to_string(), "Security".to_string());
        mappings.insert("performance".to_string(), "Performance".to_string());
        mappings.insert("perf".to_string(), "Performance".to_string());
        mappings.insert("deprecation".to_string(), "Deprecations".to_string());
        mappings.insert("deprecated".to_string(), "Deprecations".to_string());

        Self {
            section_mappings: mappings,
        }
    }
}

/// Generator for release notes
pub struct ReleaseNotesGenerator {
    config: ReleaseNotesConfig,
}

impl ReleaseNotesGenerator {
    /// Create a new generator with the given config
    pub fn new(config: ReleaseNotesConfig) -> Self {
        Self { config }
    }

    /// Create a generator with default config
    pub fn with_defaults() -> Self {
        Self::new(ReleaseNotesConfig::default())
    }

    /// Generate release notes from issues and PRs
    pub fn generate(
        &self,
        version: &str,
        issues: &[Issue],
        prs: &[PullRequest],
    ) -> ReleaseNotes {
        let mut sections: HashMap<String, Vec<ReleaseItem>> = HashMap::new();
        let mut contributors: Vec<String> = Vec::new();

        // Process issues
        for issue in issues {
            let section = self.categorize(&issue.labels);
            let item = ReleaseItem {
                number: issue.number,
                title: issue.title.clone(),
                author: issue.author.login.clone(),
                labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
            };
            sections.entry(section).or_default().push(item);

            if !contributors.contains(&issue.author.login) {
                contributors.push(issue.author.login.clone());
            }
        }

        // Process PRs
        for pr in prs {
            if !contributors.contains(&pr.author.login) {
                contributors.push(pr.author.login.clone());
            }
        }

        // Convert to ordered sections
        let section_order = [
            "Breaking Changes",
            "Security",
            "New Features",
            "Enhancements",
            "Bug Fixes",
            "Performance",
            "Documentation",
            "Deprecations",
            "Other",
        ];

        let ordered_sections: Vec<ReleaseSection> = section_order
            .iter()
            .filter_map(|title| {
                sections.remove(*title).map(|items| ReleaseSection {
                    title: title.to_string(),
                    items,
                })
            })
            .filter(|s| !s.items.is_empty())
            .collect();

        contributors.sort();

        ReleaseNotes {
            version: version.to_string(),
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            summary: format!(
                "This release includes {} issues closed and {} pull requests merged.",
                issues.len(),
                prs.len()
            ),
            sections: ordered_sections,
            contributors: contributors.clone(),
            stats: ReleaseStats {
                issues_closed: issues.len(),
                prs_merged: prs.len(),
                contributors_count: contributors.len(),
            },
        }
    }

    fn categorize(&self, labels: &[Label]) -> String {
        for label in labels {
            let lower = label.name.to_lowercase();
            for (key, section) in &self.config.section_mappings {
                if lower.contains(key) {
                    return section.clone();
                }
            }
        }
        "Other".to_string()
    }

    /// Convert release notes to markdown format
    pub fn to_markdown(&self, notes: &ReleaseNotes) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {} ({})\n\n", notes.version, notes.date));
        md.push_str(&format!("{}\n\n", notes.summary));

        for section in &notes.sections {
            md.push_str(&format!("## {}\n\n", section.title));
            for item in &section.items {
                md.push_str(&format!(
                    "- {} (#{}) @{}\n",
                    item.title, item.number, item.author
                ));
            }
            md.push('\n');
        }

        md.push_str("## Contributors\n\n");
        for contributor in &notes.contributors {
            md.push_str(&format!("- @{}\n", contributor));
        }

        md
    }
}

impl Default for ReleaseNotesGenerator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{IssueState, PullState, User};
    use chrono::Utc;

    fn create_test_user(login: &str) -> User {
        User {
            id: 1,
            login: login.to_string(),
            avatar_url: "".to_string(),
            html_url: "".to_string(),
        }
    }

    fn create_test_label(name: &str) -> Label {
        Label {
            id: 1,
            name: name.to_string(),
            color: "ff0000".to_string(),
            description: None,
        }
    }

    fn create_test_issue(number: u64, title: &str, author: &str, labels: Vec<Label>) -> Issue {
        let now = Utc::now();
        Issue {
            id: number as i64,
            number,
            title: title.to_string(),
            body: None,
            state: IssueState::Closed,
            labels,
            assignees: vec![],
            milestone: None,
            author: create_test_user(author),
            comments_count: 0,
            created_at: now,
            updated_at: now,
            closed_at: Some(now),
            closed_by: None,
        }
    }

    fn create_test_pr(number: u64, author: &str) -> PullRequest {
        let now = Utc::now();
        PullRequest {
            id: number as i64,
            number,
            title: format!("PR #{}", number),
            body: None,
            state: PullState::Closed,
            draft: false,
            author: create_test_user(author),
            labels: vec![],
            milestone: None,
            head_ref: "feature".to_string(),
            base_ref: "main".to_string(),
            merged: true,
            merged_at: Some(now),
            additions: 10,
            deletions: 5,
            changed_files: 1,
            created_at: now,
            updated_at: now,
            closed_at: Some(now),
        }
    }

    #[test]
    fn test_release_notes_basic_generation() {
        let generator = ReleaseNotesGenerator::with_defaults();

        let issues = vec![
            create_test_issue(1, "Fix login bug", "alice", vec![create_test_label("bug")]),
            create_test_issue(2, "Add dark mode", "bob", vec![create_test_label("feature")]),
        ];
        let prs = vec![create_test_pr(10, "charlie")];

        let notes = generator.generate("v1.0.0", &issues, &prs);

        assert_eq!(notes.version, "v1.0.0");
        assert_eq!(notes.stats.issues_closed, 2);
        assert_eq!(notes.stats.prs_merged, 1);
        assert_eq!(notes.stats.contributors_count, 3);
    }

    #[test]
    fn test_release_notes_categorization() {
        let generator = ReleaseNotesGenerator::with_defaults();

        let issues = vec![
            create_test_issue(1, "Fix critical bug", "alice", vec![create_test_label("bug")]),
            create_test_issue(2, "Add new feature", "bob", vec![create_test_label("enhancement")]),
            create_test_issue(3, "Security fix", "charlie", vec![create_test_label("security")]),
            create_test_issue(4, "Uncategorized item", "dave", vec![]),
        ];

        let notes = generator.generate("v1.0.0", &issues, &[]);

        // Check sections exist
        let section_titles: Vec<_> = notes.sections.iter().map(|s| s.title.as_str()).collect();
        assert!(section_titles.contains(&"Bug Fixes"));
        assert!(section_titles.contains(&"Enhancements"));
        assert!(section_titles.contains(&"Security"));
        assert!(section_titles.contains(&"Other"));
    }

    #[test]
    fn test_release_notes_section_ordering() {
        let generator = ReleaseNotesGenerator::with_defaults();

        let issues = vec![
            create_test_issue(1, "Bug fix", "alice", vec![create_test_label("bug")]),
            create_test_issue(2, "Security fix", "bob", vec![create_test_label("security")]),
            create_test_issue(3, "Breaking change", "charlie", vec![create_test_label("breaking")]),
        ];

        let notes = generator.generate("v1.0.0", &issues, &[]);

        let section_titles: Vec<_> = notes.sections.iter().map(|s| s.title.as_str()).collect();

        // Security and Breaking Changes should come before Bug Fixes
        let breaking_idx = section_titles.iter().position(|t| *t == "Breaking Changes");
        let security_idx = section_titles.iter().position(|t| *t == "Security");
        let bug_idx = section_titles.iter().position(|t| *t == "Bug Fixes");

        assert!(breaking_idx.is_some());
        assert!(security_idx.is_some());
        assert!(bug_idx.is_some());
        assert!(breaking_idx < security_idx);
        assert!(security_idx < bug_idx);
    }

    #[test]
    fn test_release_notes_contributors() {
        let generator = ReleaseNotesGenerator::with_defaults();

        let issues = vec![
            create_test_issue(1, "Issue 1", "alice", vec![]),
            create_test_issue(2, "Issue 2", "bob", vec![]),
            create_test_issue(3, "Issue 3", "alice", vec![]), // Duplicate author
        ];
        let prs = vec![
            create_test_pr(10, "charlie"),
            create_test_pr(11, "alice"), // Duplicate author
        ];

        let notes = generator.generate("v1.0.0", &issues, &prs);

        // Should have 3 unique contributors: alice, bob, charlie
        assert_eq!(notes.contributors.len(), 3);
        assert!(notes.contributors.contains(&"alice".to_string()));
        assert!(notes.contributors.contains(&"bob".to_string()));
        assert!(notes.contributors.contains(&"charlie".to_string()));
    }

    #[test]
    fn test_release_notes_to_markdown() {
        let generator = ReleaseNotesGenerator::with_defaults();

        let issues = vec![
            create_test_issue(1, "Fix login bug", "alice", vec![create_test_label("bug")]),
        ];

        let notes = generator.generate("v1.0.0", &issues, &[]);
        let markdown = generator.to_markdown(&notes);

        // Check markdown format
        assert!(markdown.contains("# v1.0.0"));
        assert!(markdown.contains("## Bug Fixes"));
        assert!(markdown.contains("Fix login bug"));
        assert!(markdown.contains("#1"));
        assert!(markdown.contains("@alice"));
        assert!(markdown.contains("## Contributors"));
    }

    #[test]
    fn test_release_notes_empty_input() {
        let generator = ReleaseNotesGenerator::with_defaults();

        let notes = generator.generate("v1.0.0", &[], &[]);

        assert_eq!(notes.stats.issues_closed, 0);
        assert_eq!(notes.stats.prs_merged, 0);
        assert_eq!(notes.stats.contributors_count, 0);
        assert!(notes.sections.is_empty());
    }

    #[test]
    fn test_release_notes_label_matching() {
        let generator = ReleaseNotesGenerator::with_defaults();

        // Labels that contain the keywords should match
        let issues = vec![
            create_test_issue(1, "Fix", "alice", vec![create_test_label("type:bug")]),
            create_test_issue(2, "Doc", "bob", vec![create_test_label("documentation-update")]),
        ];

        let notes = generator.generate("v1.0.0", &issues, &[]);
        let section_titles: Vec<_> = notes.sections.iter().map(|s| s.title.as_str()).collect();

        assert!(section_titles.contains(&"Bug Fixes"));
        assert!(section_titles.contains(&"Documentation"));
    }

    #[test]
    fn test_custom_section_mappings() {
        // Custom section mappings that map to an existing section in the order
        let mut mappings = HashMap::new();
        mappings.insert("custom-bug".to_string(), "Bug Fixes".to_string());

        let config = ReleaseNotesConfig {
            section_mappings: mappings,
        };
        let generator = ReleaseNotesGenerator::new(config);

        let issues = vec![
            create_test_issue(1, "Custom bug fix", "alice", vec![create_test_label("custom-bug")]),
        ];

        let notes = generator.generate("v1.0.0", &issues, &[]);
        let section_titles: Vec<_> = notes.sections.iter().map(|s| s.title.as_str()).collect();

        // "custom-bug" maps to "Bug Fixes" which is in the predefined section order
        assert!(section_titles.contains(&"Bug Fixes"));
        assert_eq!(notes.sections.iter().find(|s| s.title == "Bug Fixes").unwrap().items.len(), 1);
    }

    #[test]
    fn test_categorize_returns_correct_section() {
        let generator = ReleaseNotesGenerator::with_defaults();

        // Test with bug label
        let bug_labels = vec![create_test_label("bug")];
        assert_eq!(generator.categorize(&bug_labels), "Bug Fixes");

        // Test with enhancement label
        let enhancement_labels = vec![create_test_label("enhancement")];
        assert_eq!(generator.categorize(&enhancement_labels), "Enhancements");

        // Test with no matching labels
        let unknown_labels = vec![create_test_label("unknown")];
        assert_eq!(generator.categorize(&unknown_labels), "Other");

        // Test with empty labels
        let empty_labels: Vec<Label> = vec![];
        assert_eq!(generator.categorize(&empty_labels), "Other");
    }
}
