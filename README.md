# llvm-calc

A small turing-complete arithmetic expression compiler built to learn the LLVM toolchain.

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

The `-O` flag accepts optimization levels like a C compiler: `-O0` (none), `-O1`, `-O2`, `-O3`.

Supported operators are `+` `-` `*` `/` with standard precedence and parentheses.

Variable bindings use the form `name=expr;` before the final expression.

Functions use `fn f(x) = x*2;f(4)` syntax.

Loops can be written with `while`. 

All values are `f64`.

```sh
# sum 1..5
llvm-calc "i=1; s=0; while i <= 5 do (s = s + i; i = i + 1; s)"

# fib with user-defined functions
llvm-calc "fn fib(n) = if n < 2 then n else fib(n-1) + fib(n-2); fib(10)"
```

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

## LLVM IR

Example LLVM IR produced by compiling a fibonacci program:

```sh
$ llvm-calc ir "fn fib(n) = if n < 2 then n else fib(n-1) + fib(n-2); fib(10)"
```

```llvm
define double @fib(double %0) {
entry:
  %n = alloca double, align 8
  store double %0, ptr %n, align 8
  %n1 = load double, ptr %n, align 8
  %cond = fcmp olt double %n1, 2.000000e+00
  br i1 %cond, label %then, label %else

then:
  %n2 = load double, ptr %n, align 8
  br label %merge

else:
  %n3 = load double, ptr %n, align 8
  %sub = fsub double %n3, 1.000000e+00
  %call = call double @fib(double %sub)
  %n4 = load double, ptr %n, align 8
  %sub5 = fsub double %n4, 2.000000e+00
  %call6 = call double @fib(double %sub5)
  %add = fadd double %call, %call6
  br label %merge

merge:
  %result = phi double [ %n2, %then ], [ %add, %else ]
  ret double %result
}

define i32 @main() {
entry:
  %call = call double @fib(double 1.000000e+01)
  %0 = call i32 (ptr, ...) @printf(ptr @fmt, double %call)
  ret i32 0
}
```

With `-O3`, the full pipeline promotes allocas to SSA, eliminates redundant instructions, and converts calls to `tail call`:

```sh
$ llvm-calc ir -O3 "fn fib(n) = if n < 2 then n else fib(n-1) + fib(n-2); fib(10)"
```

```llvm
define double @fib(double %0) local_unnamed_addr {
entry:
  %cond = fcmp olt double %0, 2.000000e+00
  br i1 %cond, label %common.ret7, label %else

common.ret7:
  %common.ret7.op = phi double [ %add, %else ], [ %0, %entry ]
  ret double %common.ret7.op

else:
  %sub = fadd double %0, -1.000000e+00
  %call = tail call double @fib(double %sub)
  %sub5 = fadd double %0, -2.000000e+00
  %call6 = tail call double @fib(double %sub5)
  %add = fadd double %call, %call6
  br label %common.ret7
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
