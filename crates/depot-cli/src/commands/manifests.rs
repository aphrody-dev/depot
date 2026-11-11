use std::fs;
use std::path::Path;
use depot_core::{EcosystemManifests, StyleExt};

pub fn run_manifests_verify(root: &Path) -> Result<(), String> {
    let status = EcosystemManifests::inspect_root(root);

    println!("{}", "================================================".cyan());
    println!("{}", "      DEPOT 4-PILLAR MANIFEST GOVERNANCE        ".bold().cyan());
    println!("{}", "================================================".cyan());
    println!("Root Workspace:        {}", root.display().to_string().green());
    println!(
        "Chromium DEPS Pins:    {}",
        status.deps_pin_count.to_string().green().bold()
    );
    println!(
        "AOSP repo Projects:    {}",
        status.aosp_project_count.to_string().green().bold()
    );
    println!(
        "Kernel Subsystems:     {}",
        status.kernel_subsystem_count.to_string().green().bold()
    );
    println!(
        "Quilt Patch Series:    {}",
        status.patch_series_count.to_string().green().bold()
    );
    println!("{}", "================================================".cyan());

    if status.deps_pin_count == 0
        || status.aosp_project_count == 0
        || status.kernel_subsystem_count == 0
    {
        return Err(
            "One or more governance manifests (DEPS, .repo/manifests/default.xml, MAINTAINERS) are missing or empty. Run `xtk manifests`.".to_string(),
        );
    }
    Ok(())
}

pub fn run_maintainer_lookup(root: &Path, target_path: &str) -> Result<(), String> {
    let maint_file = root.join("MAINTAINERS");
    let raw = fs::read_to_string(&maint_file)
        .map_err(|e| format!("Failed to read {}: {}", maint_file.display(), e))?;
    let subsystems = EcosystemManifests::parse_kernel_maintainers(&raw);
    let matches = EcosystemManifests::lookup_maintainers_for_path(&subsystems, target_path);

    if matches.is_empty() {
        println!("No maintainer entry matched '{}'.", target_path);
        return Ok(());
    }

    for sub in matches {
        println!("{} [{}]", sub.title.cyan().bold(), sub.status.green());
        for m in &sub.maintainers {
            println!("  M: {}", m);
        }
        for t in &sub.trees {
            println!("  T: {}", t);
        }
    }
    Ok(())
}
