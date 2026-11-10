use std::path::PathBuf;
use depot_core::{Artifact, ArtifactManifest, StyleExt};

pub fn run_artifacts_verify(manifest_path: PathBuf) -> Result<(), String> {
    println!(
        "{} Loading artifact manifest from {}...",
        "->".cyan().bold(),
        manifest_path.display().to_string().yellow()
    );

    let manifest = Artifact::load_manifest(&manifest_path)?;
    let base_dir = manifest_path.parent().unwrap_or(&manifest_path);
    let report = Artifact::verify(&manifest, base_dir);

    println!(
        "{} Verified: {}/{} OK | Missing: {} | Mismatched: {}",
        if report.missing.is_empty() && report.mismatched.is_empty() {
            "[OK]".green().bold()
        } else {
            "[WARN]".red().bold()
        },
        report.verified_ok,
        report.total_artifacts,
        report.missing.len(),
        report.mismatched.len()
    );

    for m in report.missing {
        println!("  - Missing: {}", m.red());
    }
    for m in report.mismatched {
        println!("  - Mismatched: {}", m.red());
    }

    Ok(())
}

pub fn run_artifacts_hash(file_path: PathBuf) -> Result<(), String> {
    let (hash, size) = Artifact::compute_blake3(&file_path)?;
    println!(
        "{} {}:",
        "[BLAKE3]".cyan().bold(),
        file_path.display().to_string().yellow()
    );
    println!("  Hash:  {}", hash.green().bold());
    println!("  Bytes: {}", size);
    Ok(())
}

pub fn run_artifacts_init(manifest_path: PathBuf, project_name: String) -> Result<(), String> {
    let manifest = ArtifactManifest {
        version: "1.0".to_string(),
        project_name,
        artifacts: Vec::new(),
    };
    Artifact::save_manifest(&manifest, &manifest_path)?;
    println!(
        "{} Initialized artifact manifest at {}",
        "[OK]".green().bold(),
        manifest_path.display().to_string().cyan()
    );
    Ok(())
}
