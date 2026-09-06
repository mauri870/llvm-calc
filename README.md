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
llc --relocation-model=pic optimized.ll -o out.s
clang out.s -o calc
./calc
```

## Examples

```sh
$ llvm-calc "a=3; b=4; a*a + b*b"
25
```

```llvm
$ llvm-calc ir "a=3; b=4; a*a + b*b"

; ModuleID = 'calc'
source_filename = "calc"

@fmt = private constant [4 x i8] c"%g\0A\00"

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %a = alloca double, align 8
  store double 3.000000e+00, ptr %a, align 8
  %b = alloca double, align 8
  store double 4.000000e+00, ptr %b, align 8
  %a1 = load double, ptr %a, align 8
  %a2 = load double, ptr %a, align 8
  %mul = fmul double %a1, %a2
  %b3 = load double, ptr %b, align 8
  %b4 = load double, ptr %b, align 8
  %mul5 = fmul double %b3, %b4
  %add = fadd double %mul, %mul5
  %0 = call i32 (ptr, ...) @printf(ptr @fmt, double %add)
  ret i32 0
}
```

With optimization passes:

```llvm
$ llvm-calc ir -O "a=3; b=4; a*a + b*b"

; ModuleID = 'calc'
source_filename = "calc"

@fmt = private constant [4 x i8] c"%g\0A\00"

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %0 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @fmt, double 2.500000e+01)
  ret i32 0
}
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
