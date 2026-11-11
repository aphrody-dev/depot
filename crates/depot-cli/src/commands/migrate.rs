// SPDX-License-Identifier: Apache-2.0 OR MIT
use std::path::Path;
use depot_core::StyleExt;
use depot_migrate::MigrateEngine;

pub fn execute_migrate(root: &Path, dry_run: bool) -> Result<(), String> {
    println!("{}", format!("--- Depot Migrate Engine: {} ---", root.display()).cyan().bold());
    if dry_run {
        println!("{}", "[DRY RUN] No files will be modified on disk".yellow().bold());
    }

    let rep = MigrateEngine::run(root, dry_run)?;

    println!("  Detected Archetype: {}", rep.archetype.as_str().green().bold());
    println!("  Target Stack: {}", rep.target_stack.cyan());
    println!("  Files scanned: {}", rep.files_scanned);
    println!("  Files modified: {}", rep.files_modified.to_string().green().bold());
    println!("  Total AST rewrites: {}", rep.total_edits.to_string().yellow().bold());
    if !rep.legacy_deps_removed.is_empty() {
        println!("  Purged legacy packages: {}", rep.legacy_deps_removed.join(", ").red());
    }

    Ok(())
}
