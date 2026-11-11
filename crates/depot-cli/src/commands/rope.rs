// SPDX-License-Identifier: Apache-2.0 OR MIT
use std::path::Path;
use depot_core::StyleExt;
use depot_treesitter::apply_helix_rope_edits;

pub fn execute_rope(file: &Path, replacements: &[String]) -> Result<(), String> {
    println!("{}", format!("--- Depot Helix Rope Editor: {} ---", file.display()).cyan().bold());

    let content = std::fs::read_to_string(file).map_err(|e| format!("Cannot read file: {e}"))?;

    let mut edits = Vec::new();
    for r in replacements {
        let parts: Vec<&str> = r.splitn(3, ':').collect();
        if parts.len() == 3 {
            if let (Ok(start), Ok(end)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                edits.push((start, end, parts[2].to_string()));
            } else {
                eprintln!("{} Invalid byte range in: {}", "WARN:".yellow(), r);
            }
        } else {
            eprintln!("{} Expected START:END:REPLACEMENT, got: {}", "WARN:".yellow(), r);
        }
    }

    if edits.is_empty() {
        println!("  No edits specified. Use -r <start>:<end>:<text>");
        return Ok(());
    }

    let modified = apply_helix_rope_edits(&content, edits);
    std::fs::write(file, &modified).map_err(|e| format!("Cannot write file: {e}"))?;
    println!("{} Applied {} transactional edits successfully", "SUCCESS:".green().bold(), replacements.len());
    Ok(())
}
