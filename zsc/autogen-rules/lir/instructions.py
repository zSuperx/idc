from gen import *

V = "Value"
T = "TypeId"
BB = "BBID"


def binop(name: str):
    return InstrInfo(
        name=name,
        fields=[T, V, V, V],
        defs=[1],
        uses=[2, 3],
        is_terminator=False,
    )


# fmt: off
INSTRUCTIONS = [
    binop("add"),
    binop("sub"),
    binop("umul"),
    binop("smul"),
    binop("udiv"),
    binop("sdiv"),
    binop("eq"),
    binop("ne"),
    binop("sgt"),
    binop("sge"),
    binop("slt"),
    binop("sle"),
    binop("ugt"),
    binop("uge"),
    binop("ult"),
    binop("ule"),

    InstrInfo("comment", ["String"], fmt="; {v0}"),
    
    InstrInfo("call", [V], defs=[0], uses=[]),

    InstrInfo("copy", [V, V], defs=[0], uses=[1]),
    InstrInfo("load", [T, V, V], defs=[1], uses=[2]),
    InstrInfo("store", [T, V, V], defs=[], uses=[1, 2]),

    InstrInfo("alloca", [T, V], defs=[1], uses=[]),

    InstrInfo("sext", [T, V, V], defs=[1], uses=[2]),
    InstrInfo("zext", [T, V, V], defs=[1], uses=[2]),
    InstrInfo("trunc", [T, V, V], defs=[1], uses=[2]),

    # terminators
    InstrInfo("retv", defs=[], uses=[], is_terminator=True, fmt="ret"),
    InstrInfo("ret", [T, V], defs=[], uses=[1], is_terminator=True),
    InstrInfo("br", [V, BB, BB], defs=[], uses=[0], is_terminator=True),
    jmp := InstrInfo("jmp", [BB], is_terminator=True),
]
# fmt: on

# LirInstr:
# load ty, dst, (base + offset * scale + imm)
# base = Reg
# offset = Option<Reg>
# scale = usize
# imm = i128


iclass = Arch(name="lir", enum="LirInstr", val=V, instrs=INSTRUCTIONS, uncond_jump=jmp)
