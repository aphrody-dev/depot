// SPDX-License-Identifier: Apache-2.0 OR MIT
use std::path::Path;
use depot_core::StyleExt;
use depot_re::ReEngine;

pub fn execute_re(binary_path: &Path, output_json: bool) -> Result<(), String> {
    println!("{}", format!("--- Depot RE Engine: {} ---", binary_path.display()).cyan().bold());

    let audit = ReEngine::audit_file(binary_path)?;

    if output_json {
        println!("{}", serde_json::to_string_pretty(&audit).unwrap_or_default());
        return Ok(());
    }

    println!("  Format: {} (64-bit: {})", audit.format.format.green().bold(), audit.format.is_64_bit);
    println!("  Entry point: 0x{:X}", audit.format.entry_point);
    println!("  Sections: {} found", audit.format.sections.len());
    for s in audit.format.sections.iter().take(6) {
        println!("    {:12} addr: 0x{:08X} size: {:>8} bytes", s.name.cyan(), s.virtual_address, s.size);
    }
    if audit.format.sections.len() > 6 {
        println!("    ... and {} more sections", audit.format.sections.len() - 6);
    }

    println!("  Exports: {} | Imports: {}", audit.format.exports.len(), audit.format.imports.len());
    println!("  REA Markers: dwarf={} rea_sig={}", audit.format.has_dwarf_debug, audit.format.has_rea_marker);

    println!("{}", "--- MinGW CRT Boilerplate Noise Filter ---".yellow().bold());
    println!("  Total symbols: {}", audit.noise_filter.total_symbols);
    println!("  Noise filtered: {}", audit.noise_filter.noise_symbols_count.to_string().red());
    println!("  Payload symbols: {}", audit.noise_filter.payload_symbols_count.to_string().green().bold());
    println!("  Noise reduction: {:.1}%", audit.noise_filter.noise_reduction_ratio * 100.0);

    if !audit.entry_disassembly.is_empty() {
        println!("{}", "--- Entry Point Disassembly (iced-x86) ---".magenta().bold());
        for instr in audit.entry_disassembly.iter().take(8) {
            println!("    0x{:08X}: {}", instr.ip, instr.formatted);
        }
    }

    Ok(())
}
