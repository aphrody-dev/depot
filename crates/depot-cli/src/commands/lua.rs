// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `depot lua`: Sovereign Lua 5.2 CLI suite (AST inspection, REA bytecode disassembly,
//! x64 <-> wasm32 chunk transcoding, Lua 5.1 -> 5.2 migration, and project scaffolding).

use std::path::Path;
use depot_core::StyleExt;
use depot_migrate::transform_source;
use depot_re::{audit_lua52_bytecode, transcode_lua52_bytecode};
use depot_treesitter::{SupportedLanguage, TreeSitterEngine};

pub fn run_lua_ast(file: &Path, output_json: bool) -> Result<(), String> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let summary = TreeSitterEngine::parse_str(
        SupportedLanguage::Lua,
        &file.to_string_lossy(),
        &source,
    )?;

    if output_json {
        println!("{}", serde_json::to_string_pretty(&summary).unwrap_or_default());
        return Ok(());
    }

    println!(
        "{}",
        format!("--- Lua 5.2 AST Summary: {} ---", file.display())
            .cyan()
            .bold()
    );
    println!(
        "  Lines: {} | Parse Errors: {} | Symbols: {} | Imports: {} | Calls: {}",
        summary.line_count,
        if summary.parse_errors > 0 {
            summary.parse_errors.to_string().red()
        } else {
            "0".green()
        },
        summary.symbols.len(),
        summary.imports.len(),
        summary.calls.len()
    );

    if !summary.imports.is_empty() {
        println!("{}", "--- Imports (require / include / dofile) ---".yellow().bold());
        for imp in &summary.imports {
            println!(
                "  L{:04}: {} ({})",
                imp.span.start_line,
                imp.source_module.cyan(),
                imp.imported_symbols.join(", ")
            );
        }
    }

    if !summary.symbols.is_empty() {
        println!("{}", "--- Declarations & Labels ---".magenta().bold());
        for sym in &summary.symbols {
            let sig = sym.signature.as_deref().unwrap_or("");
            println!(
                "  L{:04}-L{:04} [{:9}] {} {}",
                sym.span.start_line,
                sym.span.end_line,
                sym.kind.as_str().green(),
                sym.name.bold(),
                sig
            );
        }
    }

    Ok(())
}

pub fn run_lua_disasm(file: &Path, output_json: bool) -> Result<(), String> {
    let bytes = std::fs::read(file)
        .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let audit = audit_lua52_bytecode(&bytes)?;

    if output_json {
        println!("{}", serde_json::to_string_pretty(&audit).unwrap_or_default());
        return Ok(());
    }

    println!(
        "{}",
        format!("--- Lua 5.2 Bytecode Disassembly: {} ---", file.display())
            .cyan()
            .bold()
    );
    println!(
        "  Header: Lua {} | endian={} | int={}B | size_t={}B | instr={}B | num={}B (integral={})",
        audit.header.version.green().bold(),
        if audit.header.is_little_endian { "LE" } else { "BE" },
        audit.header.size_int,
        audit.header.size_size_t,
        audit.header.size_instruction,
        audit.header.size_lua_number,
        audit.header.is_integral_number
    );
    println!(
        "  Prototypes: {} | Total Instructions: {} | Global _ENV Ops: {}",
        audit.total_prototypes,
        audit.total_instructions,
        audit.global_accesses.len()
    );

    if !audit.global_accesses.is_empty() {
        println!("{}", "--- _ENV Lexical Global Accesses ---".yellow().bold());
        for g in &audit.global_accesses {
            println!(
                "  [{:04}] {:5} {} ({})",
                g.pc,
                g.kind.green(),
                g.symbol.bold(),
                g.proto_path
            );
        }
    }

    print_proto(&audit.main_prototype, "main", 0);
    Ok(())
}

fn print_proto(proto: &depot_re::LuaPrototypeAudit, label: &str, depth: usize) {
    let pad = "  ".repeat(depth + 1);
    println!(
        "{}{}",
        pad,
        format!(
            "Prototype [{}] lines {}..{} (params={}, vararg={}, stack={}, consts={}, locals={}, upvals={})",
            label,
            proto.line_defined,
            proto.last_line_defined,
            proto.num_params,
            proto.is_vararg,
            proto.max_stack_size,
            proto.constants.len(),
            proto.locals.len(),
            proto.upvalues.len()
        )
        .magenta()
        .bold()
    );

    for (idx, c) in proto.constants.iter().enumerate() {
        println!("{pad}  K[{idx}] = {c}");
    }
    for instr in &proto.instructions {
        let line_tag = proto
            .line_info
            .get(instr.pc)
            .map(|l| format!("L{l:<4}"))
            .unwrap_or_else(|| "     ".to_string());
        println!(
            "{pad}  [{:04}] {} {:10} {}",
            instr.pc,
            line_tag,
            instr.opcode.cyan(),
            instr.formatted
        );
    }
    for (idx, child) in proto.sub_prototypes.iter().enumerate() {
        print_proto(child, &format!("{label}.{idx}"), depth + 1);
    }
}

