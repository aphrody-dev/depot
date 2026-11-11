// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Binary format triage (PE, ELF, Mach-O, WASM) via Goblin.

use goblin::Object;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryFormatReport {
    pub format: String,
    pub is_64_bit: bool,
    pub entry_point: u64,
    pub sections: Vec<SectionSummary>,
    pub exports: Vec<String>,
    pub imports: Vec<String>,
    pub has_dwarf_debug: bool,
    pub has_rea_marker: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSummary {
    pub name: String,
    pub virtual_address: u64,
    pub size: u64,
}

/// Parses raw binary bytes and returns a structured triage report.
pub fn inspect_binary_bytes(bytes: &[u8]) -> Result<BinaryFormatReport, String> {
    match Object::parse(bytes).map_err(|e| format!("parse error: {e}"))? {
        Object::PE(pe) => {
            let mut sections = Vec::new();
            let mut has_dwarf = false;
            let mut has_rea = false;

            for s in &pe.sections {
                let name = s.name().unwrap_or("").to_string();
                if name.starts_with(".debug") || name == ".eh_frame" {
                    has_dwarf = true;
                }
                if name == ".rea_sig" || name.contains("rea") {
                    has_rea = true;
                }
                sections.push(SectionSummary {
                    name,
                    virtual_address: s.virtual_address as u64,
                    size: s.virtual_size as u64,
                });
            }

            let exports = pe.exports.iter().filter_map(|e| e.name.map(|s| s.to_string())).collect();
            let imports = pe.imports.iter().map(|i| format!("{}:{}", i.dll, i.name)).collect();

            Ok(BinaryFormatReport {
                format: "PE".to_string(),
                is_64_bit: pe.is_64,
                entry_point: pe.entry as u64,
                sections,
                exports,
                imports,
                has_dwarf_debug: has_dwarf,
                has_rea_marker: has_rea,
            })
        }
        Object::Elf(elf) => {
            let mut sections = Vec::new();
            let mut has_dwarf = false;

            for h in &elf.section_headers {
                let name = elf.shdr_strtab.get_at(h.sh_name).unwrap_or("").to_string();
                if name.starts_with(".debug") {
                    has_dwarf = true;
                }
                sections.push(SectionSummary {
                    name,
                    virtual_address: h.sh_addr,
                    size: h.sh_size,
                });
            }

            let exports = elf.syms.iter().filter_map(|s| elf.strtab.get_at(s.st_name).map(|s| s.to_string())).collect();
            let imports = elf.libraries.iter().map(|s| s.to_string()).collect();

            Ok(BinaryFormatReport {
                format: "ELF".to_string(),
                is_64_bit: elf.is_64,
                entry_point: elf.entry,
                sections,
                exports,
                imports,
                has_dwarf_debug: has_dwarf,
                has_rea_marker: false,
            })
        }
        Object::Mach(mach) => match mach {
            goblin::mach::Mach::Binary(m) => {
                let mut sections = Vec::new();
                for s in m.segments.sections().flatten() {
                    if let Ok((sec, _)) = s {
                        sections.push(SectionSummary {
                            name: sec.name().unwrap_or("").to_string(),
                            virtual_address: sec.addr,
                            size: sec.size,
                        });
                    }
                }
                Ok(BinaryFormatReport {
                    format: "Mach-O".to_string(),
                    is_64_bit: m.is_64,
                    entry_point: m.entry,
                    sections,
                    exports: m.exports().unwrap_or_default().into_iter().map(|e| e.name).collect(),
                    imports: m.libs.iter().map(|s| s.to_string()).collect(),
                    has_dwarf_debug: false,
                    has_rea_marker: false,
                })
            }
            _ => Err("Fat Mach-O not yet unpacked".to_string()),
        },
        _ => Err("Unsupported binary format".to_string()),
    }
}
