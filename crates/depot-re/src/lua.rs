// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Lua 5.2 Bytecode Inspector and REA Disassembler.
//! Parses PUC-Rio Lua 5.2 compiled chunks (\x1bLua\x52), validates header and types,
//! extracts prototypes, constants, and disassembles all 40 Lua 5.2 opcodes.

use serde::{Deserialize, Serialize};

pub const LUA_SIGNATURE: &[u8] = b"\x1bLua";
pub const LUAC_VERSION_52: u8 = 0x52;
pub const LUAC_DATA_52: &[u8] = b"\x19\x93\r\n\x1a\n";

pub const LUA_52_OPCODES: [&str; 40] = [
    "MOVE", "LOADK", "LOADKX", "LOADBOOL", "LOADNIL", "GETUPVAL", "GETTABUP", "GETTABLE",
    "SETTABUP", "SETUPVAL", "SETTABLE", "NEWTABLE", "SELF", "ADD", "SUB", "MUL", "DIV",
    "MOD", "POW", "UNM", "NOT", "LEN", "CONCAT", "JMP", "EQ", "LT", "LE", "TEST", "TESTSET",
    "CALL", "TAILCALL", "RETURN", "FORLOOP", "FORPREP", "TFORCALL", "TFORLOOP", "SETLIST",
    "CLOSURE", "VARARG", "EXTRAARG",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaBytecodeHeader {
    pub version: String,
    pub is_official_format: bool,
    pub is_little_endian: bool,
    pub size_int: usize,
    pub size_size_t: usize,
    pub size_instruction: usize,
    pub size_lua_number: usize,
    pub is_integral_number: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaDisassembledInstruction {
    pub pc: usize,
    pub raw: u32,
    pub opcode: String,
    pub a: u32,
    pub b: Option<u32>,
    pub c: Option<u32>,
    pub bx: Option<u32>,
    pub sbx: Option<i32>,
    pub ax: Option<u32>,
    pub formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaPrototypeAudit {
    pub source: String,
    pub line_defined: u32,
    pub last_line_defined: u32,
    pub num_params: u8,
    pub is_vararg: bool,
    pub max_stack_size: u8,
    pub instruction_count: usize,
    pub instructions: Vec<LuaDisassembledInstruction>,
    pub constants: Vec<String>,
    pub upvalues_count: usize,
    pub sub_prototypes_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lua52BytecodeAudit {
    pub header: LuaBytecodeHeader,
    pub main_prototype: LuaPrototypeAudit,
    pub total_prototypes: usize,
    pub total_instructions: usize,
}

struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
    little_endian: bool,
    size_int: usize,
    size_size_t: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            offset: 0,
            little_endian: true,
            size_int: 4,
            size_size_t: 8,
        }
    }

    fn has_bytes(&self, n: usize) -> bool {
        self.offset + n <= self.data.len()
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        if self.offset >= self.data.len() {
            return Err("Unexpected EOF reading u8".to_string());
        }
        let b = self.data[self.offset];
        self.offset += 1;
        Ok(b)
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], String> {
        if !self.has_bytes(n) {
            return Err(format!("Unexpected EOF reading {n} bytes"));
        }
        let slice = &self.data[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let bytes = self.read_bytes(4)?;
        if self.little_endian {
            Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        } else {
            Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }
    }

    fn read_size_t(&mut self) -> Result<usize, String> {
        if self.size_size_t == 4 {
            self.read_u32().map(|v| v as usize)
        } else {
            let bytes = self.read_bytes(8)?;
            let v = if self.little_endian {
                u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ])
            } else {
                u64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ])
            };
            Ok(v as usize)
        }
    }

    fn read_string(&mut self) -> Result<String, String> {
        let len = self.read_size_t()?;
        if len == 0 {
            return Ok(String::new());
        }
        // length includes null terminator
        let str_bytes = self.read_bytes(len)?;
        let clean = if !str_bytes.is_empty() && str_bytes[str_bytes.len() - 1] == 0 {
            &str_bytes[..str_bytes.len() - 1]
        } else {
            str_bytes
        };
        Ok(String::from_utf8_lossy(clean).to_string())
    }
}

/// Checks if bytes start with the Lua 5.2 signature \x1bLua\x52.
pub fn is_lua52_bytecode(bytes: &[u8]) -> bool {
    bytes.len() >= 5 && &bytes[0..4] == LUA_SIGNATURE && bytes[4] == LUAC_VERSION_52
}

