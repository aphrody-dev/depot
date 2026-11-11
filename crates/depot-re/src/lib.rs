// SPDX-License-Identifier: Apache-2.0 OR MIT
//! `depot-re`: Consolidated reverse-engineering and runtime execution analysis engine.
//! Unifies binary format triage (`goblin`), x86-64 disassembly (`iced-x86`),
//! MinGW CRT noise filtration (`rea`), and C++/Rust/Unreal ABI classification.

pub mod abi;
pub mod binary;
pub mod disasm;
pub mod rea;

pub use abi::{CppAbiClass, classify_symbol_abi};
pub use binary::{BinaryFormatReport, SectionSummary, inspect_binary_bytes};
pub use disasm::{DisassembledInstruction, disassemble_x64};
pub use rea::{MINGW_CRT_NOISE_SYMBOLS, ReaNoiseFilterReport, filter_crt_boilerplate};

use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FullBinaryAudit {
    pub path: String,
    pub format: BinaryFormatReport,
    pub noise_filter: ReaNoiseFilterReport,
    pub entry_disassembly: Vec<DisassembledInstruction>,
}

pub struct ReEngine;

impl ReEngine {
    pub fn audit_file(path: &Path) -> Result<FullBinaryAudit, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        Self::audit_bytes(&bytes, &path.to_string_lossy())
    }

    pub fn audit_bytes(bytes: &[u8], path_label: &str) -> Result<FullBinaryAudit, String> {
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
            format: format_report,
            noise_filter,
            entry_disassembly: entry_disasm,
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
}
