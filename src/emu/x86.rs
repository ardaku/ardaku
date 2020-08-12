//! Conversion from RISC-V into X86 For Emulation
//!
//! This should not do any non-translation specific optimizations, the input
//! RISC-V should already be optimized so that this code can do mimimal work to
//! generate ideally optimized code.
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
//!
//! # Register Translation
//! ```asm
//! x0  ZERO # Use MOV instructions instead
//! x1  RA   # Use CALL and RET (stack) instructions instead
//! x2  SP   # register: ESP
//! x3  GP   # ???: register ECX(3)
//! x4  TP   # ???: register EDX(3)
//! x5  T0   # register: EAX(1)
//! x6  T1   # register: ECX(1)
//! x7  T2   # register: EDX(1)
//! x8  S0   # register: EBX(0)
//! x9  S1   # register: EBP(0)
//! x10 A0   # parameter PUSH/POP stack / return register: EAX(0)
//! x11 A1   # parameter PUSH/POP stack / +return register: ECX(0)
//! x12 A2   # parameter PUSH/POP stack / +return register: EDX(0)
//! x13 A3   # parameter/+return PUSH/POP stack
//! x14 A4   # parameter/+return PUSH/POP stack
//! x15 A5   # parameter/+return PUSH/POP stack
//! x16 A6   # parameter/+return PUSH/POP stack
//! x17 A7   # parameter/+return PUSH/POP stack
//! x18 S2   # register: ESI(0)
//! x19 S3   # register: EDI(0)
//! x20 S4   # register: EBX(1)
//! x21 S5   # register: EBP(1)
//! x22 S6   # register: ESI(1)
//! x23 S7   # register: EDI(1)
//! x24 S8   # register: EBX(2)
//! x25 S9   # register: EBP(2)
//! x26 S10  # register: ESI(2)
//! x27 S11  # register: EDI(2)
//! x28 T3   # register: EAX(2)
//! x29 T4   # register: ECX(2)
//! x30 T5   # register: EDX(2)
//! x31 T6   # register: EAX(3)
//! ```

use std::convert::TryInto;
use std::pin::Pin;

use asm_riscv::Reg::*;
use Reg::*;
use I::*;

/// An X86 Register
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum Reg {
    /// Data Accumulator - Arithmetic/Logic / General purpose 0 (should use the most)
    /// Caller Saved (Push onto stack before function call)
    /// Return value.
    EAX = 0b000,
    /// Data Counter - Loop counter / General purpose 1
    /// Caller Saved (Push onto stack before function call)
    ECX = 0b001,
    /// Data - General purpose 2
    /// Caller Saved (Push onto stack before function call)
    EDX = 0b010,
    /// Data - General purpose 3
    /// Function Saved (Push onto stack inside function, restore before return)
    EBX = 0b011,
    /// Address; SPECIAL: Stack Pointer (Not A general Purpose Register)
    /// Function Saved (Push onto stack inside function, restore before return)
    ESP = 0b100,
    /// Address; Stack Segment / Base Pointer - General purpose 4
    /// Function Saved (Push onto stack inside function, restore before return)
    EBP = 0b101,
    /// Address; Source Index (incrementer) - General purpose 5
    /// Function Saved (Push onto stack inside function, restore before return)
    ESI = 0b110,
    /// Address; Destination Index (incrementer) - General purpose 6
    /// Function Saved (Push onto stack inside function, restore before return)
    EDI = 0b111,
}

impl From<asm_riscv::Reg> for Reg {
    // Convert Reduced RISC-V Registers Into X86 Registers
    fn from(with: asm_riscv::Reg) -> Self {
        match with {
            ZERO => unreachable!(),
            RA => todo!(),
            SP => ESP,
            GP => todo!(),
            TP => todo!(),
            T0 => unreachable!(),
            T1 => unreachable!(),
            T2 => unreachable!(),
            S0 => EBX,
            S1 => EBP,
            A0 => EAX,
            A1 => ECX,
            A2 => EDX,
            A3 => unreachable!(),
            A4 => unreachable!(),
            A5 => unreachable!(),
            A6 => unreachable!(),
            A7 => unreachable!(),
            S2 => ESI,
            S3 => EDI,
            S4 => unreachable!(),
            S5 => unreachable!(),
            S6 => unreachable!(),
            S7 => unreachable!(),
            S8 => unreachable!(),
            S9 => unreachable!(),
            S10 => unreachable!(),
            S11 => unreachable!(),
            T3 => unreachable!(),
            T4 => unreachable!(),
            T5 => unreachable!(),
            T6 => unreachable!(),
        }
    }
}

