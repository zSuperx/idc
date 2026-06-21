from gen import *

V = "LirVal"


def binop(name: str):
    return InstrInfo(
        name=name,
        fields=[V, V, V],
        defs=[0],
        uses=[1, 2],
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
    InstrInfo("copy", [V, V], defs=[0], uses=[1]),
    InstrInfo("alloc", [V, "StringId"], defs=[0], uses=[]),
    InstrInfo("param", [V, "StringId", "usize"], defs=[0], uses=[]),
    InstrInfo("load", [V, V], defs=[0], uses=[1]),
    InstrInfo("store", [V, V], defs=[], uses=[0, 1]),
    # terminators
    InstrInfo("retv", is_terminator=True),
    InstrInfo("ret", [V], defs=[], uses=[0], is_terminator=True),
    InstrInfo("br", [V, "BB", "BB"], defs=[], uses=[0], is_terminator=True),
    jmp := InstrInfo("jmp", ["BB"], is_terminator=True),
]


iclass = Arch(name="lir", enum="LirInstr", val=V, instrs=INSTRUCTIONS, uncond_jump=jmp)
