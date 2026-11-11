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

    if audit.is_lua_bytecode {
        if let Some(lua) = &audit.lua_bytecode {
            println!("  Binary Type: {}", "Lua 5.2 Compiled Bytecode".green().bold());
            println!("  Version: {} (official: {}, endian: {})", lua.header.version, lua.header.is_official_format, if lua.header.is_little_endian { "little" } else { "big" });
            println!("  Types: sizeof(int)={} sizeof(size_t)={} sizeof(Instruction)={}", lua.header.size_int, lua.header.size_size_t, lua.header.size_instruction);
            println!("  Prototypes: {} | Total Instructions: {}", lua.total_prototypes, lua.total_instructions);
            println!("  Main Prototype: lines {}-{} (params: {}, vararg: {}, stack: {})",
                lua.main_prototype.line_defined,
                lua.main_prototype.last_line_defined,
                lua.main_prototype.num_params,
                lua.main_prototype.is_vararg,
                lua.main_prototype.max_stack_size
            );

            if !lua.main_prototype.constants.is_empty() {
                println!("{}", "--- Constant Pool ---".yellow().bold());
                for (idx, c) in lua.main_prototype.constants.iter().enumerate().take(8) {
                    println!("    K[{idx}] = {c}");
                }
                if lua.main_prototype.constants.len() > 8 {
                    println!("    ... and {} more constants", lua.main_prototype.constants.len() - 8);
                }
            }

            if !lua.global_accesses.is_empty() {
                println!("{}", "--- _ENV Lexical Global Accesses ---".yellow().bold());
                for g in lua.global_accesses.iter().take(12) {
                    println!(
                        "    [{:04}] {:5} {} ({})",
                        g.pc,
                        g.kind.green(),
                        g.symbol.bold(),
                        g.proto_path
                    );
                }
                if lua.global_accesses.len() > 12 {
                    println!("    ... and {} more global accesses", lua.global_accesses.len() - 12);
                }
            }

            println!("{}", "--- Lua 5.2 REA Disassembly ---".magenta().bold());
            for instr in lua.main_prototype.instructions.iter().take(16) {
                println!("    [{:04}] {:10} {}", instr.pc, instr.opcode.cyan(), instr.formatted);
            }
            if lua.main_prototype.instructions.len() > 16 {
                println!("    ... and {} more instructions", lua.main_prototype.instructions.len() - 16);
            }
        }
        return Ok(());
    }

    if let Some(format) = &audit.format {
        println!("  Format: {} (64-bit: {})", format.format.green().bold(), format.is_64_bit);
        println!("  Entry point: 0x{:X}", format.entry_point);
        println!("  Sections: {} found", format.sections.len());
        for s in format.sections.iter().take(6) {
            println!("    {:12} addr: 0x{:08X} size: {:>8} bytes", s.name.cyan(), s.virtual_address, s.size);
        }
        if format.sections.len() > 6 {
            println!("    ... and {} more sections", format.sections.len() - 6);
        }

        println!("  Exports: {} | Imports: {}", format.exports.len(), format.imports.len());
        println!("  REA Markers: dwarf={} rea_sig={}", format.has_dwarf_debug, format.has_rea_marker);
    }

    if let Some(noise) = &audit.noise_filter {
        println!("{}", "--- MinGW CRT Boilerplate Noise Filter ---".yellow().bold());
        println!("  Total symbols: {}", noise.total_symbols);
        println!("  Noise filtered: {}", noise.noise_symbols_count.to_string().red());
        println!("  Payload symbols: {}", noise.payload_symbols_count.to_string().green().bold());
        println!("  Noise reduction: {:.1}%", noise.noise_reduction_ratio * 100.0);
    }

    if !audit.entry_disassembly.is_empty() {
        println!("{}", "--- Entry Point Disassembly (iced-x86) ---".magenta().bold());
        for instr in audit.entry_disassembly.iter().take(8) {
            println!("    0x{:08X}: {}", instr.ip, instr.formatted);
        }
    }

    Ok(())
}