/// An assembly instruction (dst=0-7, src=0-7)
#[allow(clippy::enum_variant_names)]
pub(crate) enum I {
    /// R[.0]: R[.1]
    MOV(Reg, Reg),
    /// R[.0] +: R[.1]
    ADD(Reg, Reg),
    /// R[.0] -: R[.1]
    SUB(Reg, Reg),
    /// R[.0] ^^: R[.1]
    XOR(Reg, Reg),
    /// R[.0]: -.0
    NEG(Reg),
    /// R[.0]: ~.0
    NOT(Reg),
    /// R[.0]: .1
    MOVI(Reg, i32),
    /// R[.0] +: .1
    ADDI(Reg, i32),
    /// R[.0] ^^: .1
    XORI(Reg, i32),
}

impl I {
    /// Write self to machine code buffer.
    pub fn encode(&self, mc: &mut Vec<u8>) {
        match *self {
            // Move Instruction
            MOV(dst, src) => {
                mc.push(0b10001011); // MOV(32) Opcode
                mc.push((0b11 << 6) | ((src as u8) << 3) | dst as u8);
            }
            // Arithmetic Instructions
            ADD(dst, src) => {
                mc.push(0b00000011); // ADD Registers (32) Opcode
                mc.push((0b11 << 6) | ((src as u8) << 3) | dst as u8);
            }
            SUB(dst, src) => {
                mc.push(0b00101011); // SUB Registers (32) Opcode
                mc.push((0b11 << 6) | ((src as u8) << 3) | dst as u8);
            }
            XOR(dst, src) => {
                mc.push(0b00110001); // XOR Registers (32) Opcode
                mc.push((0b11 << 6) | ((src as u8) << 3) | dst as u8);
            }
            // Arithmetic 1-Operand Instructions
            NEG(dst) => {
                mc.push(0b11110111); // Unary Register (32) Opcode
                mc.push((0b11 << 6) | dst as u8); // NEG
            }
            NOT(dst) => {
                mc.push(0b11110111); // Unary Register (32) Opcode
                mc.push((0b10 << 6) | dst as u8); // NOT
            }
            // immediate Instructions
            MOVI(dst, im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b11000110); // MOVE IMMEDIATE(8) Opcode
                    mc.push((0b00000 << 3) | dst as u8); // "XOR"
                    mc.push(im as u8);
                } else {
                    // Full 32-bit immediate instruction
                    mc.push(0b11000111); // MOVE IMMEDIATE(32) Opcode
                    mc.push((0b00000 << 3) | dst as u8); // "XOR"
                    for byte in im.to_ne_bytes().iter().cloned() {
                        mc.push(byte);
                    }
                }
            }
            ADDI(dst, im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b10000011); // IMMEDIATE(8) Opcode
                    mc.push((0b00000 << 3) | dst as u8); // "ADD"
                    mc.push(im as u8);
                } else if dst == EAX {
                    // Can use accumulator shortenned instruction
                    mc.push(0b00000101);
                    for byte in im.to_ne_bytes().iter().cloned() {
                        mc.push(byte);
                    }
                } else {
                    // Full 32-bit immediate instruction
                    mc.push(0b10000001); // IMMEDIATE(32) Opcode
                    mc.push((0b00000 << 3) | dst as u8); // "ADD"
                    for byte in im.to_ne_bytes().iter().cloned() {
                        mc.push(byte);
                    }
                }
            }
            XORI(dst, im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b10000011); // IMMEDIATE(8) Opcode
                    mc.push((0b00110 << 3) | dst as u8); // "XOR"
                    mc.push(im as u8);
                } else {
                    // Full 32-bit immediate instruction
                    mc.push(0b10000001); // IMMEDIATE(32) Opcode
                    mc.push((0b00110 << 3) | dst as u8); // "XOR"
                    for byte in im.to_ne_bytes().iter().cloned() {
                        mc.push(byte);
                    }
                }
            }
        }
    }
}

