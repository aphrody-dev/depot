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
    Lua52BytecodeAudit, LuaBytecodeHeader, LuaDisassembledInstruction, LuaGlobalAccess,
    LuaLocalVar, LuaPrototypeAudit, LuaUpvalueDesc, RawConstant, audit_lua52_bytecode,
    decode_lua52_instruction, is_lua52_bytecode, transcode_lua52_bytecode,
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
        // Canonical PUC-Rio Lua 5.2.4 18-byte header (lundump.c)
        let mut chunk = Vec::new();
        chunk.extend_from_slice(b"\x1bLua\x52\x00"); // Signature + version (0x52) + official format (0)
        chunk.extend_from_slice(&[1, 4, 8, 4, 8, 0]); // little-endian, sizeof(int)=4, sizeof(size_t)=8, sizeof(Instruction)=4, sizeof(lua_Number)=8, integral=0
        chunk.extend_from_slice(b"\x19\x93\r\n\x1a\n"); // LUAC_TAIL (6 bytes)

        // Main proto: line_defined=0, last_line=0, num_params=0, is_vararg=1, max_stack=2
        chunk.extend_from_slice(&0u32.to_le_bytes()); // line_defined (int = 4 bytes)
        chunk.extend_from_slice(&0u32.to_le_bytes()); // last_line (int = 4 bytes)
        chunk.push(0); // num_params
        chunk.push(1); // is_vararg
        chunk.push(2); // max_stack

        // Code: 2 instructions:
        // [0] GETTABUP R0, U0, K0 (opcode 6 = GETTABUP, A=0, B=0, C=256 (ISK|0))
        // [1] RETURN R0..R0 (opcode 31 = RETURN, A=0, B=2)
        let instr_gettabup = 6u32 | (0u32 << 6) | (256u32 << 14) | (0u32 << 23);
        let instr_return = 31u32 | (0u32 << 6) | (2u32 << 23);
        chunk.extend_from_slice(&2u32.to_le_bytes()); // code_len = 2 (int = 4 bytes)
        chunk.extend_from_slice(&instr_gettabup.to_le_bytes());
        chunk.extend_from_slice(&instr_return.to_le_bytes());

        // Constants: 1 string constant "print" (tag 4, size_t=8 including NUL)
        chunk.extend_from_slice(&1u32.to_le_bytes());
        chunk.push(4); // LUA_TSHRSTR
        chunk.extend_from_slice(&6u64.to_le_bytes()); // "print\0".len() = 6
        chunk.extend_from_slice(b"print\0");

        // Sub-prototypes: 0
        chunk.extend_from_slice(&0u32.to_le_bytes());

        // Upvalues: 1 upvalue (_ENV)
        chunk.extend_from_slice(&1u32.to_le_bytes());
        chunk.push(1); // in_stack
        chunk.push(0); // idx

        // Debug section: source="@main.lua", line_info=0, locals=0, upvalue_names=1 ("_ENV")
        chunk.extend_from_slice(&10u64.to_le_bytes()); // "@main.lua\0"
        chunk.extend_from_slice(b"@main.lua\0");
        chunk.extend_from_slice(&0u32.to_le_bytes()); // line_info count
        chunk.extend_from_slice(&0u32.to_le_bytes()); // locals count
        chunk.extend_from_slice(&1u32.to_le_bytes()); // upvalue names count
        chunk.extend_from_slice(&5u64.to_le_bytes()); // "_ENV\0"
        chunk.extend_from_slice(b"_ENV\0");

        let audit = ReEngine::audit_bytes(&chunk, "test.luac").unwrap();
        assert!(audit.is_lua_bytecode);
        let lua = audit.lua_bytecode.unwrap();
        assert_eq!(lua.header.version, "5.2");
        assert_eq!(lua.header.size_size_t, 8);
        assert_eq!(lua.total_instructions, 2);
        assert_eq!(lua.main_prototype.instructions[0].opcode, "GETTABUP");
        assert_eq!(lua.main_prototype.instructions[1].opcode, "RETURN");
        assert_eq!(lua.global_accesses.len(), 1);
        assert_eq!(lua.global_accesses[0].symbol, "print");
        assert_eq!(lua.global_accesses[0].kind, "read");

        // Verify x64 (size_t=8) -> wasm32 (size_t=4) bytecode transcoding
        let wasm_chunk = transcode_lua52_bytecode(&chunk, 4, 4).unwrap();
        let wasm_audit = ReEngine::audit_bytes(&wasm_chunk, "test_wasm32.luac").unwrap();
        let wasm_lua = wasm_audit.lua_bytecode.unwrap();
        assert_eq!(wasm_lua.header.size_size_t, 4);
        assert_eq!(wasm_lua.global_accesses.len(), 1);
        assert_eq!(wasm_lua.global_accesses[0].symbol, "print");
    }
}