/// Disassembles a raw 32-bit Lua 5.2 instruction word.
pub fn decode_lua52_instruction(pc: usize, raw: u32, constants: &[String]) -> LuaDisassembledInstruction {
    let opcode_num = (raw & 0x3F) as usize;
    let a = (raw >> 6) & 0xFF;
    let c = (raw >> 14) & 0x1FF;
    let b = (raw >> 23) & 0x1FF;
    let bx = (raw >> 14) & 0x3FFFF;
    let sbx = bx as i32 - 131_071;
    let ax = (raw >> 6) & 0x3FF_FFFF;

    let opname = if opcode_num < LUA_52_OPCODES.len() {
        LUA_52_OPCODES[opcode_num]
    } else {
        "OP_UNKNOWN"
    };

    let rk = |operand: u32| -> String {
        if operand & (1 << 8) != 0 {
            let idx = (operand & !(1 << 8)) as usize;
            constants.get(idx).map_or_else(|| format!("K{idx}?"), |s| format!("K({s})"))
        } else {
            format!("R{operand}")
        }
    };

    let formatted = match opname {
        "MOVE" => format!("R{a} := R{b}"),
        "LOADK" => {
            let k = constants.get(bx as usize).map(|s| s.as_str()).unwrap_or("?");
            format!("R{a} := K{bx} ({k})")
        }
        "LOADBOOL" => format!("R{a} := {} (skip={c})", if b != 0 { "true" } else { "false" }),
        "LOADNIL" => format!("R{a}..R{} := nil", a + b),
        "GETUPVAL" => format!("R{a} := Upvalue[{b}]"),
        "GETTABUP" => {
            let k = rk(c);
            format!("R{a} := Upvalue[{b}][{k}]")
        }
        "GETTABLE" => {
            let k = rk(c);
            format!("R{a} := R{b}[{k}]")
        }
        "SETTABUP" => {
            let k = rk(b);
            let v = rk(c);
            format!("Upvalue[{a}][{k}] := {v}")
        }
        "SETUPVAL" => format!("Upvalue[{b}] := R{a}"),
        "SETTABLE" => {
            let k = rk(b);
            let v = rk(c);
            format!("R{a}[{k}] := {v}")
        }
        "NEWTABLE" => format!("R{a} := {{}} (narr={b}, nrec={c})"),
        "SELF" => {
            let k = rk(c);
            format!("R{} := R{b}; R{a} := R{b}[{k}]", a + 1)
        }
        "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "POW" => {
            let kb = rk(b);
            let kc = rk(c);
            format!("R{a} := {kb} {opname} {kc}")
        }
        "UNM" => format!("R{a} := -R{b}"),
        "NOT" => format!("R{a} := not R{b}"),
        "LEN" => format!("R{a} := #R{b}"),
        "CONCAT" => format!("R{a} := R{b}..R{c}"),
        "JMP" => format!("jmp {} (pc -> {})", sbx, (pc as i32 + 1 + sbx).max(0)),
        "EQ" | "LT" | "LE" => {
            let kb = rk(b);
            let kc = rk(c);
            format!("if ({kb} {opname} {kc}) ~= {a} then jmp pc+1")
        }
        "CALL" => format!("call R{a} (args: {}, rets: {})", b.saturating_sub(1), c.saturating_sub(1)),
        "RETURN" => format!("return R{a}..R{}", a + b.saturating_sub(2)),
        "CLOSURE" => format!("R{a} := closure(Proto[{bx}])"),
        _ => format!("{opname} A={a} B={b} C={c} Bx={bx}"),
    };

    LuaDisassembledInstruction {
        pc,
        raw,
        opcode: opname.to_string(),
        a,
        b: Some(b),
        c: Some(c),
        bx: Some(bx),
        sbx: Some(sbx),
        ax: Some(ax),
        formatted,
    }
}

