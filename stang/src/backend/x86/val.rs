#[derive(Clone, Copy, Debug)]
pub enum x86ValKind {
    Imm(i128),
    Reg(Reg),
    Mem(Reg, i128),
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

    pub const fn reg(reg: Reg, size: usize) -> Self {
        Self {
            kind: x86ValKind::Reg(reg),
            size,
        }
    }

    pub const fn sysv_arg_n(n: usize, size: usize) -> Self {
        match n {
            0 => x86Val::reg(Reg::DI, size),
            1 => x86Val::reg(Reg::SI, size),
            2 => x86Val::reg(Reg::D, size),
            3 => x86Val::reg(Reg::C, size),
            4 => x86Val::reg(Reg::R8, size),
            5 => x86Val::reg(Reg::R9, size),
            _ => x86Val::mem(Reg::BP, n.saturating_sub(6) as i128 * 8 + 8, size),
        }
    }

    /// Memory read of `size` bytes. This will always use the 64-bit register variant
    pub const fn mem(reg: Reg, offset: i128, size: usize) -> Self {
        Self {
            kind: x86ValKind::Mem(reg, offset),
            size,
        }
    }
}

impl std::fmt::Display for x86Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            x86ValKind::Imm(imm) => f.write_fmt(format_args!("{imm}")),
            x86ValKind::Reg(reg) => f.write_fmt(format_args!("{}", sized_reg_str(reg, self.size))),
            x86ValKind::Mem(reg, imm) => {
                let width_spec = match self.size {
                    8 => "byte",
                    16 => "word",
                    32 => "dword",
                    64 => "qword",
                    s => panic!("Unsupported size: {s}"),
                };
                match imm {
                    ..0 => f.write_fmt(format_args!(
                        "{width_spec} [{} - {}]",
                        sized_reg_str(reg, 64),
                        imm.abs()
                    )),
                    0 => f.write_fmt(format_args!("{width_spec} [{}]", sized_reg_str(reg, 64))),
                    1.. => f.write_fmt(format_args!(
                        "{width_spec} [{} + {}]",
                        sized_reg_str(reg, 64),
                        imm.abs()
                    )),
                }
            }
        }
    }
}

fn sized_reg_str(reg: Reg, size: usize) -> String {
    let names = match reg {
        Reg::A => ["al", "ax", "eax", "rax"],
        Reg::B => ["bl", "bx", "ebx", "rbx"],
        Reg::C => ["cl", "cx", "ecx", "rcx"],
        Reg::D => ["dl", "dx", "edx", "rdx"],
        Reg::SI => ["sil", "si", "esi", "rsi"],
        Reg::DI => ["dil", "di", "edi", "rdi"],
        Reg::SP => ["spl", "sp", "esp", "rsp"],
        Reg::BP => ["bpl", "bp", "ebp", "rbp"],
        Reg::R8 => ["r8b", "r8w", "r8d", "r8"],
        Reg::R9 => ["r9b", "r9w", "r9d", "r9"],
        Reg::R10 => ["r10b", "r10w", "r10d", "r10"],
        Reg::R11 => ["r11b", "r11w", "r11d", "r11"],
        Reg::R12 => ["r12b", "r12w", "r12d", "r12"],
        Reg::R13 => ["r13b", "r13w", "r13d", "r13"],
        Reg::R14 => ["r14b", "r14w", "r14d", "r14"],
        Reg::R15 => ["r15b", "r15w", "r15d", "r15"],
        Reg::Virt(virt) => {
            return match size {
                8 => format!("%{virt}b"),
                16 => format!("%{virt}w"),
                32 => format!("%{virt}d"),
                64 => format!("%{virt}q"),
                _ => panic!("Size: {size} not supported"),
            };
        }
    };
    match size {
        8 => names[0],
        16 => names[1],
        32 => names[2],
        64 => names[3],
        _ => {
            panic!("Size: {size} not supported")
        }
    }
    .to_string()
}

// Unsized Register Names

#[derive(Clone, Copy, Debug)]
pub enum Reg {
    A,
    B,
    C,
    D,

    SI,
    DI,
    SP,
    BP,

    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    Virt(usize),
}

// Sized register names - for convenience

pub const AL: x86Val = x86Val::reg(Reg::A, 8);
pub const AX: x86Val = x86Val::reg(Reg::A, 16);
pub const EAX: x86Val = x86Val::reg(Reg::A, 32);
pub const RAX: x86Val = x86Val::reg(Reg::A, 64);