/// Translate a RISC-V Instruction into X86 Machine Code.
pub fn translate(with: asm_riscv::I, mc: &mut Vec<u8>, sysargs: &mut Pin<Box<[u32; 3]>>) {
    use asm_riscv::I::*;
    match with {
        LUI { d, im } => todo!(),
        AUIPC { d, im } => todo!(),
        JAL { d, im } => todo!(),
        JALR { d, s, im } => todo!(),
        BEQ { s1, s2, im } => todo!(),
        BNE { s1, s2, im } => todo!(),
        BLT { s1, s2, im } => todo!(),
        BGE { s1, s2, im } => todo!(),
        BLTU { s1, s2, im } => todo!(),
        BGEU { s1, s2, im } => todo!(),
        LB { d, s, im } => todo!(),
        LH { d, s, im } => todo!(),
        LW { d, s, im } => todo!(),
        LBU { d, s, im } => todo!(),
        LHU { d, s, im } => todo!(),
        ADDI { d, s, im } => {
            match (d, s, im) {
                (ZERO, _, _) => { /* nop */ },
                (d, ZERO, 0) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s) => I::MOVI(d.into(), s.into()).encode(mc),
                (d, s, im) if d == s => I::ADDI(d.into(), im.into()).encode(mc),
                (d, s, im) => {
                    I::MOV(d.into(), s.into()).encode(mc);
                    I::ADDI(d.into(), im.into()).encode(mc);
                }
            }
        },
        SLTI { d, s, im } => todo!(),
        SLTUI { d, s, im } => todo!(),
        XORI { d, s, im } => {
            match (d, s, im) {
                (ZERO, _, _) => { /* nop */ },
                (d, ZERO, 0) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s) => I::MOVI(d.into(), s.into()).encode(mc),
                (d, s, im) if d == s => I::XORI(d.into(), im.into()).encode(mc),
                (d, s, im) => {
                    I::MOV(d.into(), s.into()).encode(mc);
                    I::XORI(d.into(), im.into()).encode(mc);
                }
            }
        },
        ORI { d, s, im } => todo!(),
        ANDI { d, s, im } => todo!(),
        SLLI { d, s, im } => todo!(),
        SRLI { d, s, im } => todo!(),
        SRAI { d, s, im } => todo!(),
        SB { s1, s2, im } => todo!(),
        SH { s1, s2, im } => todo!(),
        SW { s1, s2, im } => todo!(),
        ADD { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _, _) => { /* nop */ },
                (d, ZERO, ZERO) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s2) => I::MOV(d.into(), s2.into()).encode(mc),
                (d, s1, ZERO) => I::MOV(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s2 => I::ADD(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s1 => I::ADD(d.into(), s2.into()).encode(mc),
                (d, s1, s2) => {
                    I::MOV(d.into(), s1.into()).encode(mc);
                    I::ADD(d.into(), s2.into()).encode(mc);
                }
            }
        },
        SUB { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _, _) => { /* nop */ },
                (d, ZERO, ZERO) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s2) if d == s2 => I::NEG(d.into()).encode(mc),
                (d, ZERO, s2) => {
                    I::MOV(d.into(), s2.into()).encode(mc);
                    I::NEG(d.into()).encode(mc);
                }
                (d, s1, ZERO) => I::MOV(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s2 => I::SUB(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s1 => I::SUB(d.into(), s2.into()).encode(mc),
                (d, s1, s2) => {
                    I::MOV(d.into(), s1.into()).encode(mc);
                    I::SUB(d.into(), s2.into()).encode(mc);
                }
            }
        },
        SLL { d, s1, s2 } => todo!(),
        SLT { d, s1, s2 } => todo!(),
        SLTU { d, s1, s2 } => todo!(),
        SRL { d, s1, s2 } => todo!(),
        SRA { d, s1, s2 } => todo!(),
        XOR { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _, _) => { /* nop */ },
                (d, ZERO, ZERO) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s2) => I::MOV(d.into(), s2.into()).encode(mc),
                (d, s1, ZERO) => I::MOV(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s2 => I::XOR(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s1 => I::XOR(d.into(), s2.into()).encode(mc),
                (d, s1, s2) => {
                    I::MOV(d.into(), s1.into()).encode(mc);
                    I::XOR(d.into(), s2.into()).encode(mc);
                }
            }
        },
        OR { d, s1, s2 } => todo!(),
        AND { d, s1, s2 } => todo!(),
        ECALL {} => {
            // Copy syscall argument registers into kernel memory.
            let a0 = sysargs[0..].as_mut_ptr() as usize; // EAX
            let a1 = sysargs[1..].as_mut_ptr() as usize; // ECX
            let a2 = sysargs[2..].as_mut_ptr() as usize; // EDX

            eprintln!("a0 {:X}", a0);
            eprintln!("a1 {:X}", a1);
            eprintln!("a2 {:X}", a2);

            todo!()
        },
        EBREAK {} => {
            todo!() // Same as ECALL (but with return value 1 instead of 0)
        },
        FENCE { im } => todo!(),
    }
}
