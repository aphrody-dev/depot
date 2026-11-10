use std::path::PathBuf;
use depot_core::{IgnoreTemplate, SmartFilter, StyleExt};

pub fn run_ignore_init(target_file: PathBuf, template_name: String) -> Result<(), String> {
    let t = match template_name.to_lowercase().as_str() {
        "rust" => IgnoreTemplate::Rust,
        "bun" | "node" => IgnoreTemplate::BunNode,
        "chromium" | "cpp" => IgnoreTemplate::ChromiumCpp,
        "python" => IgnoreTemplate::Python,
        _ => IgnoreTemplate::LlmUniversal,
    };

    let content = SmartFilter::template(t);
    std::fs::write(&target_file, content)
        .map_err(|e| format!("Failed to write {}: {}", target_file.display(), e))?;

    println!(
        "{} Wrote {} template to {}",
        "[OK]".green().bold(),
        template_name.cyan().bold(),
        target_file.display().to_string().yellow()
    );

    Ok(())
}