pub const BL: x86Val = x86Val::reg(Reg::B, 8);
pub const BX: x86Val = x86Val::reg(Reg::B, 16);
pub const EBX: x86Val = x86Val::reg(Reg::B, 32);
pub const RBX: x86Val = x86Val::reg(Reg::B, 64);

pub const CL: x86Val = x86Val::reg(Reg::C, 8);
pub const CX: x86Val = x86Val::reg(Reg::C, 16);
pub const ECX: x86Val = x86Val::reg(Reg::C, 32);
pub const RCX: x86Val = x86Val::reg(Reg::C, 64);

pub const DL: x86Val = x86Val::reg(Reg::D, 8);
pub const DX: x86Val = x86Val::reg(Reg::D, 16);
pub const EDX: x86Val = x86Val::reg(Reg::D, 32);
pub const RDX: x86Val = x86Val::reg(Reg::D, 64);

pub const SIL: x86Val = x86Val::reg(Reg::SI, 8);
pub const SI: x86Val = x86Val::reg(Reg::SI, 16);
pub const ESI: x86Val = x86Val::reg(Reg::SI, 32);
pub const RSI: x86Val = x86Val::reg(Reg::SI, 64);

pub const DIL: x86Val = x86Val::reg(Reg::DI, 8);
pub const DI: x86Val = x86Val::reg(Reg::DI, 16);
pub const EDI: x86Val = x86Val::reg(Reg::DI, 32);
pub const RDI: x86Val = x86Val::reg(Reg::DI, 64);

pub const SPL: x86Val = x86Val::reg(Reg::SP, 8);
pub const SP: x86Val = x86Val::reg(Reg::SP, 16);
pub const ESP: x86Val = x86Val::reg(Reg::SP, 32);
pub const RSP: x86Val = x86Val::reg(Reg::SP, 64);

pub const BPL: x86Val = x86Val::reg(Reg::BP, 8);
pub const BP: x86Val = x86Val::reg(Reg::BP, 16);
pub const EBP: x86Val = x86Val::reg(Reg::BP, 32);
pub const RBP: x86Val = x86Val::reg(Reg::BP, 64);

pub const R8B: x86Val = x86Val::reg(Reg::R8, 8);
pub const R8W: x86Val = x86Val::reg(Reg::R8, 16);
pub const R8D: x86Val = x86Val::reg(Reg::R8, 32);
pub const R8: x86Val = x86Val::reg(Reg::R8, 64);

pub const R9B: x86Val = x86Val::reg(Reg::R9, 8);
pub const R9W: x86Val = x86Val::reg(Reg::R9, 16);
pub const R9D: x86Val = x86Val::reg(Reg::R9, 32);
pub const R9: x86Val = x86Val::reg(Reg::R9, 64);

pub const R10B: x86Val = x86Val::reg(Reg::R10, 8);
pub const R10W: x86Val = x86Val::reg(Reg::R10, 16);
pub const R10D: x86Val = x86Val::reg(Reg::R10, 32);
pub const R10: x86Val = x86Val::reg(Reg::R10, 64);

pub const R11B: x86Val = x86Val::reg(Reg::R11, 8);
pub const R11W: x86Val = x86Val::reg(Reg::R11, 16);
pub const R11D: x86Val = x86Val::reg(Reg::R11, 32);
pub const R11: x86Val = x86Val::reg(Reg::R11, 64);

pub const R12B: x86Val = x86Val::reg(Reg::R12, 8);
pub const R12W: x86Val = x86Val::reg(Reg::R12, 16);
pub const R12D: x86Val = x86Val::reg(Reg::R12, 32);
pub const R12: x86Val = x86Val::reg(Reg::R12, 64);

pub const R13B: x86Val = x86Val::reg(Reg::R13, 8);
pub const R13W: x86Val = x86Val::reg(Reg::R13, 16);
pub const R13D: x86Val = x86Val::reg(Reg::R13, 32);
pub const R13: x86Val = x86Val::reg(Reg::R13, 64);

pub const R14B: x86Val = x86Val::reg(Reg::R14, 8);
pub const R14W: x86Val = x86Val::reg(Reg::R14, 16);
pub const R14D: x86Val = x86Val::reg(Reg::R14, 32);
pub const R14: x86Val = x86Val::reg(Reg::R14, 64);

pub const R15B: x86Val = x86Val::reg(Reg::R15, 8);
pub const R15W: x86Val = x86Val::reg(Reg::R15, 16);
pub const R15D: x86Val = x86Val::reg(Reg::R15, 32);
pub const R15: x86Val = x86Val::reg(Reg::R15, 64);
