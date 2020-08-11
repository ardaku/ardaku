//! Conversion from RISC-V into X86 For Emulation
//!
//! While RISC-V Has 31 General Purpose Registers, X86 only has 7 - so before
//! calling `I::from(riscv)`, a register reduction alogorithm must be executed.
//! Note also that X86 instructions don't have as many parameters:
//! # Add Translation Example
//! ```asm
//! # riscv
//! add $x1, $x2, $x3
//! # x86
//! mov $x1, $x2
//! add $x1, $x3
//!
//! # riscv
//! add $x1, $x1, $x2
//! # x86
//! add $x1, $x2
//!
//! # riscv
//! add $x2, $x1, $x2
//! # x86
//! add $x2, $x1
//!
//! # riscv
//! add $zero, $x1, $x2
//! # x86
//! nop
//!
//! # riscv
//! add $x1, $x2, $zero
//! # x86
//! mov $x1, $x2
//! 
//! # riscv
//! add $x1, $zero, $x2
//! # x86
//! mov $x1, $x2
//!
//! # riscv
//! add $x1, $zero, $zero
//! # x86
//! xor $x1, $x1
//! ```

use std::convert::TryInto;

use Reg::*;
use I::*;

/// An X86 Register
#[repr(u8)]
#[derive(Copy, Clone)]
pub(crate) enum Reg {
    EAX = 0b000,
    ECX = 0b001,
    EDX = 0b010,
    EBX = 0b011,
    /// Not A general Purpose Register
    ESP = 0b100,
    EBP = 0b101,
    ESI = 0b110,
    EDI = 0b111,
}

/// An assembly instruction (dst=0-7, src=0-7)
#[allow(clippy::enum_variant_names)]
pub(crate) enum I {
    /// R[dst] +: R[src]
    ADD { dst: u8, src: u8 },
    /// R[dst] +: imm
    ADDI { dst: u8, imm: i32 },
}

impl I {
    /// Write self to machine code buffer.
    pub fn encode(&self, mc: &mut Vec<u8>) {
        match *self {
            ADD { dst, src } => {
                mc.push(0b00000001); // ADD Opcode
                mc.push((0b11 << 6) | (src << 3) | dst);
            }
            ADDI { dst, imm } => {
                if let Ok(imm) = imm.try_into() {
                    let imm: i8 = imm;
                    // Can use reduced instruction size
                    mc.push(0b10000011); // IMMEDIATE(8) Opcode
                    mc.push((0b00000 << 3) | dst); // "ADD"
                    mc.push(imm as u8);
                } else {
                    // Full 32-bit immediate instruction
                    mc.push(0b10000001); // IMMEDIATE(32) Opcode
                    mc.push((0b00000 << 3) | dst); // "ADD"
                    for byte in imm.to_ne_bytes().iter().cloned() {
                        mc.push(byte);
                    }
                }
            }
        }
    }
}

/// Translate a RISC-V Instruction into X86 Machine Code.
pub fn translate(with: riscv::I, mc: &mut Vec<u8>) {
    use riscv::I::*;
    match with {
        LUI { d, imm } => I::,
        AUIPC { d, imm } => I::,
        JAL { d, imm } => I::,
        JALR { d, s, imm } => I::,
        BEQ { s1, s2, imm } => I::,
        BNE { s1, s2, imm } => I::,
        BLT { s1, s2, imm } => I::,
        BGE { s1, s2, imm } => I::,
        BLTU { s1, s2, imm } => I::,
        BGEU { s1, s2, imm } => I::,
        LB { d, s, imm } => I::,
        LH { d, s, imm } => I::,
        LW { d, s, imm } => I::,
        LBU { d, s, imm } => I::,
        LHU { d, s, imm } => I::,
        ADDI { d, s, imm } => I::,
        SLTI { d, s, imm } => I::,
        SLTUI { d, s, imm } => I::,
        XORI { d, s, imm } => I::,
        ORI { d, s, imm } => I::,
        ANDI { d, s, imm } => I::,
        SLLI { d, s, imm } => I::,
        SRLI { d, s, imm } => I::,
        SRAI { d, s, imm } => I::,
        SB { s1, s2, imm } => I::,
        SH { s1, s2, imm } => I::,
        SW { s1, s2, imm } => I::,
        ADD { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _s1, _s2) => { /* nop */ },
                (_d, ZERO, ZERO) => I::XOR(d, d).encode(mc),
                (_d, ZERO, _s2) => I::MOV(d, s2).encode(mc),
                (_d, _s1, ZERO) => I::MOV(d, s1).encode(mc),
                (_d, _s1, _s2) if d == s2 => I::ADD(d, s1).encode(mc),
                (_d, _s1, _s2) if d == s1 => I::ADD(d, s2).encode(mc),
                (_d, _s1, _s2) => {
                    I::MOV(d, s1).encode(mc);
                    I::ADD(d, s2).encode(mc);
                }
            }
        },
        SUB { d, s1, s2 } => I::,
        SLL { d, s1, s2 } => I::,
        SLT { d, s1, s2 } => I::,
        SLTU { d, s1, s2 } => I::,
        XOR { d, s1, s2 } => I::,
        SRL { d, s1, s2 } => I::,
        SRA { d, s1, s2 } => I::,
        OR { d, s1, s2 } => I::,
        AND { d, s1, s2 } => I::,
        ECALL {} => I::,
        EBREAK {} => I::,
        FENCE { imm } => I::,
    }
}
