use colored::Colorize;
use depot_core::DepotTools;

pub fn run_info() -> Result<(), String> {
    let info = DepotTools::detect();

    println!("{}", "================================================".cyan());
    println!("{}", "           DEPOT UNIFIED ECOSYSTEM INFO         ".bold().cyan());
    println!("{}", "================================================".cyan());

    println!("Depot Tools Root:    {}", match &info.root_path {
        Some(p) => p.display().to_string().green(),
        None => "Not detected".red(),
    });

    println!("Components:");
    println!("  - gclient:         {}", status_flag(info.gclient_found));
    println!("  - cipd:            {}", status_flag(info.cipd_found));
    println!("  - fetch:           {}", status_flag(info.fetch_found));
    println!("  - autoninja:       {}", status_flag(info.autoninja_found));
    println!("  - siso:            {}", status_flag(info.siso_found));

    println!("Git Cache Directory: {}", match &info.git_cache_dir {
        Some(p) => p.display().to_string().green(),
        None => "None (not configured)".yellow(),
    });

    println!("\nEngine Features:");
    println!("  - LLM Ingestion:   {}", "Enabled (Rust native rayon + ignore)".green());
    println!("  - AST Graph:       {}", "Enabled (graphify compatible)".green());
    println!("  - Worktree Mgr:    {}", "Enabled (Windows safe)".green());
    println!("  - Git Scalar:      {}", "Enabled".green());
    println!("  - Blake3 Artifact: {}", "Enabled (UE GitDependencies style)".green());

    println!("{}", "================================================".cyan());

    Ok(())
}

fn status_flag(b: bool) -> colored::ColoredString {
    if b {
        "Available".green().bold()
    } else {
        "Missing".red()
    }
}
