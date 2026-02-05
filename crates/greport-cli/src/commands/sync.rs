//! Sync command handlers

use crate::args::SyncArgs;
use greport_core::client::{GitHubClient, IssueParams, PullParams, RepoId};

pub async fn handle_sync(
    client: &impl GitHubClient,
    repo: &RepoId,
    args: SyncArgs,
) -> anyhow::Result<()> {
    let sync_all = args.all || (!args.issues && !args.pulls);
    let sync_issues = args.issues || sync_all;
    let sync_pulls = args.pulls || sync_all;

    println!("Syncing data for {}...\n", repo);

    if sync_issues {
        print!("Fetching issues... ");
        let issues = client.list_issues(repo, IssueParams::all()).await?;
        println!("fetched {} issues", issues.len());

        // TODO: Store in local database
    }

    if sync_pulls {
        print!("Fetching pull requests... ");
        let prs = client.list_pulls(repo, PullParams::all()).await?;
        println!("fetched {} PRs", prs.len());

        // TODO: Store in local database
    }

    println!("\nSync complete!");
    println!("Note: Local database storage not yet implemented.");

    Ok(())
}
