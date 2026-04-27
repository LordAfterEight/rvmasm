#[repr(u8)]
#[allow(non_camel_case_types, dead_code)]
pub enum OpCodes {
    // ============================================================
    // Load / Store (0xA_)
    // ============================================================

    /// OP(7) - RDE(5) - IMM(20)
    /// Loads an immediate 20-bit value to register RDE.
    LOAD_IMM = 0xA0,

    /// OP(7) - RS1(5) - IMM(20)
    /// Writes the value of register RS1 to the immediate 20-bit address.
    STOR_IMM = 0xA1,

    /// OP(7) - RDE(5) - IMM(20)
    /// Loads an immediate 20-bit value to the upper 20 bits of register RDE.
    LDUP_IMM = 0xA2,

    /// OP(7) - RDE(5) - RS1(5) - xxx
    /// Loads a byte from the address stored in register RS1 to RDE.
    LOAD_BYTE = 0xA3,

    /// OP(7) - RS1(5) - RS2(5) - xxx
    /// Writes the value from register RS1 to the address stored in register RS2.
    STOR_BYTE = 0xA4,

    /// OP(7) - RDE(5) - RS1(5) - xxx
    /// Loads a word from the address stored in register RS1 to RDE.
    LOAD_WORD = 0xA5,

    /// OP(7) - RS1(5) - RS2(5) - xxx
    /// Writes the value from register RS1 to the address stored in register RS2.
    STOR_WORD = 0xA6,

    // ============================================================
    // Jump / Branch / Return (0xB_)
    // ============================================================

    /// OP(7) - IMM(25)
    /// Unconditionally jumps to the immediate 25-bit address.
    JUMP_IMM = 0xB0,

    /// OP(7) - RS1(5) - xxx
    /// Unconditionally jumps to the address stored in register RS1.
    JUMP_REG = 0xB1,

    /// OP(7) - IMM(25)
    /// Unconditionally branches to the immediate 25-bit address. Writes the current position
    /// to the address the stack pointer is pointing to before jumping.
    BRAN_IMM = 0xB2,

    /// OP(7) - RS1(5) - xxx
    /// Unconditionally branches to the address stored in register RS1. Writes the current position
    /// to the address the stack pointer is pointing to before jumping.
    BRAN_REG = 0xB3,

    /// OP(7) - xxx
    /// Used to return from a branch to the previous position. Reads the last value from the
    /// "stack" and sets the program counter to it.
    RTRN = 0xB4,

    /// OP(7) - RS1(5) - RS2(5) - RS3(5)
    /// Compares registers RS1 and RS2, jumps to the address stored in register RS3 if equal.
    JUEQ_REG = 0xB5,

    /// OP(7) - RS1(5) - RS2(5) - RS3(5)
    /// Compares registers RS1 and RS2, branches to the address stored in register RS3 if equal. Writes the current position
    /// to the address the stack pointer is pointing to before jumping.
    BREQ_REG = 0xB6,

    /// OP(7) - SIG(1) - IMM(19) - xxx
    /// Adds IMM to the program counter. SIG is a sign bit that determines whether IMM is negative (0) or positive(1). This
    /// jump is unconditional.
    JUMP_REL = 0xB7,

    /// OP(7) - SIG(1) - IMM(19) - xxx
    /// Adds IMM to the program counter. SIG is a sign bit that determines whether IMM is negative (0) or positive(1). This
    /// branch is unconditional. Writes the current position to the address the stack pointer is pointing to before branching.
    BRAN_REL = 0xB8,

    /// OP(7) - xxx
    /// Used to return from a branch to the previous position. Pops the last value from the
    /// "stack" and sets the program counter to it, freeing (setting to zero) the address where the
    /// value was stored.
    RTRN_POP = 0xB9,

    // ============================================================
    // ALU (0xC_)
    // ============================================================

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// Adds the contents of registers RS1 and RS2 and stores the result in register RDE.
    ADD = 0xC0,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// Subtracts the contents of registers RS1 and RS2 and stores the result in register RDE.
    SUB = 0xC1,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// Raises register RS1 to the power of register RS2, storing the result in register RDE.
    POW = 0xC2,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// ANDs the content of register RS1 and RS2, storing the result to register RDE.
    AND = 0xC3,

    /// OP(7) - RDE(5) - IMM(20)
    /// ORs the content of register RDE with the 20-bit immediate value.
    ORI = 0xC4,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// ORs the content of register RS1 and RS2, storing the result to register RDE.
    ORR = 0xC5,

    /// OP(7) - RDE(5) - RS1(5) - RS2(5) - xxx
    /// XORs the content of register RS1 and RS2, storing the result to register RDE.
    XOR = 0xC6,

    // ============================================================
    // Stack (0xD_)
    // ============================================================

    PUSH_RGST = 0xD0,

    POP_RGST  = 0xD1,

    // ============================================================
    // System / Control (0xF_)
    // ============================================================

    /// OP(7) - xxx
    /// Makes the core jump to its reset vector, reading the value stored inside and sets the
    /// program counter to it.
    RSET_SOFT = 0xF0,

    /// OP(7) - xxx
    /// Makes the core jump to its reset vector, reading the value stored inside and sets the
    /// program counter to it. Resets all registers.
    RSET_HARD = 0xF1,

    /// OP(7) - xxx
    HALT = 0xF2,

    /// OP - xxx
    NOOP = 0xF3,

    /// OP(7) - core_index(5) - type(5)
    /// Sends an interrupt to the core specified by core_index. The type of interrupt is determined
    /// by the type specifier.
    IRPT_SEND = 0xF4,
}

#[derive(Clone, Copy)]
pub enum OperandKind {
    None,
    Reg,
    Addr,
    RegReg,
    RegAddr,
    RegRegReg,
}

