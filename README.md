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

Loops can be written with `while`. 

All values are `f64`.

```sh
# sum 1..5
llvm-calc "i=1; s=0; while i <= 5 do (s = s + i; i = i + 1; s)"

# iterative fibonacci
llvm-calc "a=0; b=1; i=2; while i <= 10 do (tmp=a+b; a=b; b=tmp; i=i+1; b)"

# conditional: absolute value
llvm-calc "x=-7; if x < 0 then -x else x"

# nested if: clamp to [0, 10]
llvm-calc "x=15; lo=0; hi=10; if x < lo then lo else if x > hi then hi else x"
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

Example LLVM IR produced by compiling a fibonacci expression:

```sh
llvm-calc "a=0; b=1; i=2; while i <= 10 do (tmp=a+b; a=b; b=tmp; i=i+1; b)"
55
```

```llvm
llvm-calc ir "a=0; b=1; i=2; while i <= 10 do (tmp=a+b; a=b; b=tmp; i=i+1; b)"
; ModuleID = 'calc'
source_filename = "calc"

@fmt = private constant [4 x i8] c"%g\0A\00"

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %a = alloca double, align 8
  store double 0.000000e+00, ptr %a, align 8
  %b = alloca double, align 8
  store double 1.000000e+00, ptr %b, align 8
  %i = alloca double, align 8
  store double 2.000000e+00, ptr %i, align 8
  %while_result = alloca double, align 8
  store double 0.000000e+00, ptr %while_result, align 8
  br label %loop_header

loop_header:                                      ; preds = %loop_body, %entry
  %i1 = load double, ptr %i, align 8
  %while_cond = fcmp ole double %i1, 1.000000e+01
  br i1 %while_cond, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %loop_header
  %a2 = load double, ptr %a, align 8
  %b3 = load double, ptr %b, align 8
  %add = fadd double %a2, %b3
  %tmp = alloca double, align 8
  store double %add, ptr %tmp, align 8
  %b4 = load double, ptr %b, align 8
  store double %b4, ptr %a, align 8
  %tmp5 = load double, ptr %tmp, align 8
  store double %tmp5, ptr %b, align 8
  %i6 = load double, ptr %i, align 8
  %add7 = fadd double %i6, 1.000000e+00
  store double %add7, ptr %i, align 8
  %b8 = load double, ptr %b, align 8
  store double %b8, ptr %while_result, align 8
  br label %loop_header

loop_exit:                                        ; preds = %loop_header
  %while_result9 = load double, ptr %while_result, align 8
  %0 = call i32 (ptr, ...) @printf(ptr @fmt, double %while_result9)
  ret i32 0
}
```

With optimization passes:

```llvm
llvm-calc ir -O "a=0; b=1; i=2; while i <= 10 do (tmp=a+b; a=b; b=tmp; i=i+1; b)"
; ModuleID = 'calc'
source_filename = "calc"

@fmt = private constant [4 x i8] c"%g\0A\00"

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  br label %loop_header

loop_header:                                      ; preds = %loop_body, %entry
  %while_result.0 = phi double [ 0.000000e+00, %entry ], [ %add, %loop_body ]
  %i.0 = phi double [ 2.000000e+00, %entry ], [ %add7, %loop_body ]
  %b.0 = phi double [ 1.000000e+00, %entry ], [ %add, %loop_body ]
  %a.0 = phi double [ 0.000000e+00, %entry ], [ %b.0, %loop_body ]
  %while_cond = fcmp ugt double %i.0, 1.000000e+01
  br i1 %while_cond, label %loop_exit, label %loop_body

loop_body:                                        ; preds = %loop_header
  %add = fadd double %b.0, %a.0
  %add7 = fadd double %i.0, 1.000000e+00
  br label %loop_header

loop_exit:                                        ; preds = %loop_header
  %0 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @fmt, double %while_result.0)
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
