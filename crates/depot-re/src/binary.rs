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
    let parsed = Object::parse(bytes);
    if let Err(ref e) = parsed {
        if bytes.starts_with(b"MZ") {
            return parse_pe_fallback(bytes);
        }
        return Err(format!("parse error: {e}"));
    }

    match parsed.unwrap() {
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

fn parse_pe_fallback(bytes: &[u8]) -> Result<BinaryFormatReport, String> {
    if bytes.len() < 0x40 || &bytes[0..2] != b"MZ" {
        return Err("Not a PE binary (missing MZ)".to_string());
    }
    let pe_offset = u32::from_le_bytes(bytes[0x3C..0x40].try_into().unwrap()) as usize;
    if pe_offset + 24 > bytes.len() || &bytes[pe_offset..pe_offset + 4] != b"PE\0\0" {
        return Err("Invalid PE signature".to_string());
    }
    let machine = u16::from_le_bytes(bytes[pe_offset + 4..pe_offset + 6].try_into().unwrap());
    let is_64_bit = machine == 0x8664;
    let num_sections = u16::from_le_bytes(bytes[pe_offset + 6..pe_offset + 8].try_into().unwrap()) as usize;
    let opt_header_size = u16::from_le_bytes(bytes[pe_offset + 20..pe_offset + 22].try_into().unwrap()) as usize;

    let opt_offset = pe_offset + 24;
    let entry_point = if opt_header_size >= 20 && opt_offset + 20 <= bytes.len() {
        u32::from_le_bytes(bytes[opt_offset + 16..opt_offset + 20].try_into().unwrap()) as u64
    } else {
        0
    };

    let mut sections = Vec::new();
    let mut has_dwarf = false;
    let mut has_rea = false;
    let sec_table_offset = opt_offset + opt_header_size;

    for i in 0..num_sections {
        let sec_offset = sec_table_offset + i * 40;
        if sec_offset + 40 > bytes.len() {
            break;
        }
        let name_bytes = &bytes[sec_offset..sec_offset + 8];
        let name = String::from_utf8_lossy(name_bytes).trim_matches('\0').to_string();
        if name.starts_with(".debug") || name == ".eh_frame" {
            has_dwarf = true;
        }
        if name == ".rea_sig" || name.contains("rea") {
            has_rea = true;
        }
        let virt_size = u32::from_le_bytes(bytes[sec_offset + 8..sec_offset + 12].try_into().unwrap()) as u64;
        let virt_addr = u32::from_le_bytes(bytes[sec_offset + 12..sec_offset + 16].try_into().unwrap()) as u64;
        sections.push(SectionSummary {
            name,
            virtual_address: virt_addr,
            size: virt_size,
        });
    }

    Ok(BinaryFormatReport {
        format: "PE (Protected/Overlay)".to_string(),
        is_64_bit,
        entry_point,
        sections,
        exports: Vec::new(),
        imports: Vec::new(),
        has_dwarf_debug: has_dwarf,
        has_rea_marker: has_rea,
    })
}
