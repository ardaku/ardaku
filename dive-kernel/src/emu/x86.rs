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
pub enum Reg {
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
pub enum I {
    /// R[.0]: R[.1]
    MOV(Reg, Reg),
    /// R[.0] +: R[.1]
    ADD(Reg, Reg),
    /// R[.0] -: R[.1]
    SUB(Reg, Reg),
    /// R[.0] ^^: R[.1]
    XOR(Reg, Reg),
    /// R[.0] &: R[.1]
    AND(Reg, Reg),
    /// R[.0] |: R[.1]
    OR(Reg, Reg),
    /// EFLAGS <= R[.0] <, =, > R[.1]
    CMP(Reg, Reg),
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
    /// R[.0] &: .1
    ANDI(Reg, i32),
    /// R[.1] &: .2
    ORI(Reg, i32),
    /// EFLAGS <= R[.0] <(), =(0), >() R[.1]
    CMPI(Reg, i32),
    /// JUMP IF EFLAGS = 0
    JE(i16),
    /// JUMP IF EFLAGS != 0
    JNE(i16),
    /// JUMP IF EFLAGS = <
    JL(i16),
    /// JUMP IF EFLAGS = >
    JG(i16),
    /// JUMP IF EFLAGS = ≤
    JLE(i16),
    /// JUMP IF EFLAGS = ≥
    JGE(i16),
    /// JUMP IF EFLAGS = < (UNSIGNED)
    JB(i16),
    /// JUMP IF EFLAGS = > (UNSIGNED)
    JA(i16),
    /// JUMP IF EFLAGS = ≤ (UNSIGNED)
    JBE(i16),
    /// JUMP IF EFLAGS = ≥ (UNSIGNED)
    JAE(i16),
    /// JUMP IMMEDIATE UNCONDITIONAL
    JMP(i16),
    /// JUMP MEMORY UNCONDITIONAL: PC: M[R[.0]]
    JMPM(Reg),
    /// Stack.Push(PC); PC +: .0;
    CALL(i16),
    /// Stack.Push(R[.0])
    PUSH(Reg),
    /// R[.0]: Stack.Pop();
    POP(Reg),
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
                mc.push(0b00110011); // XOR Registers (32) Opcode
                mc.push((0b11 << 6) | ((src as u8) << 3) | dst as u8);
            }
            AND(dst, src) => {
                mc.push(0b00100011); // AND Registers (32) Opcode
                mc.push((0b11 << 6) | ((src as u8) << 3) | dst as u8);
            },
            OR(dst, src) => {
                mc.push(0b00001011); // OR Registers (32) Opcode
                mc.push((0b11 << 6) | ((src as u8) << 3) | dst as u8);
            },
            CMP(dst, src) => {
                mc.push(0b00111011); // CMP Registers (32) Opcode
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
            ANDI(dst, im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b10000011); // IMMEDIATE(8) Opcode
                    mc.push((0b00100 << 3) | dst as u8); // "AND"
                    mc.push(im as u8);
                } else {
                    // Full 32-bit immediate instruction
                    mc.push(0b10000001); // IMMEDIATE(32) Opcode
                    mc.push((0b00100 << 3) | dst as u8); // "AND"
                    for byte in im.to_ne_bytes().iter().cloned() {
                        mc.push(byte);
                    }
                }
            }
            ORI(dst, im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b10000011); // IMMEDIATE(8) Opcode
                    mc.push((0b00001 << 3) | dst as u8); // "OR"
                    mc.push(im as u8);
                } else {
                    // Full 32-bit immediate instruction
                    mc.push(0b10000001); // IMMEDIATE(32) Opcode
                    mc.push((0b00001 << 3) | dst as u8); // "OR"
                    for byte in im.to_ne_bytes().iter().cloned() {
                        mc.push(byte);
                    }
                }
            }
            CMPI(dst, im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b10000011); // IMMEDIATE(8) Opcode
                    mc.push((0b00111 << 3) | dst as u8); // "CMP"
                    mc.push(im as u8);
                } else {
                    // Full 32-bit immediate instruction
                    mc.push(0b10000001); // IMMEDIATE(32) Opcode
                    mc.push((0b00111 << 3) | dst as u8); // "CMP"
                    for byte in im.to_ne_bytes().iter().cloned() {
                        mc.push(byte);
                    }
                }
            }
            JE(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01110100); // IF ZERO JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10000100); // IF EQUAL (0)
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JNE(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01110101); // IF NOT ZERO JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10000101); // IF NOT EQUAL (0)
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JL(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01111100); // IF LESS JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10001100); // IF LESS
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JG(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01111111); // IF GREATER JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10001111); // IF GREATER
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JLE(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01111110); // IF LESS OR EQUAL JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10001110); // IF GREATER OR EQUAL
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JGE(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01111101); // IF GREATER OR EQUAL JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10001101); // IF GREATER OR EQUAL
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JB(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01110010); // IF BELOW JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10000010); // IF BELOW
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JA(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01110111); // IF ABOVE JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10000111); // IF ABOVE
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JBE(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01110110); // IF BELOW OR EQUAL JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10000110); // IF BELOW OR EQUAL
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JAE(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b01110011); // IF ABOVE OR EQUAL JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b00001111); // JUMP NEAR
                    mc.push(0b10000011); // IF ABOVE OR EQUAL
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JMP(im) => {
                if let Ok(im) = im.try_into() {
                    let im: i8 = im;
                    // Can use reduced instruction size
                    mc.push(0b11101011); // UNCONDITIONAL JUMP SHORT
                    mc.push(im as u8);
                } else {
                    // Full 16-bit near immediate instruction
                    let im = im.to_ne_bytes();
                    mc.push(0b11101001); // JUMP NEAR UNCONDITIONAL
                    mc.push(im[0]);
                    mc.push(im[1]);
                }
            }
            JMPM(src) => {
                mc.push(0b11111111); // Unary Register (32) Opcode
                mc.push((0b00101 << 3) | src as u8); // NOT
            }
            CALL(im) => {
                let im = im.to_ne_bytes();
                mc.push(0b11101000); // Call near relative Opcode
                mc.push(im[0]);
                mc.push(im[1]);
            }
            PUSH(dst) => {
                mc.push(0b01010000 + dst as u8); // Push register opcodes
            }
            POP(dst) => {
                mc.push(0b01011000 + dst as u8); // Pop register opcodes
            }
        }
    }
}

