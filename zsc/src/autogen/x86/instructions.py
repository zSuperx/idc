from gen import *

V = "x86Val"

def binop(name) -> InstrInfo:
    return InstrInfo(
        name=name,
        fields=[V, V],
        defs=[0],
        uses=[0, 1],
        is_terminator=False,
    )


def jump(name) -> InstrInfo:
    return InstrInfo(name=name, fields=["BB"], defs=[], uses=[], is_terminator=True)


def move(name) -> InstrInfo:
    return InstrInfo(name=name, fields=[V, V], uses=[1], defs=[0])


INSTRUCTIONS = [
    binop("add"),
    binop("sub"),
    InstrInfo("mul", [V, V], defs=[0], uses=[0, 1], fmt="mul {v1}"),
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
    InstrInfo("cmp", [V, V], uses=[0, 1]),
    InstrInfo("comment", ["String"], fmt="; {v0}"),
    InstrInfo("push", [V], defs=[], uses=[0]),
    InstrInfo("pop", [V], defs=[0], uses=[]),
    # terminators
    InstrInfo("call", [V], uses=[0]),
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
    val=V,
    instrs=INSTRUCTIONS,
    uncond_jump=jmp,
)
