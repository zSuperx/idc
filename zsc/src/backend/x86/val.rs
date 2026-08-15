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
            0 => x86Val::reg(RegName::DI as usize, size),
            1 => x86Val::reg(RegName::SI as usize, size),
            2 => x86Val::reg(RegName::D as usize, size),
            3 => x86Val::reg(RegName::C as usize, size),
            4 => x86Val::reg(RegName::R8 as usize, size),
            5 => x86Val::reg(RegName::R9 as usize, size),
            _ => x86Val::mem(RegName::BP as usize, n.saturating_sub(6) as i128 + 8, size),
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
                    8 => "byte",
                    16 => "word",
                    32 => "dword",
                    64 => "qword",
                    s => panic!("Unsupported size: {s}"),
                };
                match imm {
                    ..0 => f.write_fmt(format_args!(
                        "{width_spec} [{} - {}]",
                        sized_reg(reg, 64),
                        imm.abs()
                    )),
                    0 => f.write_fmt(format_args!("{width_spec} [{}]", sized_reg(reg, 8))),
                    1.. => f.write_fmt(format_args!(
                        "{width_spec} [{} + {}]",
                        sized_reg(reg, 64),
                        imm.abs()
                    )),
                }
            }
        }
    }
}

fn sized_reg(reg: usize, size: usize) -> String {
    let names = match reg {
        0 => ["al", "ax", "eax", "rax"],
        1 => ["bl", "bx", "ebx", "rbx"],
        2 => ["cl", "cx", "ecx", "rcx"],
        3 => ["dl", "dx", "edx", "rdx"],
        4 => ["sil", "si", "esi", "rsi"],
        5 => ["dil", "di", "edi", "rdi"],
        6 => ["spl", "sp", "esp", "rsp"],
        7 => ["bpl", "bp", "ebp", "rbp"],
        8 => ["r8b", "r8w", "r8d", "r8"],
        9 => ["r9b", "r9w", "r9d", "r9"],
        10 => ["r10b", "r10w", "r10d", "r10"],
        11 => ["r11b", "r11w", "r11d", "r11"],
        12 => ["r12b", "r12w", "r12d", "r12"],
        13 => ["r13b", "r13w", "r13d", "r13"],
        14 => ["r14b", "r14w", "r14d", "r14"],
        15 => ["r15b", "r15w", "r15d", "r15"],
        virt => {
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

#[repr(usize)]
pub enum RegName {
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


// Sized Registers

pub const AL: x86Val = x86Val::reg(RegName::A as usize, 8);
pub const AX: x86Val = x86Val::reg(RegName::A as usize, 16);
pub const EAX: x86Val = x86Val::reg(RegName::A as usize, 32);
pub const RAX: x86Val = x86Val::reg(RegName::A as usize, 64);

pub const BL: x86Val = x86Val::reg(RegName::B as usize, 8);
pub const BX: x86Val = x86Val::reg(RegName::B as usize, 16);
pub const EBX: x86Val = x86Val::reg(RegName::B as usize, 32);
pub const RBX: x86Val = x86Val::reg(RegName::B as usize, 64);

pub const CL: x86Val = x86Val::reg(RegName::C as usize, 8);
pub const CX: x86Val = x86Val::reg(RegName::C as usize, 16);
pub const ECX: x86Val = x86Val::reg(RegName::C as usize, 32);
pub const RCX: x86Val = x86Val::reg(RegName::C as usize, 64);

pub const DL: x86Val = x86Val::reg(RegName::D as usize, 8);
pub const DX: x86Val = x86Val::reg(RegName::D as usize, 16);
pub const EDX: x86Val = x86Val::reg(RegName::D as usize, 32);
pub const RDX: x86Val = x86Val::reg(RegName::D as usize, 64);

pub const SIL: x86Val = x86Val::reg(RegName::SI as usize, 8);
pub const SI: x86Val = x86Val::reg(RegName::SI as usize, 16);
pub const ESI: x86Val = x86Val::reg(RegName::SI as usize, 32);
pub const RSI: x86Val = x86Val::reg(RegName::SI as usize, 64);

pub const DIL: x86Val = x86Val::reg(RegName::DI as usize, 8);
pub const DI: x86Val = x86Val::reg(RegName::DI as usize, 16);
pub const EDI: x86Val = x86Val::reg(RegName::DI as usize, 32);
pub const RDI: x86Val = x86Val::reg(RegName::DI as usize, 64);

pub const SPL: x86Val = x86Val::reg(RegName::SP as usize, 8);
pub const SP: x86Val = x86Val::reg(RegName::SP as usize, 16);
pub const ESP: x86Val = x86Val::reg(RegName::SP as usize, 32);
pub const RSP: x86Val = x86Val::reg(RegName::SP as usize, 64);

pub const BPL: x86Val = x86Val::reg(RegName::BP as usize, 8);
pub const BP: x86Val = x86Val::reg(RegName::BP as usize, 16);
pub const EBP: x86Val = x86Val::reg(RegName::BP as usize, 32);
pub const RBP: x86Val = x86Val::reg(RegName::BP as usize, 64);

pub const R8B: x86Val = x86Val::reg(RegName::R8 as usize, 8);
pub const R8W: x86Val = x86Val::reg(RegName::R8 as usize, 16);
pub const R8D: x86Val = x86Val::reg(RegName::R8 as usize, 32);
pub const R8: x86Val = x86Val::reg(RegName::R8 as usize, 64);

pub const R9B: x86Val = x86Val::reg(RegName::R9 as usize, 8);
pub const R9W: x86Val = x86Val::reg(RegName::R9 as usize, 16);
pub const R9D: x86Val = x86Val::reg(RegName::R9 as usize, 32);
pub const R9: x86Val = x86Val::reg(RegName::R9 as usize, 64);

pub const R10B: x86Val = x86Val::reg(RegName::R10 as usize, 8);
pub const R10W: x86Val = x86Val::reg(RegName::R10 as usize, 16);
pub const R10D: x86Val = x86Val::reg(RegName::R10 as usize, 32);
pub const R10: x86Val = x86Val::reg(RegName::R10 as usize, 64);

pub const R11B: x86Val = x86Val::reg(RegName::R11 as usize, 8);
pub const R11W: x86Val = x86Val::reg(RegName::R11 as usize, 16);
pub const R11D: x86Val = x86Val::reg(RegName::R11 as usize, 32);
pub const R11: x86Val = x86Val::reg(RegName::R11 as usize, 64);

pub const R12B: x86Val = x86Val::reg(RegName::R12 as usize, 8);
pub const R12W: x86Val = x86Val::reg(RegName::R12 as usize, 16);
pub const R12D: x86Val = x86Val::reg(RegName::R12 as usize, 32);
pub const R12: x86Val = x86Val::reg(RegName::R12 as usize, 64);

pub const R13B: x86Val = x86Val::reg(RegName::R13 as usize, 8);
pub const R13W: x86Val = x86Val::reg(RegName::R13 as usize, 16);
pub const R13D: x86Val = x86Val::reg(RegName::R13 as usize, 32);
pub const R13: x86Val = x86Val::reg(RegName::R13 as usize, 64);

pub const R14B: x86Val = x86Val::reg(RegName::R14 as usize, 8);
pub const R14W: x86Val = x86Val::reg(RegName::R14 as usize, 16);
pub const R14D: x86Val = x86Val::reg(RegName::R14 as usize, 32);
pub const R14: x86Val = x86Val::reg(RegName::R14 as usize, 64);

pub const R15B: x86Val = x86Val::reg(RegName::R15 as usize, 8);
pub const R15W: x86Val = x86Val::reg(RegName::R15 as usize, 16);
pub const R15D: x86Val = x86Val::reg(RegName::R15 as usize, 32);
pub const R15: x86Val = x86Val::reg(RegName::R15 as usize, 64);
