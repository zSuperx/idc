# My notes on ABI

These are some thought-spills I've had on how to handle ABI rules in my backend.

They're very loose and should not be taken as a real spec (yet).

## GOAL

I'd like an interface where the frontend user (the one who emits STIR code)
does NOT have to worry about ABI implementations at all.

Instead, they can provide a target-triple/ABI when requesting STIR to lower to
MC.

This means that user-defined aggregate types like structs must be given to STIR
so it has the necessary info to lower things like passing/returning structs by
value.

## Stupid Example

Below is an example pipeline I thought of for a dumbed down version of the
x86_64 System V ABI (no FP registers, no unions, just structs containing int
primitives).
```
// raw decls (passed from frontend)
struct Point1 = { i32, i32 }
struct Point2 = { i32, i32, Point2 };

sum(struct.Point %0):
.entrypoint.0:
        ptr %1 = getaddr struct.Point %0, i32, #0
        %2 = load i32 from ptr %1
        ptr %3 = getaddr struct.Point %0, i32, #1
        %4 = load i32 from ptr %3
        %5 = add i32 %2, %4
        ret i32, %5

// flatten decls (done by x86 Backend when it first consumes IRFunction)
struct Point2 = { i32, i32, i32, i32 };

// We see that 4 x i32 fits in 2 x i64, so it can be mapped to registers when used as argument

// compress inner values
struct Point2 = { grp64, gpr64 };


; Map struct.Point %0 to %3q, %4q
sum(%6, %7):
.entrypoint.0:
        ptr %1 = getaddr struct.Point %0, i32, #0
        %2 = load i32 from ptr %1
        ptr %3 = getaddr struct.Point %0, i32, #1
        %4 = load i32 from ptr %3
        %5 = add i32 %2, %4
        ret i32, %5

becomes

sum(%6, %7):
.entrypoint.0:
        %2 = get lower 32 bits of %6
        %4 = get upper 32 bits of %6
        %5 = add i32, %2, %4
        ret i32, %5
```

The question is: how do I set up a framework to hand off STIR to the respective
backend?

The issue is really with trying to reuse as much of the STIR ISA as possible.
In the above example, the x86 Backend will supposedly mutate the function
and/or its arguments after mapping an IR VReg to an x86 VReg. So when replacing
value references, should it use x86Value or IRValue? 

## More thought out solution

To avoid creating more layers of IR after the STIR phase, the x86 Backend can
keep a map `v2p: HashMap<IRValue, x86Value>`. That is, it tracks what x86Value
each IRValue resolves to when translating. Thus, we would have (in order):

```
struct Point1 = { i32, i32 }
struct Point2 = { i32, i32, Point2 };

sum(%0: struct.Point):
.entrypoint.0:
        ptr %1 = getaddr struct.Point %0, i32, #0
        %2 = load i32 from ptr %1
        ptr %3 = getaddr struct.Point %0, i32, #1
        %4 = load i32 from ptr %3
        %5 = add i32 %2, %4
        ret i32, %5
```

Flatten and split `struct.Point` into `(%3: i64, %4: i64)`

```
struct Point1 = { i32, i32 };
struct Point2 = { i32, i32, i32, i32 };

sum(%6: i64, %7: i64):
.entrypoint.0:
        ptr %1 = getaddr struct.Point %0, i32, #0
        %2 = load i32 from ptr %1
        ptr %3 = getaddr struct.Point %0, i32, #1
        %4 = load i32 from ptr %3
        %5 = add i32 %2, %4
        ret i32, %5
```

%1 maps to bottom 32 bits of %6 and is therefore no longer a ptr. Same with %3
but the upper 32 bits. So traverse the function and replace:

- `%dst = load i32 from ptr %0` with:
    - `%dst = trunc i64 %6 to i32`
- `%dst = load i32 from ptr %0` with:
    - `%tmp = lshr i64 %6 by 32 bits`
    - `%dst = trunc i64 %tmp to i32`

As we do this, we add the following mappings to `v2p`:
    - `%6 -> rdi`
    - `%7 -> rsi`

> [!NOTE]
> Since we're deleting the %1 and %3 (and maybe more) `def`s, assert there are
> no `use`s anywhere else after the replacement. This should already be the
> case since the ptr + load was only materialized for retrieving the struct
> field. This should probably be a generic helper visitor function.

We should then get:

```
sum(%6, %7):
.entrypoint.0:
        %2 = trunc i64 %6 to i32
        %8 = lshr i64 %6 by 32 bits
        %4 = trunc i64 %8 to i32
```

which can (hopefully) be cleanly translated to...

```
sum:
.sum.prologue:
        push rbp
        mov rbp, rsp
        jmp .entrypoint.0
.entrypoint.0:
        mov %2d, edi
        mov %8q, rdi
        shr %8q, 32
        mov %9d, %2d
        add %9d, %8d
        mov eax, %9d
        jmp .sum.epilogue
.sum.epilogue:
        mov rsp, rbp
        pop rbp
        ret
```

...and then optimized and colored for register allocation!