impl OpCodes {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0xA0 => Some(Self::LOAD_IMM),
            0xA1 => Some(Self::STOR_IMM),
            0xA2 => Some(Self::LDUP_IMM),
            0xA3 => Some(Self::LOAD_BYTE),
            0xA4 => Some(Self::STOR_BYTE),
            0xA5 => Some(Self::LOAD_WORD),
            0xA6 => Some(Self::STOR_WORD),
            0xB0 => Some(Self::JUMP_IMM),
            0xB1 => Some(Self::JUMP_REG),
            0xB2 => Some(Self::BRAN_IMM),
            0xB3 => Some(Self::BRAN_REG),
            0xB4 => Some(Self::RTRN),
            0xB5 => Some(Self::JUEQ_REG),
            0xB6 => Some(Self::BREQ_REG),
            0xB7 => Some(Self::JUMP_REL),
            0xB8 => Some(Self::BRAN_REL),
            0xB9 => Some(Self::RTRN_POP),
            0xC0 => Some(Self::ADD),
            0xC1 => Some(Self::SUB),
            0xC2 => Some(Self::POW),
            0xC3 => Some(Self::AND),
            0xC4 => Some(Self::ORI),
            0xC5 => Some(Self::ORR),
            0xC6 => Some(Self::XOR),
            0xD0 => Some(Self::PUSH_RGST),
            0xD1 => Some(Self::POP_RGST),
            0xF0 => Some(Self::RSET_SOFT),
            0xF1 => Some(Self::RSET_HARD),
            0xF2 => Some(Self::HALT),
            0xF3 => Some(Self::NOOP),
            0xF4 => Some(Self::IRPT_SEND),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::LOAD_IMM  => "LOAD_IMM",
            Self::STOR_IMM  => "STOR_IMM",
            Self::LDUP_IMM  => "LDUP_IMM",
            Self::LOAD_BYTE => "LOAD_BYTE",
            Self::STOR_BYTE => "STOR_BYTE",
            Self::LOAD_WORD => "LOAD_WORD",
            Self::STOR_WORD => "STOR_WORD",
            Self::JUMP_IMM  => "JUMP_IMM",
            Self::JUMP_REG  => "JUMP_REG",
            Self::BRAN_IMM  => "BRAN_IMM",
            Self::BRAN_REG  => "BRAN_REG",
            Self::RTRN      => "RTRN",
            Self::JUEQ_REG  => "JUEQ_REG",
            Self::BREQ_REG  => "BREQ_REG",
            Self::JUMP_REL  => "JUMP_REL",
            Self::BRAN_REL  => "BRAN_REL",
            Self::RTRN_POP  => "RTRN_POP",
            Self::ADD       => "ADD",
            Self::SUB       => "SUB",
            Self::POW       => "POW",
            Self::AND       => "AND",
            Self::ORI       => "ORI",
            Self::ORR       => "ORR",
            Self::XOR       => "XOR",
            Self::PUSH_RGST => "PUSH_RGST",
            Self::POP_RGST  => "POP_RGST",
            Self::RSET_SOFT => "RSET_SOFT",
            Self::RSET_HARD => "RSET_HARD",
            Self::HALT      => "HALT",
            Self::NOOP      => "NOOP",
            Self::IRPT_SEND => "IRPT_SEND",
        }
    }

    pub fn operands(&self) -> OperandKind {
        match self {
            Self::LOAD_IMM  => OperandKind::RegAddr,
            Self::STOR_IMM  => OperandKind::RegAddr,
            Self::LDUP_IMM  => OperandKind::RegAddr,
            Self::LOAD_BYTE => OperandKind::RegReg,
            Self::STOR_BYTE => OperandKind::RegReg,
            Self::LOAD_WORD => OperandKind::RegReg,
            Self::STOR_WORD => OperandKind::RegReg,
            Self::JUMP_IMM  => OperandKind::Addr,
            Self::JUMP_REG  => OperandKind::Reg,
            Self::BRAN_IMM  => OperandKind::Addr,
            Self::BRAN_REG  => OperandKind::Reg,
            Self::RTRN      => OperandKind::None,
            Self::JUEQ_REG  => OperandKind::RegRegReg,
            Self::BREQ_REG  => OperandKind::RegRegReg,
            Self::JUMP_REL  => OperandKind::Addr,
            Self::BRAN_REL  => OperandKind::Addr,
            Self::RTRN_POP  => OperandKind::None,
            Self::ADD       => OperandKind::RegRegReg,
            Self::SUB       => OperandKind::RegRegReg,
            Self::POW       => OperandKind::RegRegReg,
            Self::AND       => OperandKind::RegRegReg,
            Self::ORI       => OperandKind::RegAddr,
            Self::ORR       => OperandKind::RegRegReg,
            Self::XOR       => OperandKind::RegRegReg,
            Self::PUSH_RGST => OperandKind::Reg,
            Self::POP_RGST  => OperandKind::Reg,
            Self::RSET_SOFT => OperandKind::None,
            Self::RSET_HARD => OperandKind::None,
            Self::HALT      => OperandKind::None,
            Self::NOOP      => OperandKind::None,
            Self::IRPT_SEND => OperandKind::RegReg,
        }
    }

    /// Returns the total instruction size in bytes (opcode + operands) given an addr_width.
    pub fn instruction_size(&self, addr_width: usize) -> usize {
        1 + match self.operands() {
            OperandKind::None       => 0,
            OperandKind::Reg        => 1,
            OperandKind::Addr       => addr_width,
            OperandKind::RegReg     => 2,
            OperandKind::RegAddr    => 1 + addr_width,
            OperandKind::RegRegReg  => 3,
        }
    }
}
