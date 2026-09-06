# llvm-calc

A small arithmetic expression compiler built to learn the LLVM toolchain.

The grammar is intentionally trivial so that effort goes into the compiler design.

## Pipeline

```
"3+4*2": parser -> AST -> codegen -> LLVM IR
```

## Usage

```sh
# JIT-execute an expression and print the result
llvm-calc "3 + 4 * (2 - 1)"

# Print LLVM IR only, no execution
llvm-calc ir "3 + 4 * (2 - 1)"

# Run the optimizer before printing IR
llvm-calc ir -O "3 + 4 * (2 - 1)"

# Run with optimization before JIT execution
llvm-calc -O "3 + 4 * (2 - 1)"
```

Supported operators are `+` `-` `*` `/` with standard precedence and parentheses.
All values are `f64` for simplicity.

## Compose with LLVM tools

Because the `ir` subcommand emits standard LLVM IR source code, it composes the LLVM ecosystem:

For example, AOT via llc+clang:

```sh
llvm-calc ir "3 + 4 * (2 - 1)" -o out.ll
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
