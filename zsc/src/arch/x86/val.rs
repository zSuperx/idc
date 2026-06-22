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

    pub const fn sysv_arg_n(n: usize, size: usize) -> Self {
        match n {
            0 => x86Val::reg(DI, size),
            1 => x86Val::reg(SI, size),
            2 => x86Val::reg(D, size),
            3 => x86Val::reg(C, size),
            4 => x86Val::reg(R8, size),
            5 => x86Val::reg(R9, size),
            _ => x86Val::mem(BP, n.saturating_sub(6) as i128 + 8, size),
        }
    }

    /// Memory read of `size` bytes. This will always use the 8-byte register variant
    pub const fn mem(val: usize, offset: i128, size: usize) -> Self {
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
            x86ValKind::Reg(reg) => f.write_fmt(format_args!("{}", sized_reg(reg, self.size))),
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
                        sized_reg(reg, 8),
                        imm.abs()
                    )),
                    0 => f.write_fmt(format_args!("{width_spec} [{}]", sized_reg(reg, 8))),
                    1.. => f.write_fmt(format_args!(
                        "{width_spec} [{} + {}]",
                        sized_reg(reg, 8),
                        imm.abs()
                    )),
                }
            }
        }
    }
}

fn sized_reg(reg: usize, size: usize) -> String {
    let names = match reg {
        A => ["al", "ax", "eax", "rax"],
        B => ["bl", "bx", "ebx", "rbx"],
        C => ["cl", "cx", "ecx", "rcx"],
        D => ["dl", "dx", "edx", "rdx"],
        SI => ["sil", "si", "esi", "rsi"],
        DI => ["dil", "di", "edi", "rdi"],
        SP => ["spl", "sp", "esp", "rsp"],
        BP => ["bpl", "bp", "ebp", "rbp"],
        R8 => ["r8b", "r8w", "r8d", "r8"],
        R9 => ["r9b", "r9w", "r9d", "r9"],
        R10 => ["r10b", "r10w", "r10d", "r10"],
        R11 => ["r11b", "r11w", "r11d", "r11"],
        R12 => ["r12b", "r12w", "r12d", "r12"],
        R13 => ["r13b", "r13w", "r13d", "r13"],
        R14 => ["r14b", "r14w", "r14d", "r14"],
        R15 => ["r15b", "r15w", "r15d", "r15"],
        virt => {
            return match size {
                1 => format!("%{virt}b"),
                2 => format!("%{virt}w"),
                4 => format!("%{virt}d"),
                8 => format!("%{virt}q"),
                _ => unreachable!(),
            };
        }
    };
    match size {
        1 => names[0],
        2 => names[1],
        4 => names[2],
        8 => names[3],
        _ => unreachable!(),
    }
    .to_string()
}

pub const A: usize = 0;
pub const B: usize = 1;
pub const C: usize = 2;
pub const D: usize = 3;

pub const SI: usize = 4;
pub const DI: usize = 5;
pub const SP: usize = 6;
pub const BP: usize = 7;

pub const R8: usize = 8;
pub const R9: usize = 9;
pub const R10: usize = 10;
pub const R11: usize = 11;
pub const R12: usize = 12;
pub const R13: usize = 13;
pub const R14: usize = 14;
pub const R15: usize = 15;
