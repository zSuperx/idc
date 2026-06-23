from gen import *

V = "LirVal"
T = "LirType"

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
    binop("zext"),
    binop("sext"),
    binop("trunc"),
    InstrInfo("copy", [T, V, V], defs=[1], uses=[2]),
    InstrInfo("alloc", [T, V, "StringId"], defs=[1], uses=[]),
    InstrInfo("arg", [T, V, "StringId", "usize"], defs=[1], uses=[]),
    InstrInfo("stkarg", [T, V, "StringId", "usize"], defs=[1], uses=[]),
    InstrInfo("load", [T, V, V], defs=[1], uses=[2]),
    InstrInfo("store", [T, V, V], defs=[], uses=[1, 2]),
    InstrInfo("comment", ["String"], fmt="; {v0}"),
    # terminators
    InstrInfo("retv", is_terminator=True),
    InstrInfo("ret", [T, V], defs=[], uses=[1], is_terminator=True),
    InstrInfo("br", [T, V, "BB", "BB"], defs=[], uses=[1], is_terminator=True),
    jmp := InstrInfo("jmp", ["BB"], is_terminator=True),
]


iclass = Arch(name="lir", enum="LirInstr", val=V, instrs=INSTRUCTIONS, uncond_jump=jmp)
