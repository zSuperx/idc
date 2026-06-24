# Still looking for a name

# TODO

These are just some goals I have for this compiler project. It's not in order
and I haven't researched every goal yet. They're just things I want to do at
some point.

- [ ] Improve parser API:
    - Right now the parser will just `die!` if it encounters a token it didn't
      expect. This should be replaced with `ParseError` behavior, so the parent
      parser can try another production rule
    - For example, I plan to make `sizeof` work for both expressions AND types.
      This requires me to parse either an `expr` OR a `type` once I see a
      `sizeof` token.


- [ ] Get a better convergence algorithm for LIVE_{IN,OUT} set computation:
    - It can be vastly improved by popping items out of a worklist. When a
      basic block sees its LIVE sets change, it should push its predecessors
      into_usize the worklist.
    - The issue is that there is currently no way to find a basic block's
      predecessors. This is a TODO for when that API gets overhauled. For now,
      it just loops forever until convergence.

- [ ] Type casting:
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

- [ ] Function calls (x86):
    - This should be easy enough, the only hurdle is tracking `use` and `def`
      sets in liveness analysis and register allocation
    - Off the top of my head, `foo(n1, ..., n6)` should use (in
      increasing order) `DI, SI, D, C, R8, R9` and def `A`
    - Also something that might be challenging is (re)storing caller/callee
      saved registers
        - `call foo` should "use" all caller saved registers, so if any are
          `LIVE` right after the `call`, they should be pushed onto the stack
          before the `call` and popped off after

- [ ] `sizeof(x)`:
    - This should be easy, since every type has a size, and every expression
      has a type
    - Probably add `alignof(x)` while you're at it

- [ ] Structs and custom types
    - This might be a doozy. I understand the frontend and field access in the
      backend, but what about casting?
    - Also how do you pass structs by value? It's probably via the stack
      somewhere in the SYS V ABI
    - After some light testing with `clang`, it seems like you just put it on
      the stack, but IN ORDER. E.g. if the first argument is a struct, you push
      the struct onto the stack, THEN stack arguments (if they exists) are
      pushed after that. I believe this in turn renders `rdi` unused?

- [ ] Arrays:
    - Depends on sizeof
    - God knows how I'm gonna do this
    - I plan to keep things simple via `x[i]` == `*(x + i * sizeof(*x))`
    - But I really don't understand the frontend implementation:
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

- [ ] Improve register precoloring
    - It sucks!

- [x] Fix register precoloring:
    - The current model just makes a clique between all physical registers and
      hands that to the graph coloring algorithm. Then it just ignores the
      colors assigned to the physical registers and uses the numbers `0 - 15`
      instead. This is WRONG.
    - True precoloring requires integration with the graph coloring algorithm
      itself, meaning I either need to find a better crate to do it for me, or
      fork it and do it myself (likely).


_(Note: This list was created on 06/22/2026, so it may not have all goals)_
