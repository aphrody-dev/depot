use std::path::Path;
use depot_core::{
    get_cpu_info, get_memory_status, query_perf_frequency, DepotTools, EcosystemManifests, StyleExt,
};

pub fn run_info() -> Result<(), String> {
    let info = DepotTools::detect();
    let gov = EcosystemManifests::inspect_root(Path::new(r"C:\src"));

    println!("{}", "================================================".cyan());
    println!("{}", "           DEPOT UNIFIED ECOSYSTEM INFO         ".bold().cyan());
    println!("{}", "================================================".cyan());

    // Native Platform & Windows NT kernel32 telemetry via windows-sys
    let (cpus, arch_id) = get_cpu_info();
    let arch_name = match arch_id {
        9 => "x86_64 (AMD64)",
        12 => "ARM64",
        0 => "x86 (IA32)",
        _ => "Unknown",
    };
    println!("Platform Hardware:   {} logical cores, {} architecture", cpus.to_string().green().bold(), arch_name.cyan());

    if let Some((total_phys, avail_phys)) = get_memory_status() {
        let total_disp = format!("{:.2} GB", total_phys as f64 / (1024.0 * 1024.0 * 1024.0));
        let avail_disp = format!("{:.2} GB", avail_phys as f64 / (1024.0 * 1024.0 * 1024.0));
        println!(
            "Memory (RAM):        {} total, {} available",
            total_disp.green().bold(),
            avail_disp.cyan()
        );
    }

    let perf_freq = query_perf_frequency();
    println!("Timer Resolution:    {} Hz (QueryPerformanceCounter)", perf_freq.to_string().yellow());

    println!("\nDepot Tools Root:    {}", match &info.root_path {
        Some(p) => p.display().to_string().green(),
        None => "Not detected".red(),
    });

    println!("Components:");
    println!("  - gclient:         {}", status_flag(info.gclient_found));
    println!("  - repo (AOSP):     {}", status_flag(info.repo_found));
    println!("  - cipd:            {}", status_flag(info.cipd_found));
    println!("  - fetch:           {}", status_flag(info.fetch_found));
    println!("  - autoninja:       {}", status_flag(info.autoninja_found));
    println!("  - siso:            {}", status_flag(info.siso_found));

    println!("\n4-Pillar Manifests (C:\\src):");
    println!("  - Chromium DEPS:   {} pinned entries", gov.deps_pin_count.to_string().green());
    println!("  - AOSP repo XML:   {} projects", gov.aosp_project_count.to_string().green());
    println!("  - Kernel OWNERS:   {} subsystems", gov.kernel_subsystem_count.to_string().green());
    println!("  - Quilt Series:    {} patches", gov.patch_series_count.to_string().green());

    println!("Git Cache Directory: {}", match &info.git_cache_dir {
        Some(p) => p.display().to_string().green(),
        None => "None (not configured)".yellow(),
    });

    println!("\nEngine Features:");
    println!("  - CLI Engine:      {}", "Google argh (zero-bloat derive)".green());
    println!("  - Memory/Framing:  {}", "Google zerocopy 0.8 (zero-copy FFI)".green());
    println!("  - Win32 Platform:  {}", "Microsoft windows-sys (native NT kernel32)".green());
    println!("  - LLM Ingestion:   {}", "Enabled (Rust native rayon + ignore)".green());
    println!("  - AST Graph:       {}", "Enabled (graphify compatible)".green());
    println!("  - Worktree Mgr:    {}", "Enabled (Windows safe)".green());
    println!("  - Git Scalar:      {}", "Enabled".green());
    println!("  - Blake3 Artifact: {}", "Enabled (UE GitDependencies style)".green());

    println!("{}", "================================================".cyan());

    Ok(())
}

fn status_flag(b: bool) -> String {
    if b {
        "Available".green().bold()
    } else {
        "Missing".red()
    }
}
