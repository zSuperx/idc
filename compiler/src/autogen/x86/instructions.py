from gen import *


def binop(name) -> InstrInfo:
    return InstrInfo(
        name=name,
        fields=["x86Val", "x86Val"],
        outs=[0],
        ins=[0, 1],
        is_terminator=False,
    )


def jump(name) -> InstrInfo:
    return InstrInfo(name=name, fields=["BB"], outs=[], ins=[], is_terminator=True)


def move(name) -> InstrInfo:
    return InstrInfo(name=name, fields=["x86Val", "x86Val"], ins=[1], outs=[0])


INSTRUCTIONS = [
    binop("add"),
    binop("sub"),
    binop("mul"),
    binop("imul"),
    binop("div"),
    binop("idiv"),
    binop("and"),
    binop("or"),
    binop("xor"),
    binop("shl"),
    binop("shr"),
    binop("sal"),
    binop("sar"),
    binop("lea"),
    #
    move("mov"),
    move("cmove"),
    move("cmovne"),
    move("cmovl"),
    move("cmovle"),
    move("cmovg"),
    move("cmovge"),
    #
    InstrInfo("cmp", ["x86Val", "x86Val"]),
    InstrInfo("comment", ["String"], fmt="; {v0}"),
    InstrInfo("push", ["x86Val"], outs=[], ins=[0]),
    InstrInfo("pop", ["x86Val"], outs=[0], ins=[]),
    # terminators
    InstrInfo("call", ["x86Val"], ins=[0]),
    InstrInfo("ret", is_terminator=True),
    jmp := jump("jmp"),
    jump("je"),
    jump("jne"),
    jump("jl"),
    jump("jle"),
    jump("jg"),
    jump("jge"),
    jump("jo"),
    jump("jno"),
    jump("jz"),
    jump("jnz"),
]

iclass = InstructionClass(
    arch="x86",
    enum="x86Instr",
    val="x86Val",
    instrs=INSTRUCTIONS,
    uncond_jump=jmp,
)
