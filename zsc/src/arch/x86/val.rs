#[derive(Clone, Copy, Debug)]
pub enum x86Val {
    Imm(i128),
    Reg(x86Reg),
    Mem(usize, x86Reg, i128),
}

impl std::fmt::Display for x86Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            x86Val::Imm(imm) => f.write_fmt(format_args!("{imm}")),
            x86Val::Reg(reg) => f.write_fmt(format_args!("{reg}")),
            x86Val::Mem(size, reg, imm) => {
                let width_spec = match size {
                    1 => "byte",
                    2 => "word",
                    4 => "dword",
                    8 => "qword",
                    _ => unreachable!(),
                };
                match imm {
                    ..0 => f.write_fmt(format_args!("{width_spec} [{reg} - {}]", imm.abs())),
                    0 => f.write_fmt(format_args!("{width_spec} [{reg}]")),
                    0.. => f.write_fmt(format_args!("{width_spec} [{reg} + {}]", imm.abs())),
                }
            }
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum x86Reg {
    Rdi, // arg 1
    Rsi, // arg 2
    Rdx, // arg 3
    Rcx, // arg 4
    R8,  // arg 5
    R9,  // arg 6

    // Temporary registers that functions may change
    Rax, // Return value
    R10,
    R11,

    // Callee-saved registers that will stay unchanged
    Rsp, // Stack pointer
    Rbp, // Frame pointer
    Rbx, // Base pointer
    R12,
    R13,
    R14,
    R15,

    Virt(usize),
}

impl x86Reg {
    pub fn from_usize(num: usize) -> Self {
        match num {
            0 => x86Reg::Rdi,
            1 => x86Reg::Rsi,
            2 => x86Reg::Rdx,
            3 => x86Reg::Rcx,
            4 => x86Reg::R8,
            5 => x86Reg::R9,
            6 => x86Reg::Rax,
            7 => x86Reg::R10,
            8 => x86Reg::R11,
            9 => x86Reg::Rsp,
            10 => x86Reg::Rbp,
            11 => x86Reg::Rbx,
            12 => x86Reg::R12,
            13 => x86Reg::R13,
            14 => x86Reg::R14,
            15 => x86Reg::R15,
            x => panic!("Unmapped registers {x}"),
        }
    }

    pub fn into_usize(self) -> usize {
        match self {
            x86Reg::Rdi => 0,
            x86Reg::Rsi => 1,
            x86Reg::Rdx => 2,
            x86Reg::Rcx => 3,
            x86Reg::R8 => 4,
            x86Reg::R9 => 5,
            x86Reg::Rax => 6,
            x86Reg::R10 => 7,
            x86Reg::R11 => 8,
            x86Reg::Rsp => 9,
            x86Reg::Rbp => 10,
            x86Reg::Rbx => 11,
            x86Reg::R12 => 12,
            x86Reg::R13 => 13,
            x86Reg::R14 => 14,
            x86Reg::R15 => 15,
            x86Reg::Virt(i) => i + 15,
        }
    }
}

impl std::fmt::Display for x86Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            x86Reg::Virt(reg_id) => f.write_fmt(format_args!("%{reg_id}")),
            _ => f.write_str(&format!("{self:?}").to_ascii_lowercase()),
        }
    }
}
