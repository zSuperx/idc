# Still looking for a name

## A NOTE ON AI POLICY

This is an educational project, and was written from scratch using no AI
assisted coding. The only AI involved was me asking where to learn more about
certain topics (and occassionally asking the AI itself for explanations). 

As such, I will **not** be incorporating any kind of AI generated code for the
forseeable future. 

The only exception to this rule is code that is truly out of the scope of
compilers and its subtopics, like printing error messages or QoL derive macros.
(Both of these examples once existed within the repository but were replaced
with hand-written implementations due to the generated code being unpleasant to
work with).

## TODOs

These are just some goals I have for this compiler project. It's not in order
and I haven't researched every goal yet. They're just things I want to do at
some point.

### Unfinished/Planned:

- [ ] Create a test suite:
    - This should include a separate source file for every type of test (i.e. `test_cast.idc`)
    - Write a script or harness to test them all.

- [ ] Improve parser API:
    - Right now the parser will just `die!` if it encounters a token it didn't
      expect. This should be replaced with `ParseError` behavior, so the parent
      parser can try another production rule
    - For example, I plan to make `sizeof` work for both expressions AND types.
      This requires me to parse either an `expr` OR a `type` once I see a
      `sizeof` token.

- [ ] Add better diagnostics. 
    - Don't `die!()` upon facing any error
    - Instead, try and bail out and parse/type check the next logical object
      - This can include parse errors or sema errors
      - If we fail to parse/sema check a statement, check the next statement (if in block)
      - If we fail on a function, check the next function

- [ ] Look into (ab)using `lea` for math!

- [ ] Get a better convergence algorithm for `LIVE_{IN,OUT}` set computation:
    - It can be vastly improved by popping items out of a worklist. When a
      basic block sees its `LIVE` sets change, it should push its predecessors
      into the worklist.
    - The issue is that there is currently no way to find a basic block's
      predecessors. This is a TODO for when that API gets overhauled. For now,
      it just loops forever until convergence.

- [ ] Improve the IRFunction builder API
    - First of all, define type aliases
      - type STFunction = IRFunction<IRInstr, IRType>
      - type x86Function = IRFunction<x86Instr, LLType>
    - Then define custom impls on those types (i.e. print)
    - Add function arguments to the IRFunction struct and a way to lower them for function calls

- [ ] Add optimizations.
    - This should be a lot easier to do now that I have a visitor dfs function in place on the builder API

- [ ] Function calls (x86):
    - This is harder than I first thought.
    - If I choose to go via the System V ABI, I will have to implement structs
    - Therefore I need to obey their rules. Allegedly...
        - The first 6 "eightbyte" arguments are passed via registers
        - This includes structs. HOWEVER, if there are e.g. 3 registers left
          and a struct has 4 "eightbyte" fields, it gets put completely on the stack

- [ ] Arrays:
    - I really don't understand the frontend implementation:
        - In C, `char x[4096]` means `x` has type `char[4096]`, and it can
          "decay" to a `char*`
        - As of now, this makes no sense to me.

- [ ] Strings & String Literals:
    - This kind of depends on how Arrays go. I feel like the NULL terminated
      route is not a good idea and should be "opt-in". 
    - That is, strings should just be an array of characters, and the language
      itself can construct Dynamic Array Strings via structs.

- [ ] Floating Point values:
    - Might be easy for x86, but god help me when I do RISC-V & ARM since they
      have separate register files for Int and FP
    - Probably look into generalizing the register allocator API first, so
      logic can be reused across architectures and int/fp

- [ ] Type inference:
    - Currently there is type checking with hints, which is what lets you omit
      the type in `let` statements
    - But this should eventually be changed into type inference, likely via the
      Unification Algorithm

- [ ] Add a graph coloring component for regalloc

- [ ] Allocate spills:
    - When the graph coloring algorithm "runs out of colors" to assign (as in
      it goes over 16), that color should be assigned to a stack slot
    - We need a post-alloc hook that counts how many spills happened and
      allocates as much space as needed

### Finished

- [x] `sizeof(x)`:
    - This should be easy, since every type has a size, and every expression
      has a type
    - Probably add `alignof(x)` while you're at it


- [x] Change LirVal::Mem (and x86Val::Mem) to include compex addressing modes:
      - DONT lower Expr::Index to (Deref + Add + Mul) subexpr, as a lot of
        information ends up getting lost 
      - Instead, change the LirVal::Mem API to include base `Reg`, offset `Option<Reg>`,
        scale `usize` and displacement `usize`
        - For types, scale and disp can get away with being unwrapped values
          since the values `1` and `0` can be used to represent their absence
      - This is in hopes of emitting a physical memory operand like `[rbx + rax * 8 + 1]`

- [x] Type casting:
    - Possible syntax:
        - `@i32(x)` / `@*i32(x)` <- this is what I'm doing right now
        - `@(x, i32)`
        - `cast(x, i32)`
        - `x as i32` (Rust) lame
        - `(i32)x` (C) hard to parse
        - `x::cast<i32>` wtf is this
    - Since type casting between primitive types of different sizes implies
      either zero/sign extending or truncating/zeroing leading bits, this also
      means `Zext`, `Sext` and `Trunc` instructions should be added to the
      `LIR` ISA
    - A cast from `T` to `U` where `sizeof(T) > sizeof(U)` should result in
      `Trunc u, t, n`, where `n` = `sizeof(U) * 8`
    - A cast from `U` to `T` should result in either
      `Zext t, u, n` or `Sext t, u, n`, where `n` = `sizeof(T) * 8`
    - When `float` is added to the language, `Trunc` should be used, otherwise
      emit an `And t, (1 << sizeof(T) * 8 - 1)`, which just masks out
      everything. Might have to be careful with signed integers, but think
      about that later
    - Casting should be valid between:
        1. Same sized types (this means all pointers can be cast to and from
           each other)
        2. Any primitive with any other primitive

### Dropped/Deprecated


_(Note: This list was created on 06/22/2026, so it may not have all goals)_
