# llvm-calc

A small arithmetic expression compiler built to learn the LLVM toolchain.

The grammar is intentionally trivial so that effort goes into the compiler design.

## Pipeline

```
"x=42;3+4*x": parser -> AST -> codegen -> LLVM IR
```

## Usage

```sh
# JIT-execute an expression
llvm-calc "a=3; b=4; a*a + b*b"

# Print LLVM IR
llvm-calc ir "a=3; b=4; a*a + b*b"

# Interactive REPL
llvm-calc
```

The `-O` flag enables additional optimization passes.

Supported operators are `+` `-` `*` `/` with standard precedence and parentheses.
Variable bindings use the form `name=expr;` before the final expression.
All values are `f64`.

## Compose with LLVM tools

Because the `ir` subcommand emits standard LLVM IR, it composes with the LLVM ecosystem.

For example, AOT compilation via llc + clang:

```sh
llvm-calc ir "a=3; b=4; a*a + b*b" -o out.ll
opt -O2 -S out.ll -o optimized.ll
llc --relocation-model=pic out.ll -o out.s
clang out.s -o calc && ./calc
```

## Build

Requires LLVM 22 and Rust stable.

```sh
cargo build --release
```

## Test

```sh
bats tests/calc.bats
```
