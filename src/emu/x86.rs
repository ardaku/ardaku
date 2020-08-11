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
    /// General purpose 0 (should use the most)
    EAX = 0b000,
    /// General purpose 1
    ECX = 0b001,
    /// General purpose 2
    EDX = 0b010,
    /// General purpose 3
    EBX = 0b011,
    /// Not A general Purpose Register
    ESP = 0b100,
    /// General purpose 4
    EBP = 0b101,
    /// General purpose 5
    ESI = 0b110,
    /// General purpose 6
    EDI = 0b111,
}

/// An assembly instruction (dst=0-7, src=0-7)
#[allow(clippy::enum_variant_names)]
pub(crate) enum I {
    /// R[.0] : R[.1]
    MOV(u8, u8),
    /// R[.0] +: R[.1]
    ADD(u8, u8),
    /// R[.0] \: R[.1]
    XOR(u8, u8),
    /// R[.0] +: .1
    ADDI(u8, i32),
}

impl I {
    /// Write self to machine code buffer.
    pub fn encode(&self, mc: &mut Vec<u8>) {
        match *self {
            // Move Instruction
            MOV { dst, src } => {
                mc.push(0b10001011); // MOV(32) Opcode
                mc.push((0b11 << 6) | (src << 3) | dst);
            }
            // Arithmetic Instructions
            ADD { dst, src } => {
                mc.push(0b00000001); // ADD(32) Opcode
                mc.push((0b11 << 6) | (src << 3) | dst);
            }
            XOR { dst, src } => {
                mc.push(0b00110001); // XOR(32) Opcode
                mc.push((0b11 << 6) | (src << 3) | dst);
            }
            // Immediate Instructions
            ADDI { dst, imm } => {
                if let Ok(imm) = imm.try_into() {
                    let imm: i8 = imm;
                    // Can use reduced instruction size
                    mc.push(0b10000011); // IMMEDIATE(8) Opcode
                    mc.push((0b00000 << 3) | dst); // "ADD"
                    mc.push(imm as u8);
                } else if dst == 0 { // EAX
                    // Can use accumulator shortenned instruction
                    mc.push(0b00000101);
                    for byte in imm.to_ne_bytes().iter().cloned() {
                        mc.push(byte);
                    }
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
        LUI { d, imm } => todo!(),
        AUIPC { d, imm } => todo!(),
        JAL { d, imm } => todo!(),
        JALR { d, s, imm } => todo!(),
        BEQ { s1, s2, imm } => todo!(),
        BNE { s1, s2, imm } => todo!(),
        BLT { s1, s2, imm } => todo!(),
        BGE { s1, s2, imm } => todo!(),
        BLTU { s1, s2, imm } => todo!(),
        BGEU { s1, s2, imm } => todo!(),
        LB { d, s, imm } => todo!(),
        LH { d, s, imm } => todo!(),
        LW { d, s, imm } => todo!(),
        LBU { d, s, imm } => todo!(),
        LHU { d, s, imm } => todo!(),
        ADDI { d, s, imm } => {
            match (d, s, imm) {
                (ZERO, _, _) => { /* nop */ },
                (d, ZERO, 0) => I::XOR(d, d).encode(mc),
                (d, ZERO, s) => I::MOV(d, s).encode(mc),
                (d, s, imm) if d == s => I::ADDI(d, imm).encode(mc),
                (d, s, imm) => {
                    I::MOV(d, s).encode(mc);
                    I::ADDI(d, imm).encode(mc);
                }
            }
        },
        SLTI { d, s, imm } => todo!(),
        SLTUI { d, s, imm } => todo!(),
        XORI { d, s, imm } => todo!(),
        ORI { d, s, imm } => todo!(),
        ANDI { d, s, imm } => todo!(),
        SLLI { d, s, imm } => todo!(),
        SRLI { d, s, imm } => todo!(),
        SRAI { d, s, imm } => todo!(),
        SB { s1, s2, imm } => todo!(),
        SH { s1, s2, imm } => todo!(),
        SW { s1, s2, imm } => todo!(),
        ADD { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _, _) => { /* nop */ },
                (d, ZERO, ZERO) => I::XOR(d, d).encode(mc),
                (d, ZERO, s2) => I::MOV(d, s2).encode(mc),
                (d, s1, ZERO) => I::MOV(d, s1).encode(mc),
                (d, s1, s2) if d == s2 => I::ADD(d, s1).encode(mc),
                (d, s1, s2) if d == s1 => I::ADD(d, s2).encode(mc),
                (d, s1, s2) => {
                    I::MOV(d, s1).encode(mc);
                    I::ADD(d, s2).encode(mc);
                }
            }
        },
        SUB { d, s1, s2 } => todo!(),
        SLL { d, s1, s2 } => todo!(),
        SLT { d, s1, s2 } => todo!(),
        SLTU { d, s1, s2 } => todo!(),
        XOR { d, s1, s2 } => todo!(),
        SRL { d, s1, s2 } => todo!(),
        SRA { d, s1, s2 } => todo!(),
        OR { d, s1, s2 } => todo!(),
        AND { d, s1, s2 } => todo!(),
        ECALL {} => todo!(),
        EBREAK {} => todo!(),
        FENCE { imm } => todo!(),
    }
}