/// Parses a Lua 5.2 prototype hierarchy from reader.
fn parse_prototype(reader: &mut Reader, proto_count: &mut usize, instr_count: &mut usize) -> Result<LuaPrototypeAudit, String> {
    *proto_count += 1;
    let line_defined = reader.read_u32()?;
    let last_line_defined = reader.read_u32()?;
    let num_params = reader.read_u8()?;
    let is_vararg = reader.read_u8()? != 0;
    let max_stack_size = reader.read_u8()?;

    // Code instructions
    let code_len = reader.read_size_t()?;
    let mut raw_code = Vec::with_capacity(code_len);
    for _ in 0..code_len {
        raw_code.push(reader.read_u32()?);
    }
    *instr_count += code_len;

    // Constants
    let const_len = reader.read_size_t()?;
    let mut constants = Vec::with_capacity(const_len);
    for _ in 0..const_len {
        let tag = reader.read_u8()?;
        let val_str = match tag {
            0 => "nil".to_string(),
            1 => {
                let b = reader.read_u8()?;
                if b != 0 { "true".to_string() } else { "false".to_string() }
            }
            3 => {
                // lua_Number (f64 or i64)
                let num_bytes = reader.read_bytes(8)?;
                let f = f64::from_le_bytes([
                    num_bytes[0], num_bytes[1], num_bytes[2], num_bytes[3],
                    num_bytes[4], num_bytes[5], num_bytes[6], num_bytes[7],
                ]);
                f.to_string()
            }
            4 => {
                let s = reader.read_string()?;
                format!("\"{s}\"")
            }
            _ => format!("<unknown tag {tag}>"),
        };
        constants.push(val_str);
    }

    // Sub-prototypes
    let sub_len = reader.read_size_t()?;
    for _ in 0..sub_len {
        let _ = parse_prototype(reader, proto_count, instr_count)?;
    }

    // Upvalues
    let upval_len = reader.read_size_t()?;
    for _ in 0..upval_len {
        let _ = reader.read_u8()?; // in_stack
        let _ = reader.read_u8()?; // idx
    }

    // Debug info: source string
    let source = reader.read_string().unwrap_or_default();

    // Disassemble instructions with resolved constants
    let mut instructions = Vec::with_capacity(raw_code.len());
    for (pc, &raw) in raw_code.iter().enumerate() {
        instructions.push(decode_lua52_instruction(pc, raw, &constants));
    }

    Ok(LuaPrototypeAudit {
        source,
        line_defined,
        last_line_defined,
        num_params,
        is_vararg,
        max_stack_size,
        instruction_count: code_len,
        instructions,
        constants,
        upvalues_count: upval_len,
        sub_prototypes_count: sub_len,
    })
}

/// Fully audits and disassembles a Lua 5.2 compiled bytecode chunk.
pub fn audit_lua52_bytecode(bytes: &[u8]) -> Result<Lua52BytecodeAudit, String> {
    if !is_lua52_bytecode(bytes) {
        return Err("Not a Lua 5.2 bytecode binary (missing \\x1bLua\\x52 signature)".to_string());
    }

    let mut reader = Reader::new(bytes);
    let _ = reader.read_bytes(4)?; // \x1bLua
    let version = reader.read_u8()?; // 0x52
    let format = reader.read_u8()?;

    let data_magic = reader.read_bytes(6)?;
    let is_little_endian = data_magic[0] == 0x19;

    let size_int = reader.read_u8()? as usize;
    let size_size_t = reader.read_u8()? as usize;
    let size_instruction = reader.read_u8()? as usize;
    let size_lua_number = reader.read_u8()? as usize;
    let is_integral_number = reader.read_u8()? != 0;

    reader.little_endian = is_little_endian;
    reader.size_int = size_int;
    reader.size_size_t = size_size_t;

    let header = LuaBytecodeHeader {
        version: format!("{}.{}", version >> 4, version & 0x0F),
        is_official_format: format == 0,
        is_little_endian,
        size_int,
        size_size_t,
        size_instruction,
        size_lua_number,
        is_integral_number,
    };

    let mut total_protos = 0usize;
    let mut total_instrs = 0usize;
    let main_proto = parse_prototype(&mut reader, &mut total_protos, &mut total_instrs)?;

    Ok(Lua52BytecodeAudit {
        header,
        main_prototype: main_proto,
        total_prototypes: total_protos,
        total_instructions: total_instrs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_check() {
        assert!(is_lua52_bytecode(b"\x1bLua\x52\x00extra"));
        assert!(!is_lua52_bytecode(b"\x1bLua\x51\x00extra")); // 5.1
        assert!(!is_lua52_bytecode(b"MZ\x90\x00"));
    }

    #[test]
    fn test_instruction_decode() {
        // OP_LOADK R0 := K0
        // opcode=1 (LOADK), a=0, bx=0
        let raw = 1u32;
        let consts = vec!["\"hello\"".to_string()];
        let instr = decode_lua52_instruction(0, raw, &consts);
        assert_eq!(instr.opcode, "LOADK");
        assert_eq!(instr.a, 0);
        assert!(instr.formatted.contains("K0 (\"hello\")"));
    }
}
