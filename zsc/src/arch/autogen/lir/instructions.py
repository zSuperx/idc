from gen import *

V = "LirVal"
T = "TypeId"


def binop(name: str):
    return InstrInfo(
        name=name,
        fields=[T, V, V, V],
        defs=[1],
        uses=[2, 3],
        is_terminator=False,
    )


INSTRUCTIONS = [
    binop("add"),
    binop("sub"),
    binop("umul"),
    binop("smul"),
    binop("udiv"),
    binop("sdiv"),
    binop("eq"),
    binop("sgt"),
    binop("sge"),
    binop("slt"),
    binop("sle"),
    binop("ugt"),
    binop("uge"),
    binop("ult"),
    binop("ule"),
    InstrInfo("arg", [T, V, "usize"], defs=[], uses=[1]),
    InstrInfo("call", [T, V], defs=[1], uses=[]),
    InstrInfo("copy", [T, V, V], defs=[1], uses=[2]),
    InstrInfo("alloc", [T, V, "StringId"], defs=[1], uses=[]),
    InstrInfo("param", [T, V, "StringId", "usize"], defs=[1], uses=[]),
    InstrInfo("sparam", [T, V, "StringId", "usize"], defs=[1], uses=[]),
    InstrInfo("load", [T, V, V], defs=[1], uses=[2]),
    InstrInfo("store", [T, V, V], defs=[], uses=[1, 2]),
    InstrInfo("comment", ["String"], fmt="; {v0}"),
    InstrInfo("sext", [T, V, V], uses=[2], defs=[1]),
    InstrInfo("zext", [T, V, V], uses=[2], defs=[1]),
    InstrInfo("trunc", [T, V, V], uses=[2], defs=[1]),
    # terminators
    InstrInfo("retv", is_terminator=True),
    InstrInfo("ret", [T, V], defs=[], uses=[1], is_terminator=True),
    InstrInfo("br", [T, V, "BB", "BB"], defs=[], uses=[1], is_terminator=True),
    jmp := InstrInfo("jmp", ["BB"], is_terminator=True),
]

# LirInstr:
# load ty, dst, (base + offset * scale + imm)
# base = Reg
# offset = Option<Reg>
# scale = usize
# imm = i128


iclass = Arch(name="lir", enum="LirInstr", val=V, instrs=INSTRUCTIONS, uncond_jump=jmp)
