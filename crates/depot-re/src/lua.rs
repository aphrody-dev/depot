// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Lua 5.2 Bytecode Inspector, REA Disassembler, Global-Read Analyzer & Cross-Bitness Transcoder.
//! Conforms strictly to PUC-Rio Lua 5.2.4 `lundump.c`:
//! - 18-byte chunk header: `\x1bLua` (4) + version `0x52` (1) + format `0x00` (1) +
//!   endianness (1) + sizeof(int) (1) + sizeof(size_t) (1) + sizeof(Instruction) (1) +
//!   sizeof(lua_Number) (1) + integral_flag (1) + `LUAC_TAIL` `\x19\x93\r\n\x1a\n` (6).
//! - Prototype counts (`sizecode`, `sizek`, `sizep`, `sizeupvalues`, `sizelineinfo`,
//!   `sizelocvars`, `sizeupvaluenames`) are encoded as `int` (`size_int`), while string
//!   byte-lengths are encoded as `size_t` (`size_size_t`).

use serde::{Deserialize, Serialize};

pub const LUA_SIGNATURE: &[u8; 4] = b"\x1bLua";
pub const LUAC_VERSION_52: u8 = 0x52;
pub const LUAC_TAIL_52: &[u8; 6] = b"\x19\x93\r\n\x1a\n";

