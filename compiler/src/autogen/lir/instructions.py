from gen import *


def binop(name: str):
    return InstrInfo(
        name=name,
        fields=["LirType", "LirVal", "LirVal", "LirVal"],
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
    InstrInfo("copyr", ["LirType", "LirVal", "LirVal"], outs=[1], ins=[2]),
    InstrInfo("alloc", ["LirType", "LirVal", "StringId"], outs=[1], ins=[]),
    InstrInfo("param", ["LirType", "LirVal", "StringId", "usize"], outs=[1], ins=[]),
    InstrInfo("load", ["LirType", "LirVal", "LirVal"], outs=[1], ins=[2]),
    InstrInfo("store", ["LirType", "LirVal", "LirVal"], outs=[], ins=[1, 2]),
    # terminators
    InstrInfo("retv", is_terminator=True),
    InstrInfo("ret", ["LirType", "LirVal"], outs=[], ins=[1], is_terminator=True),
    InstrInfo("br", ["LirVal", "BB", "BB"], outs=[], ins=[0], is_terminator=True),
    jmp := InstrInfo("jmp", ["BB"], is_terminator=True),
]


iclass = InstructionClass(
    arch="lir", enum="LirInstr", val="LirVal", instrs=INSTRUCTIONS, uncond_jump=jmp
)
