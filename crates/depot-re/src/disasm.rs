// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Disassembly engine using iced-x86.

use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, IntelFormatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisassembledInstruction {
    pub ip: u64,
    pub mnemonic: String,
    pub formatted: String,
    pub bytes: Vec<u8>,
}

/// Disassembles up to `max_instructions` x86-64 instructions starting at `ip`.
pub fn disassemble_x64(bytes: &[u8], ip: u64, max_instructions: usize) -> Vec<DisassembledInstruction> {
    let mut decoder = Decoder::with_ip(64, bytes, ip, DecoderOptions::NONE);
    let mut formatter = IntelFormatter::new();
    let mut instructions = Vec::new();
    let mut instruction = Instruction::default();
    let mut output = String::new();

    while decoder.can_decode() && instructions.len() < max_instructions {
        decoder.decode_out(&mut instruction);
        output.clear();
        formatter.format(&instruction, &mut output);

        let start = (instruction.ip() - ip) as usize;
        let len = instruction.len();
        let instr_bytes = if start + len <= bytes.len() {
            bytes[start..start + len].to_vec()
        } else {
            Vec::new()
        };

        instructions.push(DisassembledInstruction {
            ip: instruction.ip(),
            mnemonic: format!("{:?}", instruction.mnemonic()),
            formatted: output.clone(),
            bytes: instr_bytes,
        });
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_nop_and_ret() {
        // 0x90 (NOP), 0xC3 (RET)
        let code = [0x90, 0xC3];
        let instrs = disassemble_x64(&code, 0x1000, 10);
        assert_eq!(instrs.len(), 2);
        assert_eq!(instrs[0].formatted, "nop");
        assert_eq!(instrs[1].formatted, "ret");
    }
}
