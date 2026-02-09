//! CLI argument definitions

use clap::{Parser, Subcommand, ValueEnum};

/// GitHub reporting and analytics tool
#[derive(Parser)]
#[command(name = "greport")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Target repository (owner/repo)
    #[arg(short, long, global = true, env = "GREPORT_REPO")]
    pub repo: Option<String>,

    /// Target organization
    #[arg(short, long, global = true)]
    pub org: Option<String>,

    /// Output format
    #[arg(short, long, global = true, default_value = "table")]
    pub format: OutputFormat,

    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Bypass cache
    #[arg(long, global = true)]
    pub no_cache: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Issue reports and analytics
    Issues(IssuesArgs),

    /// Pull request reports
    Prs(PrsArgs),

    /// Release tracking and notes
    Releases(ReleasesArgs),

    /// Contributor analytics
    Contrib(ContribArgs),

    /// Configuration management
    Config(ConfigArgs),

    /// Sync data from GitHub
    Sync(SyncArgs),
}

// Issues commands
#[derive(Parser)]
pub struct IssuesArgs {
    #[command(subcommand)]
    pub command: IssuesCommands,
}

#[derive(Subcommand, Clone)]
pub enum IssuesCommands {
    /// List issues with filters
    List {
        /// Filter by state
        #[arg(long, default_value = "open")]
        state: IssueStateFilter,

        /// Filter by labels (comma-separated)
        #[arg(long)]
        labels: Option<String>,

        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,

        /// Filter by milestone
        #[arg(long)]
        milestone: Option<String>,

        /// Filter by created since date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// Count issues by grouping
    Count {
        /// Group by field
        #[arg(long, default_value = "state")]
        group_by: GroupBy,

        /// Filter by state
        #[arg(long)]
        state: Option<IssueStateFilter>,
    },

    /// Issue age analysis
    Age {
        /// Only open issues
        #[arg(long)]
        open_only: bool,
    },

    /// Find stale issues
    Stale {
        /// Days without activity
        #[arg(long, default_value = "30")]
        days: i64,
    },

    /// Velocity metrics
    Velocity {
        /// Time period
        #[arg(long, default_value = "week")]
        period: PeriodArg,

        /// Number of periods
        #[arg(long, default_value = "12")]
        last: usize,
    },

    /// Burndown chart data
    Burndown {
        /// Milestone name
        #[arg(long)]
        milestone: String,
    },

    /// SLA compliance report
    Sla,

    /// Issue metrics summary
    Metrics,
}

// Pull request commands
#[derive(Parser)]
pub struct PrsArgs {
    #[command(subcommand)]
    pub command: PrsCommands,
}

#[derive(Subcommand, Clone)]
pub enum PrsCommands {
    /// List pull requests with filters
    List {
        /// Filter by state
        #[arg(long, default_value = "open")]
        state: PrStateFilter,

        /// Filter by author
        #[arg(long)]
        author: Option<String>,

        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// PR metrics summary
    Metrics,

    /// PRs without reviews
    Unreviewed,
}

// Release commands
#[derive(Parser)]
pub struct ReleasesArgs {
    #[command(subcommand)]
    pub command: ReleasesCommands,
}

#[derive(Subcommand, Clone)]
pub enum ReleasesCommands {
    /// List releases
    List {
        /// Maximum results
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// Generate release notes
    Notes {
        /// Milestone name
        #[arg(long)]
        milestone: String,

        /// Version string
        #[arg(long)]
        version: Option<String>,
    },

    /// Milestone progress
    Progress {
        /// Milestone name
        #[arg(long)]
        milestone: String,
    },
}

// Contributor commands
#[derive(Parser)]
pub struct ContribArgs {
    #[command(subcommand)]
    pub command: ContribCommands,
}

#[derive(Subcommand, Clone)]
pub enum ContribCommands {
    /// List contributors with stats
    List {
        /// Sort by metric
        #[arg(long, default_value = "issues")]
        sort_by: ContribSort,

        /// Maximum results
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Stats for a specific contributor
    Stats {
        /// Username
        username: String,
    },
}

// Config commands
#[derive(Parser)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Initialize configuration file
    Init {
        /// Overwrite existing config
        #[arg(long)]
        force: bool,
    },

    /// Set a configuration value
    Set {
        /// Key to set
        key: String,
        /// Value to set
        value: String,
    },

    /// Show configuration file path
    Path,
}

// Sync command
#[derive(Parser, Clone)]
pub struct SyncArgs {
    /// Sync all data
    #[arg(long)]
    pub all: bool,

    /// Sync only issues
    #[arg(long)]
    pub issues: bool,

    /// Sync only pull requests
    #[arg(long)]
    pub pulls: bool,
}

// Value enums
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
    Markdown,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum IssueStateFilter {
    #[default]
    Open,
    Closed,
    All,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum PrStateFilter {
    #[default]
    Open,
    Closed,
    All,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum GroupBy {
    #[default]
    State,
    Label,
    Assignee,
    Milestone,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum PeriodArg {
    Day,
    #[default]
    Week,
    Month,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum ContribSort {
    #[default]
    Issues,
    Prs,
    Comments,
}

impl From<IssueStateFilter> for greport_core::client::IssueStateFilter {
    fn from(val: IssueStateFilter) -> Self {
        match val {
            IssueStateFilter::Open => greport_core::client::IssueStateFilter::Open,
            IssueStateFilter::Closed => greport_core::client::IssueStateFilter::Closed,
            IssueStateFilter::All => greport_core::client::IssueStateFilter::All,
        }
    }
}

impl From<PrStateFilter> for greport_core::client::PullStateFilter {
    fn from(val: PrStateFilter) -> Self {
        match val {
            PrStateFilter::Open => greport_core::client::PullStateFilter::Open,
            PrStateFilter::Closed => greport_core::client::PullStateFilter::Closed,
            PrStateFilter::All => greport_core::client::PullStateFilter::All,
        }
    }
}

impl From<PeriodArg> for greport_core::metrics::Period {
    fn from(val: PeriodArg) -> Self {
        match val {
            PeriodArg::Day => greport_core::metrics::Period::Day,
            PeriodArg::Week => greport_core::metrics::Period::Week,
            PeriodArg::Month => greport_core::metrics::Period::Month,
        }
    }
}