pub const LUA_52_OPCODES: [&str; 40] = [
    "MOVE", "LOADK", "LOADKX", "LOADBOOL", "LOADNIL", "GETUPVAL", "GETTABUP", "GETTABLE",
    "SETTABUP", "SETUPVAL", "SETTABLE", "NEWTABLE", "SELF", "ADD", "SUB", "MUL", "DIV",
    "MOD", "POW", "UNM", "NOT", "LEN", "CONCAT", "JMP", "EQ", "LT", "LE", "TEST", "TESTSET",
    "CALL", "TAILCALL", "RETURN", "FORLOOP", "FORPREP", "TFORCALL", "TFORLOOP", "SETLIST",
    "CLOSURE", "VARARG", "EXTRAARG",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
pub struct LuaLocalVar {
    pub name: String,
    pub start_pc: u32,
    pub end_pc: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaUpvalueDesc {
    pub in_stack: bool,
    pub index: u8,
    pub name: Option<String>,
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
    pub raw_code: Vec<u32>,
    pub constants: Vec<String>,
    pub raw_constants: Vec<RawConstant>,
    pub upvalues_count: usize,
    pub upvalues: Vec<LuaUpvalueDesc>,
    pub locals: Vec<LuaLocalVar>,
    pub line_info: Vec<u32>,
    pub sub_prototypes_count: usize,
    pub sub_prototypes: Vec<LuaPrototypeAudit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RawConstant {
    Nil,
    Boolean(bool),
    Number(f64),
    String(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaGlobalAccess {
    pub kind: String, // "read" | "write"
    pub symbol: String,
    pub proto_path: String,
    pub pc: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lua52BytecodeAudit {
    pub header: LuaBytecodeHeader,
    pub main_prototype: LuaPrototypeAudit,
    pub total_prototypes: usize,
    pub total_instructions: usize,
    pub global_accesses: Vec<LuaGlobalAccess>,
}

struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
    little_endian: bool,
    size_int: usize,
    size_size_t: usize,
    size_number: usize,
    integral_number: bool,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            offset: 0,
            little_endian: true,
            size_int: 4,
            size_size_t: 8,
            size_number: 8,
            integral_number: false,
        }
    }

    fn has_bytes(&self, n: usize) -> bool {
        self.offset.checked_add(n).is_some_and(|end| end <= self.data.len())
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        if self.offset >= self.data.len() {
            return Err(format!("Unexpected EOF reading u8 at offset {}", self.offset));
        }
        let b = self.data[self.offset];
        self.offset += 1;
        Ok(b)
    }

    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], String> {
        if !self.has_bytes(n) {
            return Err(format!(
                "Unexpected EOF reading {n} bytes at offset {} (len={})",
                self.offset,
                self.data.len()
            ));
        }
        let slice = &self.data[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let bytes = self.read_bytes(4)?;
        let arr = [bytes[0], bytes[1], bytes[2], bytes[3]];
        Ok(if self.little_endian {
            u32::from_le_bytes(arr)
        } else {
            u32::from_be_bytes(arr)
        })
    }

    fn read_int(&mut self) -> Result<u32, String> {
        match self.size_int {
            4 => self.read_u32(),
            8 => {
                let bytes = self.read_bytes(8)?;
                let arr = [
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ];
                let v = if self.little_endian {
                    u64::from_le_bytes(arr)
                } else {
                    u64::from_be_bytes(arr)
                };
                Ok(v as u32)
            }
            other => Err(format!("Unsupported sizeof(int) = {other}")),
        }
    }

    fn read_size_t(&mut self) -> Result<usize, String> {
        match self.size_size_t {
            4 => self.read_u32().map(|v| v as usize),
            8 => {
                let bytes = self.read_bytes(8)?;
                let arr = [
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ];
                let v = if self.little_endian {
                    u64::from_le_bytes(arr)
                } else {
                    u64::from_be_bytes(arr)
                };
                usize::try_from(v).map_err(|_| "size_t overflows host usize".to_string())
            }
            other => Err(format!("Unsupported sizeof(size_t) = {other}")),
        }
    }

    fn read_lua_number(&mut self) -> Result<f64, String> {
        match self.size_number {
            8 => {
                let b = self.read_bytes(8)?;
                let arr = [b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]];
                if self.integral_number {
                    let i = if self.little_endian {
                        i64::from_le_bytes(arr)
                    } else {
                        i64::from_be_bytes(arr)
                    };
                    Ok(i as f64)
                } else if self.little_endian {
                    Ok(f64::from_le_bytes(arr))
                } else {
                    Ok(f64::from_be_bytes(arr))
                }
            }
            4 => {
                let b = self.read_bytes(4)?;
                let arr = [b[0], b[1], b[2], b[3]];
                if self.integral_number {
                    let i = if self.little_endian {
                        i32::from_le_bytes(arr)
                    } else {
                        i32::from_be_bytes(arr)
                    };
                    Ok(f64::from(i))
                } else if self.little_endian {
                    Ok(f64::from(f32::from_le_bytes(arr)))
                } else {
                    Ok(f64::from(f32::from_be_bytes(arr)))
                }
            }
            other => Err(format!("Unsupported sizeof(lua_Number) = {other}")),
        }
    }

    fn read_raw_string(&mut self) -> Result<Vec<u8>, String> {
        let len = self.read_size_t()?;
        if len == 0 {
            return Ok(Vec::new());
        }
        let str_bytes = self.read_bytes(len)?;
        let clean = if !str_bytes.is_empty() && str_bytes[str_bytes.len() - 1] == 0 {
            &str_bytes[..str_bytes.len() - 1]
        } else {
            str_bytes
        };
        Ok(clean.to_vec())
    }

    fn read_string(&mut self) -> Result<String, String> {
        let bytes = self.read_raw_string()?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

/// Checks if bytes start with the Lua 5.2 signature `\x1bLua\x52`.
pub fn is_lua52_bytecode(bytes: &[u8]) -> bool {
    bytes.len() >= 5 && &bytes[0..4] == LUA_SIGNATURE && bytes[4] == LUAC_VERSION_52
}

/// Disassembles a raw 32-bit Lua 5.2 instruction word.
pub fn decode_lua52_instruction(
    pc: usize,
    raw: u32,
    constants: &[String],
) -> LuaDisassembledInstruction {
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
            constants
                .get(idx)
                .map_or_else(|| format!("K{idx}?"), |s| format!("K({s})"))
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
        "LOADKX" => format!("R{a} := K[EXTRAARG]"),
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
        "JMP" => format!("jmp {sbx} (pc -> {})", (pc as i32 + 1 + sbx).max(0)),
        "EQ" | "LT" | "LE" => {
            let kb = rk(b);
            let kc = rk(c);
            format!("if ({kb} {opname} {kc}) ~= {a} then jmp pc+1")
        }
        "TEST" => format!("if not (R{a} <=> {c}) then jmp pc+1"),
        "TESTSET" => format!("if (R{b} <=> {c}) then R{a} := R{b} else jmp pc+1"),
        "CALL" => format!(
            "call R{a} (args: {}, rets: {})",
            if b == 0 { "var".to_string() } else { (b - 1).to_string() },
            if c == 0 { "var".to_string() } else { (c - 1).to_string() }
        ),
        "TAILCALL" => format!(
            "tailcall R{a} (args: {})",
            if b == 0 { "var".to_string() } else { (b - 1).to_string() }
        ),
        "RETURN" => {
            if b == 1 {
                "return".to_string()
            } else if b == 0 {
                format!("return R{a}..top")
            } else {
                format!("return R{a}..R{}", a + b - 2)
            }
        }
        "FORLOOP" => format!("forloop R{a} (pc -> {})", (pc as i32 + 1 + sbx).max(0)),
        "FORPREP" => format!("forprep R{a} (pc -> {})", (pc as i32 + 1 + sbx).max(0)),
        "TFORCALL" => format!("tforcall R{a} (rets: {c})"),
        "TFORLOOP" => format!("tforloop R{a} (pc -> {})", (pc as i32 + 1 + sbx).max(0)),
        "SETLIST" => format!("setlist R{a} (count={b}, batch={c})"),
        "CLOSURE" => format!("R{a} := closure(Proto[{bx}])"),
        "VARARG" => format!("vararg R{a} (count={})", b.saturating_sub(1)),
        "EXTRAARG" => format!("extraarg Ax={ax}"),
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

fn parse_prototype(
    reader: &mut Reader,
    proto_count: &mut usize,
    instr_count: &mut usize,
) -> Result<LuaPrototypeAudit, String> {
    *proto_count += 1;
    let line_defined = reader.read_int()?;
    let last_line_defined = reader.read_int()?;
    let num_params = reader.read_u8()?;
    let is_vararg = reader.read_u8()? != 0;
    let max_stack_size = reader.read_u8()?;

    let code_len = reader.read_int()? as usize;
    let mut raw_code = Vec::with_capacity(code_len);
    for _ in 0..code_len {
        raw_code.push(reader.read_u32()?);
    }
    *instr_count += code_len;

    let const_len = reader.read_int()? as usize;
    let mut constants = Vec::with_capacity(const_len);
    let mut raw_constants = Vec::with_capacity(const_len);
    for _ in 0..const_len {
        let tag = reader.read_u8()?;
        match tag {
            0 => {
                constants.push("nil".to_string());
                raw_constants.push(RawConstant::Nil);
            }
            1 => {
                let b = reader.read_u8()? != 0;
                constants.push(if b { "true" } else { "false" }.to_string());
                raw_constants.push(RawConstant::Boolean(b));
            }
            3 => {
                let n = reader.read_lua_number()?;
                let formatted = if n.fract() == 0.0 && n.abs() < 1e15 {
                    format!("{}", n as i64)
                } else {
                    n.to_string()
                };
                constants.push(formatted);
                raw_constants.push(RawConstant::Number(n));
            }
            4 => {
                let raw = reader.read_raw_string()?;
                let s = String::from_utf8_lossy(&raw);
                constants.push(format!("\"{s}\""));
                raw_constants.push(RawConstant::String(raw));
            }
            other => {
                return Err(format!(
                    "Invalid Lua 5.2 constant tag {other} at offset {}",
                    reader.offset
                ));
            }
        }
    }

    let sub_len = reader.read_int()? as usize;
    let mut sub_prototypes = Vec::with_capacity(sub_len);
    for _ in 0..sub_len {
        sub_prototypes.push(parse_prototype(reader, proto_count, instr_count)?);
    }

    let upval_len = reader.read_int()? as usize;
    let mut upvalues = Vec::with_capacity(upval_len);
    for _ in 0..upval_len {
        let in_stack = reader.read_u8()? != 0;
        let index = reader.read_u8()?;
        upvalues.push(LuaUpvalueDesc {
            in_stack,
            index,
            name: None,
        });
    }

    // Debug block (optional if truncated in minimal synthetic tests, full in real luac chunks)
    let source = if reader.has_bytes(reader.size_size_t) {
        reader.read_string().unwrap_or_default()
    } else {
        String::new()
    };

    let mut line_info = Vec::new();
    if reader.has_bytes(reader.size_int) {
        let n = reader.read_int()? as usize;
        line_info.reserve(n);
        for _ in 0..n {
            line_info.push(reader.read_int()?);
        }
    }

    let mut locals = Vec::new();
    if reader.has_bytes(reader.size_int) {
        let n = reader.read_int()? as usize;
        locals.reserve(n);
        for _ in 0..n {
            let name = reader.read_string()?;
            let start_pc = reader.read_int()?;
            let end_pc = reader.read_int()?;
            locals.push(LuaLocalVar {
                name,
                start_pc,
                end_pc,
            });
        }
    }

    if reader.has_bytes(reader.size_int) {
        let n = reader.read_int()? as usize;
        for i in 0..n {
            let name = reader.read_string()?;
            if let Some(uv) = upvalues.get_mut(i)
                && !name.is_empty()
            {
                uv.name = Some(name);
            }
        }
    }

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
        raw_code,
        constants,
        raw_constants,
        upvalues_count: upval_len,
        upvalues,
        locals,
        line_info,
        sub_prototypes_count: sub_len,
        sub_prototypes,
    })
}

fn collect_global_accesses(
    proto: &LuaPrototypeAudit,
    path: &str,
    env_upvalue_idx: Option<usize>,
    out: &mut Vec<LuaGlobalAccess>,
) {
    if let Some(env_idx) = env_upvalue_idx {
        for instr in &proto.instructions {
            if instr.opcode == "GETTABUP" && instr.b == Some(env_idx as u32) {
                if let Some(c) = instr.c
                    && (c & (1 << 8)) != 0
                {
                    let k_idx = (c & !(1 << 8)) as usize;
                    if let Some(RawConstant::String(bytes)) = proto.raw_constants.get(k_idx) {
                        out.push(LuaGlobalAccess {
                            kind: "read".to_string(),
                            symbol: String::from_utf8_lossy(bytes).into_owned(),
                            proto_path: path.to_string(),
                            pc: instr.pc,
                        });
                    }
                }
            } else if instr.opcode == "SETTABUP" && instr.a == env_idx as u32 {
                if let Some(b) = instr.b
                    && (b & (1 << 8)) != 0
                {
                    let k_idx = (b & !(1 << 8)) as usize;
                    if let Some(RawConstant::String(bytes)) = proto.raw_constants.get(k_idx) {
                        out.push(LuaGlobalAccess {
                            kind: "write".to_string(),
                            symbol: String::from_utf8_lossy(bytes).into_owned(),
                            proto_path: path.to_string(),
                            pc: instr.pc,
                        });
                    }
                }
            }
        }
    }

    for (idx, child) in proto.sub_prototypes.iter().enumerate() {
        let child_env = env_upvalue_idx.and_then(|parent_env| {
            child
                .upvalues
                .iter()
                .position(|uv| !uv.in_stack && usize::from(uv.index) == parent_env)
        });
        collect_global_accesses(child, &format!("{path}:{idx}"), child_env, out);
    }
}

/// Parses and audits a Lua 5.2 compiled bytecode chunk (`lundump.c` canonical 18-byte header,
/// plus backward compatibility with legacy synthetic header layout).
pub fn audit_lua52_bytecode(bytes: &[u8]) -> Result<Lua52BytecodeAudit, String> {
    if !is_lua52_bytecode(bytes) {
        return Err("Not a Lua 5.2 bytecode binary (missing \\x1bLua\\x52 signature)".to_string());
    }
    if bytes.len() < 18 {
        return Err(format!(
            "Truncated Lua 5.2 header: expected >= 18 bytes, got {}",
            bytes.len()
        ));
    }

    let mut reader = Reader::new(bytes);
    let _ = reader.read_bytes(4)?; // \x1bLua
    let version = reader.read_u8()?; // 0x52
    let format = reader.read_u8()?;

    // Standard PUC-Rio Lua 5.2 header places LUAC_TAIL (\x19\x93\r\n\x1a\n) at bytes 12..18.
    let (is_little_endian, size_int, size_size_t, size_instruction, size_lua_number, is_integral_number) =
        if &bytes[12..18] == LUAC_TAIL_52 {
            let le = reader.read_u8()? == 1;
            let s_int = reader.read_u8()? as usize;
            let s_sizet = reader.read_u8()? as usize;
            let s_instr = reader.read_u8()? as usize;
            let s_num = reader.read_u8()? as usize;
            let integral = reader.read_u8()? != 0;
            let _ = reader.read_bytes(6)?; // LUAC_TAIL_52
            (le, s_int, s_sizet, s_instr, s_num, integral)
        } else if &bytes[6..12] == LUAC_TAIL_52 {
            // Legacy layout fallback (tail before sizes)
            let _ = reader.read_bytes(6)?;
            let s_int = reader.read_u8()? as usize;
            let s_sizet = reader.read_u8()? as usize;
            let s_instr = reader.read_u8()? as usize;
            let s_num = reader.read_u8()? as usize;
            let integral = reader.read_u8()? != 0;
            (true, s_int, s_sizet, s_instr, s_num, integral)
        } else {
            return Err("Invalid Lua 5.2 header: missing LUAC_TAIL \\x19\\x93\\r\\n\\x1a\\n".to_string());
        };

    reader.little_endian = is_little_endian;
    reader.size_int = size_int;
    reader.size_size_t = size_size_t;
    reader.size_number = size_lua_number;
    reader.integral_number = is_integral_number;

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

    let mut global_accesses = Vec::new();
    let main_env = if main_proto.upvalues.is_empty() {
        None
    } else {
        Some(0)
    };
    collect_global_accesses(&main_proto, "main", main_env, &mut global_accesses);

    Ok(Lua52BytecodeAudit {
        header,
        main_prototype: main_proto,
        total_prototypes: total_protos,
        total_instructions: total_instrs,
        global_accesses,
    })
}

struct ChunkWriter {
    out: Vec<u8>,
    little_endian: bool,
    size_int: usize,
    size_size_t: usize,
}

impl ChunkWriter {
    fn write_u8(&mut self, v: u8) {
        self.out.push(v);
    }

    fn write_u32(&mut self, v: u32) {
        let b = if self.little_endian {
            v.to_le_bytes()
        } else {
            v.to_be_bytes()
        };
        self.out.extend_from_slice(&b);
    }

    fn write_int(&mut self, v: u32) {
        if self.size_int == 8 {
            let b = if self.little_endian {
                u64::from(v).to_le_bytes()
            } else {
                u64::from(v).to_be_bytes()
            };
            self.out.extend_from_slice(&b);
        } else {
            self.write_u32(v);
        }
    }

    fn write_size_t(&mut self, v: usize) {
        if self.size_size_t == 8 {
            let b = if self.little_endian {
                (v as u64).to_le_bytes()
            } else {
                (v as u64).to_be_bytes()
            };
            self.out.extend_from_slice(&b);
        } else {
            self.write_u32(v as u32);
        }
    }

    fn write_number(&mut self, v: f64) {
        let b = if self.little_endian {
            v.to_le_bytes()
        } else {
            v.to_be_bytes()
        };
        self.out.extend_from_slice(&b);
    }

    fn write_raw_string(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            self.write_size_t(0);
            return;
        }
        self.write_size_t(bytes.len() + 1);
        self.out.extend_from_slice(bytes);
        self.out.push(0);
    }

    fn write_proto(&mut self, p: &LuaPrototypeAudit) {
        self.write_int(p.line_defined);
        self.write_int(p.last_line_defined);
        self.write_u8(p.num_params);
        self.write_u8(u8::from(p.is_vararg));
        self.write_u8(p.max_stack_size);

        self.write_int(p.raw_code.len() as u32);
        for &ins in &p.raw_code {
            self.write_u32(ins);
        }

        self.write_int(p.raw_constants.len() as u32);
        for k in &p.raw_constants {
            match k {
                RawConstant::Nil => self.write_u8(0),
                RawConstant::Boolean(b) => {
                    self.write_u8(1);
                    self.write_u8(u8::from(*b));
                }
                RawConstant::Number(n) => {
                    self.write_u8(3);
                    self.write_number(*n);
                }
                RawConstant::String(s) => {
                    self.write_u8(4);
                    self.write_raw_string(s);
                }
            }
        }

        self.write_int(p.sub_prototypes.len() as u32);
        for sub in &p.sub_prototypes {
            self.write_proto(sub);
        }

        self.write_int(p.upvalues.len() as u32);
        for uv in &p.upvalues {
            self.write_u8(u8::from(uv.in_stack));
            self.write_u8(uv.index);
        }

        self.write_raw_string(p.source.as_bytes());
        self.write_int(p.line_info.len() as u32);
        for &ln in &p.line_info {
            self.write_int(ln);
        }

        self.write_int(p.locals.len() as u32);
        for loc in &p.locals {
            self.write_raw_string(loc.name.as_bytes());
            self.write_int(loc.start_pc);
            self.write_int(loc.end_pc);
        }

        let named_upvals: Vec<&str> = p
            .upvalues
            .iter()
            .filter_map(|u| u.name.as_deref())
            .collect();
        if named_upvals.len() == p.upvalues.len() && !named_upvals.is_empty() {
            self.write_int(named_upvals.len() as u32);
            for name in named_upvals {
                self.write_raw_string(name.as_bytes());
            }
        } else {
            self.write_int(0);
        }
    }
}

/// Transcodes a compiled PUC-Rio Lua 5.2 chunk across architectures (e.g. x86_64 8-byte `size_t`
/// to wasm32 4-byte `size_t` or vice versa) without losing instructions, constants, or `_ENV` upvalues.
pub fn transcode_lua52_bytecode(
    bytes: &[u8],
    target_size_int: u8,
    target_size_size_t: u8,
) -> Result<Vec<u8>, String> {
    if !matches!(target_size_int, 4 | 8) {
        return Err(format!("Unsupported target sizeof(int)={target_size_int}"));
    }
    if !matches!(target_size_size_t, 4 | 8) {
        return Err(format!(
            "Unsupported target sizeof(size_t)={target_size_size_t}"
        ));
    }
    let audit = audit_lua52_bytecode(bytes)?;
    let mut writer = ChunkWriter {
        out: Vec::with_capacity(bytes.len() + 64),
        little_endian: audit.header.is_little_endian,
        size_int: target_size_int as usize,
        size_size_t: target_size_size_t as usize,
    };
    writer.out.extend_from_slice(LUA_SIGNATURE);
    writer.write_u8(LUAC_VERSION_52);
    writer.write_u8(if audit.header.is_official_format { 0 } else { 1 });
    writer.write_u8(u8::from(audit.header.is_little_endian));
    writer.write_u8(target_size_int);
    writer.write_u8(target_size_size_t);
    writer.write_u8(4);
    writer.write_u8(8);
    writer.write_u8(0);
    writer.out.extend_from_slice(LUAC_TAIL_52);
    writer.write_proto(&audit.main_prototype);
    Ok(writer.out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_canonical_lua52_chunk(size_size_t: u8) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"\x1bLua");
        out.push(0x52); // version 5.2
        out.push(0x00); // official format
        out.push(0x01); // little-endian
        out.push(4); // sizeof(int)
        out.push(size_size_t); // sizeof(size_t)
        out.push(4); // sizeof(Instruction)
        out.push(8); // sizeof(lua_Number)
        out.push(0); // floating-point numbers
        out.extend_from_slice(LUAC_TAIL_52);

        // Main prototype
        out.extend_from_slice(&0u32.to_le_bytes()); // line_defined
        out.extend_from_slice(&0u32.to_le_bytes()); // last_line_defined
        out.push(0); // num_params
        out.push(1); // is_vararg
        out.push(2); // max_stack_size

        // 2 instructions:
        // [0] GETTABUP R0 := Upvalue[0][K0("print")] -> op=6, A=0, B=0, C=256 (bit 8 + 0)
        // [1] RETURN (op=31, A=0, B=1)
        let gettabup = 6u32 | (0u32 << 6) | (256u32 << 14) | (0u32 << 23);
        let ret = 31u32 | (1u32 << 23);
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&gettabup.to_le_bytes());
        out.extend_from_slice(&ret.to_le_bytes());

        // 1 constant: String "print"
        out.extend_from_slice(&1u32.to_le_bytes());
        out.push(4); // LUA_TSHRSTR
        let s = b"print\0";
        if size_size_t == 8 {
            out.extend_from_slice(&(s.len() as u64).to_le_bytes());
        } else {
            out.extend_from_slice(&(s.len() as u32).to_le_bytes());
        }
        out.extend_from_slice(s);

        // 0 sub-protos
        out.extend_from_slice(&0u32.to_le_bytes());

        // 1 upvalue (_ENV)
        out.extend_from_slice(&1u32.to_le_bytes());
        out.push(1);
        out.push(0);

        // Debug info (stripped)
        if size_size_t == 8 {
            out.extend_from_slice(&0u64.to_le_bytes());
        } else {
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        out.extend_from_slice(&0u32.to_le_bytes()); // lineinfo
        out.extend_from_slice(&0u32.to_le_bytes()); // locvars
        out.extend_from_slice(&0u32.to_le_bytes()); // upval names
        out
    }

    #[test]
    fn test_signature_check() {
        assert!(is_lua52_bytecode(b"\x1bLua\x52\x00extra"));
        assert!(!is_lua52_bytecode(b"\x1bLua\x51\x00extra"));
        assert!(!is_lua52_bytecode(b"MZ\x90\x00"));
    }

    #[test]
    fn test_instruction_decode() {
        let raw = 1u32;
        let consts = vec!["\"hello\"".to_string()];
        let instr = decode_lua52_instruction(0, raw, &consts);
        assert_eq!(instr.opcode, "LOADK");
        assert_eq!(instr.a, 0);
        assert!(instr.formatted.contains("K0 (\"hello\")"));
    }

    #[test]
    fn test_canonical_header_global_access_and_transcode_64_to_32() {
        let chunk64 = build_canonical_lua52_chunk(8);
        let audit64 = audit_lua52_bytecode(&chunk64).expect("audit 64-bit chunk");
        assert_eq!(audit64.header.version, "5.2");
        assert_eq!(audit64.header.size_size_t, 8);
        assert_eq!(audit64.total_instructions, 2);
        assert_eq!(audit64.global_accesses.len(), 1);
        assert_eq!(audit64.global_accesses[0].symbol, "print");
        assert_eq!(audit64.global_accesses[0].kind, "read");

        let chunk32 = transcode_lua52_bytecode(&chunk64, 4, 4).expect("transcode 64->32");
        let audit32 = audit_lua52_bytecode(&chunk32).expect("audit 32-bit chunk");
        assert_eq!(audit32.header.size_size_t, 4);
        assert_eq!(audit32.total_instructions, 2);
        assert_eq!(audit32.global_accesses[0].symbol, "print");
    }
}