/// Translate a RISC-V Instruction into X86 Machine Code.
pub fn translate(
    with: asm_riscv::I,
    mc: &mut Vec<u8>,
    sysargs: &mut Pin<Box<[u32; 3]>>,
) {
    use asm_riscv::I::*;
    match with {
        LUI { d, im } => {
            match (d, im) {
                (ZERO, _) => { /* nop */ },
                (d, im) => I::MOVI(d.into(), im << 12).encode(mc),
            }
        },
        AUIPC { d, im } => {
            match (d, im) {
                (ZERO, _) => { /* nop */ }
                (d, im) => {
                    // Write current PC into d
                    I::CALL(0).encode(mc);
                    I::POP(d.into()).encode(mc);
                    // Add upper immediate to d
                    I::ADDI(d.into(), im << 12).encode(mc);
                }
            }
        },
        JAL { d, im } => {
            match (d, im) {
                (ZERO, im) => I::JMPF(im).encode(mc),
                (RA, im) => I::CALL(im).encode(mc),
                (d, im) => {
                    I::CALL(im).encode(mc);
                    I::POP(d.into()).encode(mc);
                    I::PUSH(d.into()).encode(mc);
                }
            }
        },
        JALR { d, s, im } => {
            match (d, s, im) {
                (ZERO, ZERO, im) => I::JMPA(im).encode(mc),
                (d, ZERO, im) => {
                    I::JMPA(im).encode(mc);
                    I::POP(d.into()).encode(mc);
                }
                (ZERO, s, im) => {} // I::JMP(s, im).encode(mc),
                (RA, im) => I::CALL(im).encode(mc),
                (d, im) => {
                    I::CALL(im).encode(mc);
                    I::POP(d.into()).encode(mc);
                    I::PUSH(d.into()).encode(mc);
                }
            }
        }
        BEQ { s1, s2, im } => {
            match (s1, s2, im) {
                (ZERO, ZERO, im) => I::JMP(im).encode(mc),
                (ZERO, s2, im) => {
                    I::CMPI(s2.into(), 0).encode(mc);
                    I::JE(im).encode(mc);
                },
                (s1, ZERO, im) => {
                    I::CMPI(s1.into(), 0).encode(mc);
                    I::JE(im).encode(mc);
                },
                (s1, s2, im) if s1 == s2 => {
                    I::JMP(im).encode(mc);
                },
                (s1, s2, im) => {
                    I::CMP(s1.into(), s2.into()).encode(mc);
                    I::JE(im).encode(mc);
                },
            }
        },
        BNE { s1, s2, im } => {
            match (s1, s2, im) {
                (s1, s2, im) if s1 == s2 => { /* nop */ },
                (ZERO, s2, im) => {
                    I::CMPI(s2.into(), 0).encode(mc);
                    I::JNE(im).encode(mc);
                },
                (s1, ZERO, im) => {
                    I::CMPI(s1.into(), 0).encode(mc);
                    I::JNE(im).encode(mc);
                },
                (s1, s2, im) => {
                    I::CMP(s1.into(), s2.into()).encode(mc);
                    I::JNE(im).encode(mc);
                },
            }
        },
        BLT { s1, s2, im } => {
            match (s1, s2, im) {
                (s1, s2, im) if s1 == s2 => { /* nop */ },
                (ZERO, s2, im) => {
                    I::CMPI(s2.into(), 0).encode(mc);
                    I::JL(im).encode(mc);
                },
                (s1, ZERO, im) => {
                    I::CMPI(s1.into(), 0).encode(mc);
                    I::JG(im).encode(mc);
                },
                (s1, s2, im) => {
                    I::CMP(s1.into(), s2.into()).encode(mc);
                    I::JL(im).encode(mc);
                },
            }
        },
        BGE { s1, s2, im } => {
            match (s1, s2, im) {
                (ZERO, ZERO, im) => I::JMP(im).encode(mc),
                (ZERO, s2, im) => {
                    I::CMPI(s2.into(), 0).encode(mc);
                    I::JGE(im).encode(mc);
                },
                (s1, ZERO, im) => {
                    I::CMPI(s1.into(), 0).encode(mc);
                    I::JLE(im).encode(mc);
                },
                (s1, s2, im) if s1 == s2 => {
                    I::JMP(im).encode(mc);
                },
                (s1, s2, im) => {
                    I::CMP(s1.into(), s2.into()).encode(mc);
                    I::JGE(im).encode(mc);
                },
            }
        },
        BLTU { s1, s2, im } => {
            match (s1, s2, im) {
                (s1, s2, im) if s1 == s2 => { /* nop */ },
                (ZERO, s2, im) => {
                    I::CMPI(s2.into(), 0).encode(mc);
                    I::JB(im).encode(mc);
                },
                (s1, ZERO, im) => {
                    I::CMPI(s1.into(), 0).encode(mc);
                    I::JA(im).encode(mc);
                },
                (s1, s2, im) => {
                    I::CMP(s1.into(), s2.into()).encode(mc);
                    I::JB(im).encode(mc);
                },
            }
        },
        BGEU { s1, s2, im } => {
            match (s1, s2, im) {
                (ZERO, ZERO, im) => I::JMP(im).encode(mc),
                (ZERO, s2, im) => {
                    I::CMPI(s2.into(), 0).encode(mc);
                    I::JAE(im).encode(mc);
                },
                (s1, ZERO, im) => {
                    I::CMPI(s1.into(), 0).encode(mc);
                    I::JBE(im).encode(mc);
                },
                (s1, s2, im) if s1 == s2 => {
                    I::JMP(im).encode(mc);
                },
                (s1, s2, im) => {
                    I::CMP(s1.into(), s2.into()).encode(mc);
                    I::JAE(im).encode(mc);
                },
            }
        },
        LB { d, s, im } => todo!(),
        LH { d, s, im } => todo!(),
        LW { d, s, im } => todo!(),
        LBU { d, s, im } => todo!(),
        LHU { d, s, im } => todo!(),
        ADDI { d, s, im } => {
            match (d, s, im) {
                (ZERO, _, _) => { /* nop */ }
                (d, ZERO, 0) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s) => I::MOVI(d.into(), s.into()).encode(mc),
                (d, s, im) if d == s => I::ADDI(d.into(), im.into()).encode(mc),
                (d, s, im) => {
                    I::MOV(d.into(), s.into()).encode(mc);
                    I::ADDI(d.into(), im.into()).encode(mc);
                }
            }
        }
        SLTI { d, s, im } => todo!(),
        SLTUI { d, s, im } => todo!(),
        XORI { d, s, im } => {
            match (d, s, im) {
                (ZERO, _, _) => { /* nop */ }
                (d, ZERO, 0) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s) => I::MOVI(d.into(), s.into()).encode(mc),
                (d, s, im) if d == s => I::XORI(d.into(), im.into()).encode(mc),
                (d, s, im) => {
                    I::MOV(d.into(), s.into()).encode(mc);
                    I::XORI(d.into(), im.into()).encode(mc);
                }
            }
        }
        ORI { d, s, im } => {
            match (d, s, im) {
                (ZERO, _, _) => { /* nop */ }
                (d, ZERO, 0) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s) => I::MOVI(d.into(), s.into()).encode(mc),
                (d, s, im) if d == s => I::ORI(d.into(), im.into()).encode(mc),
                (d, s, im) => {
                    I::MOV(d.into(), s.into()).encode(mc);
                    I::ORI(d.into(), im.into()).encode(mc);
                }
            }
        },
        ANDI { d, s, im } => {
            match (d, s, im) {
                (ZERO, _, _) => { /* nop */ }
                (d, ZERO, _) => I::XOR(d.into(), d.into()).encode(mc),
                (d, _, 0) => I::XOR(d.into(), d.into()).encode(mc),
                (d, s, im) if d == s => I::ANDI(d.into(), im.into()).encode(mc),
                (d, s, im) => {
                    I::MOV(d.into(), s.into()).encode(mc);
                    I::ANDI(d.into(), im.into()).encode(mc);
                }
            }
        },
        SLLI { d, s, im } => todo!(),
        SRLI { d, s, im } => todo!(),
        SRAI { d, s, im } => todo!(),
        SB { s1, s2, im } => todo!(),
        SH { s1, s2, im } => todo!(),
        SW { s1, s2, im } => todo!(),
        ADD { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _, _) => { /* nop */ }
                (d, ZERO, ZERO) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s2) => I::MOV(d.into(), s2.into()).encode(mc),
                (d, s1, ZERO) => I::MOV(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s2 => {
                    I::ADD(d.into(), s1.into()).encode(mc)
                }
                (d, s1, s2) if d == s1 => {
                    I::ADD(d.into(), s2.into()).encode(mc)
                }
                (d, s1, s2) => {
                    I::MOV(d.into(), s1.into()).encode(mc);
                    I::ADD(d.into(), s2.into()).encode(mc);
                }
            }
        }
        SUB { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _, _) => { /* nop */ }
                (d, ZERO, ZERO) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s2) if d == s2 => I::NEG(d.into()).encode(mc),
                (d, ZERO, s2) => {
                    I::MOV(d.into(), s2.into()).encode(mc);
                    I::NEG(d.into()).encode(mc);
                }
                (d, s1, ZERO) => I::MOV(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s2 => {
                    I::SUB(d.into(), s1.into()).encode(mc)
                }
                (d, s1, s2) if d == s1 => {
                    I::SUB(d.into(), s2.into()).encode(mc)
                }
                (d, s1, s2) => {
                    I::MOV(d.into(), s1.into()).encode(mc);
                    I::SUB(d.into(), s2.into()).encode(mc);
                }
            }
        }
        SLL { d, s1, s2 } => todo!(),
        SLT { d, s1, s2 } => todo!(),
        SLTU { d, s1, s2 } => todo!(),
        SRL { d, s1, s2 } => todo!(),
        SRA { d, s1, s2 } => todo!(),
        XOR { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _, _) => { /* nop */ }
                (d, ZERO, ZERO) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s2) => I::MOV(d.into(), s2.into()).encode(mc),
                (d, s1, ZERO) => I::MOV(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s2 => {
                    I::XOR(d.into(), s1.into()).encode(mc)
                }
                (d, s1, s2) if d == s1 => {
                    I::XOR(d.into(), s2.into()).encode(mc)
                }
                (d, s1, s2) => {
                    I::MOV(d.into(), s1.into()).encode(mc);
                    I::XOR(d.into(), s2.into()).encode(mc);
                }
            }
        }
        OR { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _, _) => { /* nop */ }
                (d, ZERO, ZERO) => I::XOR(d.into(), d.into()).encode(mc),
                (d, ZERO, s2) => I::MOV(d.into(), s2.into()).encode(mc),
                (d, s1, ZERO) => I::MOV(d.into(), s1.into()).encode(mc),
                (d, s1, s2) if d == s2 => {
                    I::OR(d.into(), s1.into()).encode(mc)
                }
                (d, s1, s2) if d == s1 => {
                    I::OR(d.into(), s2.into()).encode(mc)
                }
                (d, s1, s2) => {
                    I::MOV(d.into(), s1.into()).encode(mc);
                    I::OR(d.into(), s2.into()).encode(mc);
                }
            }
        },
        AND { d, s1, s2 } => {
            match (d, s1, s2) {
                (ZERO, _, _) => { /* nop */ }
                (d, ZERO, _) => I::XOR(d.into(), d.into()).encode(mc),
                (d, _, ZERO) => I::XOR(d.into(), d.into()).encode(mc),
                (d, s1, s2) if d == s2 => {
                    I::AND(d.into(), s1.into()).encode(mc)
                }
                (d, s1, s2) if d == s1 => {
                    I::AND(d.into(), s2.into()).encode(mc)
                }
                (d, s1, s2) => {
                    I::MOV(d.into(), s1.into()).encode(mc);
                    I::AND(d.into(), s2.into()).encode(mc);
                }
            }
        },
        ECALL {} => {
            // Copy syscall argument registers into kernel memory.
            let a0 = sysargs[0..].as_mut_ptr() as usize; // EAX
            let a1 = sysargs[1..].as_mut_ptr() as usize; // ECX
            let a2 = sysargs[2..].as_mut_ptr() as usize; // EDX

            eprintln!("a0 {:X}", a0);
            eprintln!("a1 {:X}", a1);
            eprintln!("a2 {:X}", a2);

            todo!()
        }
        EBREAK {} => {
            todo!() // Same as ECALL (but with return value 1 instead of 0)
        }
        FENCE { im } => todo!(),
    }
}