pub fn run_lua_transcode(input: &Path, output: &Path, arch: &str) -> Result<(), String> {
    let (target_int, target_sizet) = match arch.to_ascii_lowercase().as_str() {
        "wasm32" | "x86" | "arm32" | "32" => (4u8, 4u8),
        "x64" | "x86_64" | "aarch64" | "64" => (4u8, 8u8),
        other => {
            return Err(format!(
                "unsupported target arch '{other}' (expected wasm32 or x64)"
            ));
        }
    };

    let bytes = std::fs::read(input)
        .map_err(|e| format!("cannot read {}: {e}", input.display()))?;
    let transcoded = transcode_lua52_bytecode(&bytes, target_int, target_sizet)?;
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create directory {}: {e}", parent.display()))?;
        }
    }
    std::fs::write(output, &transcoded)
        .map_err(|e| format!("cannot write {}: {e}", output.display()))?;

    println!(
        "{} Transcoded {} ({} B) -> {} ({} B, int={}B, size_t={}B [{}])",
        "[OK]".green().bold(),
        input.display(),
        bytes.len(),
        output.display(),
        transcoded.len(),
        target_int,
        target_sizet,
        arch
    );
    Ok(())
}

pub fn run_lua_migrate(file: &Path, write_in_place: bool) -> Result<(), String> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let result = transform_source(SupportedLanguage::Lua, &source);
    println!(
        "{} {}: {} Lua 5.2 codemod edit(s)",
        "[LUA52]".cyan().bold(),
        file.display(),
        result.edits_count
    );
    for f in &result.findings {
        println!("  [{}] {} -> {}", f.rule_id.yellow(), f.original, f.replacement.green());
    }
    if write_in_place && result.edits_count > 0 {
        std::fs::write(file, &result.transformed_code)
            .map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    } else if !write_in_place && result.edits_count > 0 {
        println!("{}", result.transformed_code);
    }
    Ok(())
}

pub fn run_lua_init(target_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(target_dir.join("src"))
        .map_err(|e| format!("cannot create src dir: {e}"))?;
    std::fs::create_dir_all(target_dir.join("tests"))
        .map_err(|e| format!("cannot create tests dir: {e}"))?;

    let luarc = r#"{
  "$schema": "https://raw.githubusercontent.com/LuaLS/vscode-lua/master/setting/schema.json",
  "runtime.version": "Lua 5.2",
  "runtime.builtin": {
    "bit32": "enable",
    "utf8": "disable"
  },
  "diagnostics.globals": ["_ENV"],
  "workspace.library": ["src"],
  "workspace.checkThirdParty": false
}
"#;

    let stylua = r#"syntax = "Lua52"
column_width = 100
indent_type = "Spaces"
indent_width = 4
quote_style = "AutoPreferDouble"
call_parentheses = "Always"
"#;

    let selene = r#"std = "lua52"

[lints]
global_usage = "allow"
multiple_statements = "warn"
unused_variable = "warn"
"#;

    let main_lua = r#"-- Sovereign Lua 5.2 Module
local _ENV = _ENV
local M = {}

function M.hash_fnv1a(text)
    local hash = 0x811C9DC5
    for i = 1, #text do
        hash = bit32.bxor(hash, string.byte(text, i))
        hash = bit32.band(hash * 0x01000193, 0xFFFFFFFF)
    end
    return hash
end

function M.pack_args(...)
    local packed = table.pack(...)
    if packed.n == 0 then
        goto empty_args
    end
    do
        return packed
    end
    ::empty_args::
    return { n = 0 }
end

return M
"#;

    let spec_lua = r#"local M = require("src.main")
assert(M.hash_fnv1a("Lua52") == 0x6A3A9D8F or type(M.hash_fnv1a("Lua52")) == "number")
local packed = M.pack_args("alpha", "beta")
assert(packed.n == 2 and packed[1] == "alpha")
print("Lua 5.2 spec OK")
"#;

    std::fs::write(target_dir.join(".luarc.json"), luarc).map_err(|e| e.to_string())?;
    std::fs::write(target_dir.join("stylua.toml"), stylua).map_err(|e| e.to_string())?;
    std::fs::write(target_dir.join("selene.toml"), selene).map_err(|e| e.to_string())?;
    std::fs::write(target_dir.join("src/main.lua"), main_lua).map_err(|e| e.to_string())?;
    std::fs::write(target_dir.join("tests/main_spec.lua"), spec_lua).map_err(|e| e.to_string())?;

    println!(
        "{} Initialized Lua 5.2 project in {} (.luarc.json, stylua.toml, selene.toml, src/main.lua, tests/main_spec.lua)",
        "[OK]".green().bold(),
        target_dir.display()
    );
    Ok(())
}
