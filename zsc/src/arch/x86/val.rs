#[derive(Clone, Copy, Debug)]
pub enum x86ValKind {
    Imm(i128),
    Reg(usize),
    Mem(usize, i128),
}

#[derive(Clone, Copy, Debug)]
pub struct x86Val {
    pub kind: x86ValKind,
    pub size: usize,
}

impl x86Val {
    pub fn imm(val: i128, size: usize) -> Self {
        Self {
            kind: x86ValKind::Imm(val),
            size,
        }
    }

    pub const fn reg(val: usize, size: usize) -> Self {
        Self {
            kind: x86ValKind::Reg(val),
            size,
        }
    }

    /// Memory read of `size` bytes. This will always use the 8-byte register variant
    pub fn mem(val: usize, offset: i128, size: usize) -> Self {
        Self {
            kind: x86ValKind::Mem(val, offset),
            size,
        }
    }
}

impl std::fmt::Display for x86Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            x86ValKind::Imm(imm) => f.write_fmt(format_args!("{imm}")),
            x86ValKind::Reg(reg) => {
                f.write_fmt(format_args!("{}", sized_reg(x86Reg::from(reg), self.size)))
            }
            x86ValKind::Mem(reg, imm) => {
                let width_spec = match self.size {
                    1 => "byte",
                    2 => "word",
                    4 => "dword",
                    8 => "qword",
                    _ => unreachable!(),
                };
                match imm {
                    ..0 => f.write_fmt(format_args!(
                        "{width_spec} [{} - {}]",
                        sized_reg(x86Reg::from(reg), 8),
                        imm.abs()
                    )),
                    0 => f.write_fmt(format_args!(
                        "{width_spec} [{}]",
                        sized_reg(x86Reg::from(reg), 8)
                    )),
                    0.. => f.write_fmt(format_args!(
                        "{width_spec} [{} + {}]",
                        sized_reg(x86Reg::from(reg), 8),
                        imm.abs()
                    )),
                }
            }
        }
    }
}

impl From<usize> for x86Reg {
    fn from(value: usize) -> Self {
        match value {
            0 => x86Reg::A,
            1 => x86Reg::B,
            2 => x86Reg::C,
            3 => x86Reg::D,

            4 => x86Reg::SI,
            5 => x86Reg::DI,
            6 => x86Reg::SP,
            7 => x86Reg::BP,

            8 => x86Reg::R8,
            9 => x86Reg::R9,
            10 => x86Reg::R10,
            11 => x86Reg::R11,
            12 => x86Reg::R12,
            13 => x86Reg::R13,
            14 => x86Reg::R14,
            15 => x86Reg::R15,
            _ => unreachable!(),
        }
    }
}

fn sized_reg(reg: x86Reg, size: usize) -> &'static str {
    let names = match reg {
        x86Reg::A => ["al", "ax", "eax", "rax"],
        x86Reg::B => ["bl", "bx", "ebx", "rbx"],
        x86Reg::C => ["cl", "cx", "ecx", "rcx"],
        x86Reg::D => ["dl", "dx", "edx", "rdx"],
        x86Reg::SI => ["sil", "si", "esi", "rsi"],
        x86Reg::DI => ["dil", "di", "edi", "rdi"],
        x86Reg::SP => ["spl", "sp", "esp", "rsp"],
        x86Reg::BP => ["bpl", "bp", "ebp", "rbp"],
        x86Reg::R8 => ["r8b", "r8w", "r8d", "r8"],
        x86Reg::R9 => ["r9b", "r9w", "r9d", "r9"],
        x86Reg::R10 => ["r10b", "r10w", "r10d", "r10"],
        x86Reg::R11 => ["r11b", "r11w", "r11d", "r11"],
        x86Reg::R12 => ["r12b", "r12w", "r12d", "r12"],
        x86Reg::R13 => ["r13b", "r13w", "r13d", "r13"],
        x86Reg::R14 => ["r14b", "r14w", "r14d", "r14"],
        x86Reg::R15 => ["r15b", "r15w", "r15d", "r15"],
    };
    match size {
        1 => names[0],
        2 => names[1],
        4 => names[2],
        8 => names[3],
        _ => unreachable!(),
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum x86Reg {
    A = 0,
    B = 1,
    C = 2,
    D = 3,

    SI = 4,
    DI = 5,
    SP = 6,
    BP = 7,

    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
}
