from gen import *

V = "LirVal"
T = "LirType"


def binop(name: str):
    return InstrInfo(
        name=name,
        fields=[T, V, V, V],
        outs=[1],
        ins=[2, 3],
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
    InstrInfo("copyr", [T, V, V], outs=[1], ins=[2]),
    InstrInfo("alloc", [T, V, "StringId"], outs=[1], ins=[]),
    InstrInfo("param", [T, V, "StringId", "usize"], outs=[1], ins=[]),
    InstrInfo("load", [T, V, V], outs=[1], ins=[2]),
    InstrInfo("store", [T, V, V], outs=[], ins=[1, 2]),
    # terminators
    InstrInfo("retv", is_terminator=True),
    InstrInfo("ret", [T, V], outs=[], ins=[1], is_terminator=True),
    InstrInfo("br", [V, "BB", "BB"], outs=[], ins=[0], is_terminator=True),
    jmp := InstrInfo("jmp", ["BB"], is_terminator=True),
]


iclass = InstructionClass(
    arch="lir", enum="LirInstr", val=V, instrs=INSTRUCTIONS, uncond_jump=jmp
)
