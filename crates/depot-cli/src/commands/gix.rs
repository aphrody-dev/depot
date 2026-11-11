use std::path::PathBuf;
use depot_core::{GixEngine, StyleExt};

pub fn run_gix_status(
    repo_path: Option<PathBuf>,
    commits: usize,
    json_output: bool,
) -> Result<(), String> {
    let p = repo_path.unwrap_or_else(|| PathBuf::from("."));
    let telemetry = GixEngine::inspect(&p, commits)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&telemetry).unwrap());
        return Ok(());
    }

    println!("{} Git Repository Telemetry:\n", "[GIX]".cyan().bold());
    println!("  - Workdir:          {}", telemetry.workdir.yellow());
    println!("  - Git dir:          {}", telemetry.git_dir);
    println!("  - Current Branch:   {}", telemetry.branch.green().bold());
    if let Some(ref c) = telemetry.head_commit {
        let subj = telemetry.head_subject.as_deref().unwrap_or("");
        println!("  - HEAD Commit:      {} ({})", &c[..c.len().min(10)].cyan(), subj);
    }
    println!("  - Tracked entries:  {}", telemetry.tracked_entry_count);
    println!("  - Has Jujutsu:      {}", if telemetry.has_jujutsu { "yes".green() } else { "no".into() });

    if !telemetry.remotes.is_empty() {
        println!("\n  Remotes:");
        for (name, url) in &telemetry.remotes {
            println!("    - {}: {}", name.bold(), url);
        }
    }

    if !telemetry.recent_commits.is_empty() {
        println!("\n  Recent Commits ({}):", telemetry.recent_commits.len());
        for c in &telemetry.recent_commits {
            println!(
                "    - {} {} [{}] <{}>",
                c.short_sha.cyan().bold(),
                c.subject,
                c.author_name.yellow(),
                c.author_email
            );
        }
    }

    Ok(())
}
