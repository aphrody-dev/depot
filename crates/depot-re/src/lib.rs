// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `depot-re`: Consolidated reverse-engineering and runtime execution analysis engine.
//! Unifies binary format triage (`goblin`), x86-64 disassembly (`iced-x86`),
//! MinGW CRT noise filtration (`rea`), C++/Rust/Unreal ABI classification,
//! and Lua 5.2 compiled bytecode disassembly & REA audit.

pub mod abi;
pub mod binary;
pub mod disasm;
pub mod lua;
pub mod rea;

pub use abi::{CppAbiClass, classify_symbol_abi};
pub use binary::{BinaryFormatReport, SectionSummary, inspect_binary_bytes};
pub use disasm::{DisassembledInstruction, disassemble_x64};
pub use lua::{
    Lua52BytecodeAudit, LuaBytecodeHeader, LuaDisassembledInstruction, LuaPrototypeAudit,
    audit_lua52_bytecode, decode_lua52_instruction, is_lua52_bytecode,
};
pub use rea::{MINGW_CRT_NOISE_SYMBOLS, ReaNoiseFilterReport, filter_crt_boilerplate};

use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FullBinaryAudit {
    pub path: String,
    pub is_lua_bytecode: bool,
    pub format: Option<BinaryFormatReport>,
    pub noise_filter: Option<ReaNoiseFilterReport>,
    pub entry_disassembly: Vec<DisassembledInstruction>,
    pub lua_bytecode: Option<Lua52BytecodeAudit>,
}

pub struct ReEngine;

impl ReEngine {
    pub fn audit_file(path: &Path) -> Result<FullBinaryAudit, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        Self::audit_bytes(&bytes, &path.to_string_lossy())
    }

    pub fn audit_bytes(bytes: &[u8], path_label: &str) -> Result<FullBinaryAudit, String> {
        if is_lua52_bytecode(bytes) {
            let lua_audit = audit_lua52_bytecode(bytes)?;
            return Ok(FullBinaryAudit {
                path: path_label.to_string(),
                is_lua_bytecode: true,
                format: None,
                noise_filter: None,
                entry_disassembly: Vec::new(),
                lua_bytecode: Some(lua_audit),
            });
        }

        let format_report = inspect_binary_bytes(bytes)?;
        let noise_filter = filter_crt_boilerplate(&format_report.exports);

        // Disassemble first 16 instructions from the entry point if possible
        let entry_disasm = if format_report.is_64_bit && !bytes.is_empty() {
            let offset = (format_report.entry_point as usize).min(bytes.len().saturating_sub(64));
            disassemble_x64(&bytes[offset..], format_report.entry_point, 16)
        } else {
            Vec::new()
        };

        Ok(FullBinaryAudit {
            path: path_label.to_string(),
            is_lua_bytecode: false,
            format: Some(format_report),
            noise_filter: Some(noise_filter),
            entry_disassembly: entry_disasm,
            lua_bytecode: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_re_engine_symbols() {
        let syms = vec![
            "mainCRTStartup".to_string(),
            "_DllMainCRTStartup".to_string(),
            "my_payload_calc".to_string(),
            "?real_work@@YAHXZ".to_string(),
        ];
        let rep = filter_crt_boilerplate(&syms);
        assert_eq!(rep.noise_symbols_count, 2);
        assert_eq!(rep.payload_symbols_count, 2);
        assert_eq!(classify_symbol_abi(&rep.payload_symbols[1]), CppAbiClass::MsvcCpp);
    }

    #[test]
    fn test_lua52_bytecode_audit() {
        // Minimal fake Lua 5.2 header
        let mut chunk = Vec::new();
        chunk.extend_from_slice(b"\x1bLua\x52\x00"); // Signature + version + format
        chunk.extend_from_slice(b"\x19\x93\r\n\x1a\n"); // LUAC_DATA
        chunk.extend_from_slice(&[4, 8, 4, 8, 0]); // sizeof int, size_t, Instruction, lua_Number, integral
        // Main proto: line_defined=0, last_line=0, num_params=0, is_vararg=1, max_stack=2
        chunk.extend_from_slice(&[0, 0, 0, 0]); // line_defined
        chunk.extend_from_slice(&[0, 0, 0, 0]); // last_line
        chunk.push(0); // num_params
        chunk.push(1); // is_vararg
        chunk.push(2); // max_stack
        // Code: 1 instruction: RETURN R0..R0 (opcode 31 = RETURN, a=0, b=1)
        // raw = 31 | (0 << 6) | (1 << 23)
        let raw_instr = 31u32 | (1u32 << 23);
        chunk.extend_from_slice(&1u64.to_le_bytes()); // code_len = 1
        chunk.extend_from_slice(&raw_instr.to_le_bytes());
        // Constants: 0
        chunk.extend_from_slice(&0u64.to_le_bytes());
        // Sub-prototypes: 0
        chunk.extend_from_slice(&0u64.to_le_bytes());
        // Upvalues: 1 upvalue (_ENV)
        chunk.extend_from_slice(&1u64.to_le_bytes());
        chunk.push(1); // in_stack
        chunk.push(0); // idx
        // Source string: 0
        chunk.extend_from_slice(&0u64.to_le_bytes());

        let audit = ReEngine::audit_bytes(&chunk, "test.luac").unwrap();
        assert!(audit.is_lua_bytecode);
        let lua = audit.lua_bytecode.unwrap();
        assert_eq!(lua.header.version, "5.2");
        assert_eq!(lua.total_instructions, 1);
        assert_eq!(lua.main_prototype.instructions[0].opcode, "RETURN");
    }
}
