#!/usr/bin/env bats

REPO="$(dirname "$BATS_TEST_DIRNAME")"
BIN="$REPO/target/debug/llvm-calc"

setup_file() {
    cargo build --manifest-path "$REPO/Cargo.toml" >&2
}

@test "bare number" {
    run "$BIN" "7"
    [ "$status" -eq 0 ]
    [[ "$output" == *"7"* ]]
}

@test "addition" {
    run "$BIN" "3 + 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"7"* ]]
}

@test "subtraction" {
    run "$BIN" "10 - 3"
    [ "$status" -eq 0 ]
    [[ "$output" == *"7"* ]]
}

@test "multiplication" {
    run "$BIN" "3 * 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"12"* ]]
}

@test "division" {
    run "$BIN" "20 / 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"5"* ]]
}

@test "float literal" {
    run "$BIN" "3.14"
    [ "$status" -eq 0 ]
    [[ "$output" == *"3.14"* ]]
}

@test "float arithmetic" {
    run "$BIN" "3.14 * 2"
    [ "$status" -eq 0 ]
    [[ "$output" == *"6.28"* ]]
}

@test "float addition" {
    run "$BIN" "1.5 + 2.5"
    [ "$status" -eq 0 ]
    [[ "$output" == *"4"* ]]
}

@test "float variable" {
    run "$BIN" "x=0.5; x * 6"
    [ "$status" -eq 0 ]
    [[ "$output" == *"3"* ]]
}

@test "mul takes precedence over add" {
    run "$BIN" "2 + 3 * 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"14"* ]]
}

@test "parens override precedence" {
    run "$BIN" "(2 + 3) * 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"20"* ]]
}

@test "div takes precedence over add" {
    run "$BIN" "10 / 2 + 3"
    [ "$status" -eq 0 ]
    [[ "$output" == *"8"* ]]
}

@test "nested parens" {
    run "$BIN" "3 + 4 * (2 - 1)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"7"* ]]
}

@test "ir subcommand output contains LLVM IR" {
    run "$BIN" ir "1 + 1"
    [ "$status" -eq 0 ]
    [[ "$output" == *"define i32 @main()"* ]]
}

@test "default command does not print IR" {
    run "$BIN" "1 + 1"
    [ "$status" -eq 0 ]
    [[ "$output" != *"define i32 @main()"* ]]
}

@test "-O2 flag produces optimized IR" {
    run "$BIN" -O2 "1 + 1"
    [ "$status" -eq 0 ]
    [[ "$output" == *"2"* ]]
}

@test "optimized and unoptimized JIT produce the same result" {
    run "$BIN" "a=3; b=4; a*a + b*b"
    [ "$status" -eq 0 ]
    local unopt="$output"
    run "$BIN" -O2 "a=3; b=4; a*a + b*b"
    [ "$status" -eq 0 ]
    [ "$output" = "$unopt" ]
}

@test "ir -O3 promotes recursive calls to tail calls" {
    run "$BIN" ir -O3 "fn fib(n) = if n < 2 then n else fib(n-1) + fib(n-2); fib(10)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"tail call double @fib"* ]]
}

@test "ir subcommand prints IR and exits without running" {
    run "$BIN" ir "3 + 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"define i32 @main()"* ]]
}

@test "ir subcommand respects -O2" {
    run "$BIN" ir -O2 "3 + 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"@main()"* ]]
}

@test "ir -o writes IR to file" {
    local out
    out="$(mktemp --suffix=.ll)"
    run "$BIN" ir "3 + 4" -o "$out"
    [ "$status" -eq 0 ]
    [ -s "$out" ]
    grep -q "define i32 @main()" "$out"
    rm -f "$out"
}

@test "ir output pipes into opt" {
    if ! command -v opt &>/dev/null; then skip "opt not found"; fi
    local ll out
    ll="$(mktemp --suffix=.ll)"
    out="$(mktemp --suffix=.ll)"
    "$BIN" ir "3 + 4 * (2 - 1)" -o "$ll"
    run opt -O2 -S "$ll" -o "$out"
    [ "$status" -eq 0 ]
    [ -s "$out" ]
    rm -f "$ll" "$out"
}

@test "ir output compiles and runs via llc + clang" {
    if ! command -v llc &>/dev/null; then skip "llc not found"; fi
    if ! command -v clang &>/dev/null; then skip "clang not found"; fi
    local ll asm bin
    ll="$(mktemp --suffix=.ll)"
    asm="$(mktemp --suffix=.s)"
    bin="$(mktemp)"
    "$BIN" ir "3 + 4 * (2 - 1)" -o "$ll"
    llc --relocation-model=pic "$ll" -o "$asm"
    clang "$asm" -o "$bin"
    run "$bin"
    [ "$status" -eq 0 ]
    [[ "$output" == *"7"* ]]
    rm -f "$ll" "$asm" "$bin"
}

@test "parse error exits non-zero" {
    run "$BIN" "bad @@"
    [ "$status" -ne 0 ]
    [[ "$output" == *"parse error"* ]]
}

@test "single variable" {
    run "$BIN" "x=42; x"
    [ "$status" -eq 0 ]
    [[ "$output" == *"42"* ]]
}

@test "variable used in expression" {
    run "$BIN" "x=10; x * 3"
    [ "$status" -eq 0 ]
    [[ "$output" == *"30"* ]]
}

@test "multiple variables" {
    run "$BIN" "x=10; y=20; x*y"
    [ "$status" -eq 0 ]
    [[ "$output" == *"200"* ]]
}

@test "variable in sub-expression" {
    run "$BIN" "a=3; b=4; a*a + b*b"
    [ "$status" -eq 0 ]
    [[ "$output" == *"25"* ]]
}

@test "ir shows alloca without -O" {
    run "$BIN" ir "x=5; x"
    [ "$status" -eq 0 ]
    [[ "$output" == *"alloca double"* ]]
}

@test "ir -O2 folds variables away" {
    run "$BIN" ir -O2 "x=5; x"
    [ "$status" -eq 0 ]
    [[ "$output" != *"alloca"* ]]
}

@test "undefined variable exits non-zero" {
    run "$BIN" "x + 1"
    [ "$status" -ne 0 ]
    [[ "$output" == *"undefined variable"* ]]
}

@test "negative literal" {
    run "$BIN" "-5"
    [ "$status" -eq 0 ]
    [[ "$output" == *"-5"* ]]
}

@test "negate expression" {
    run "$BIN" "-(3 + 4)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"-7"* ]]
}

@test "negate variable" {
    run "$BIN" "x=10; -x"
    [ "$status" -eq 0 ]
    [[ "$output" == *"-10"* ]]
}

@test "double negation" {
    run "$BIN" "-(-3)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"3"* ]]
}

@test "negation in expression" {
    run "$BIN" "10 + -3"
    [ "$status" -eq 0 ]
    [[ "$output" == *"7"* ]]
}

@test "repl recovers from undefined variable" {
    run bash -c "printf 'x + 1\n3 + 4\n' | $BIN"
    [ "$status" -eq 0 ]
    [[ "$output" == *"undefined variable"* ]]
    [[ "$output" == *"7"* ]]
}

@test "repl evaluates multiple expressions" {
    run bash -c "printf '3 + 4\n2 * 5\n' | $BIN"
    [ "$status" -eq 0 ]
    [[ "$output" == *"7"* ]]
    [[ "$output" == *"10"* ]]
}

@test "repl recovers from parse errors" {
    run bash -c "printf 'bad @@\n3 + 4\n' | $BIN"
    [ "$status" -eq 0 ]
    [[ "$output" == *"parse error"* ]]
    [[ "$output" == *"7"* ]]
}

@test "repl exits cleanly on EOF" {
    run bash -c "echo '' | $BIN"
    [ "$status" -eq 0 ]
}

@test "if true branch taken" {
    run "$BIN" "if 1 < 2 then 42 else 0"
    [ "$status" -eq 0 ]
    [[ "$output" == *"42"* ]]
}

@test "if false branch taken" {
    run "$BIN" "if 2 < 1 then 42 else 0"
    [ "$status" -eq 0 ]
    [[ "$output" == *"0"* ]]
}

@test "if with greater-than" {
    run "$BIN" "if 5 > 3 then 1 else 2"
    [ "$status" -eq 0 ]
    [[ "$output" == *"1"* ]]
}

@test "if with equality" {
    run "$BIN" "if 3 == 3 then 9 else 0"
    [ "$status" -eq 0 ]
    [[ "$output" == *"9"* ]]
}

@test "if with not-equal" {
    run "$BIN" "if 3 != 4 then 5 else 0"
    [ "$status" -eq 0 ]
    [[ "$output" == *"5"* ]]
}

@test "if with less-or-equal" {
    run "$BIN" "if 3 <= 3 then 1 else 0"
    [ "$status" -eq 0 ]
    [[ "$output" == *"1"* ]]
}

@test "if with greater-or-equal" {
    run "$BIN" "if 4 >= 5 then 1 else 0"
    [ "$status" -eq 0 ]
    [[ "$output" == *"0"* ]]
}

@test "if with variable in condition" {
    run "$BIN" "n=5; if n < 10 then n else 0"
    [ "$status" -eq 0 ]
    [[ "$output" == *"5"* ]]
}

@test "if with expressions in branches" {
    run "$BIN" "a=3; b=4; if a < b then a*a else b*b"
    [ "$status" -eq 0 ]
    [[ "$output" == *"9"* ]]
}

@test "nested if" {
    run "$BIN" "x=5; if x < 3 then 1 else if x < 7 then 2 else 3"
    [ "$status" -eq 0 ]
    [[ "$output" == *"2"* ]]
}

@test "if ir contains conditional branch" {
    run "$BIN" ir "if 1 < 2 then 42 else 0"
    [ "$status" -eq 0 ]
    [[ "$output" == *"br i1"* ]]
}

@test "if ir contains phi node" {
    run "$BIN" ir "if 1 < 2 then 42 else 0"
    [ "$status" -eq 0 ]
    [[ "$output" == *"phi double"* ]]
}

@test "while loop body never runs when condition is false" {
    run "$BIN" "while 0 < 0 do (1)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"0"* ]]
}

@test "while loop runs once" {
    run "$BIN" "i=1; while i <= 1 do (i = i + 1; i)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"2"* ]]
}

@test "while loop sum 1 to 5" {
    run "$BIN" "i=1; s=0; while i <= 5 do (s = s + i; i = i + 1; s)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"15"* ]]
}

@test "while loop mutation visible after loop" {
    run "$BIN" "i=1; s=0; dummy = while i <= 5 do (s = s + i; i = i + 1; s); s"
    [ "$status" -eq 0 ]
    [[ "$output" == *"15"* ]]
}

@test "while loop iterative fibonacci fib(10)" {
    run "$BIN" "a=0; b=1; i=2; while i <= 10 do (tmp=a+b; a=b; b=tmp; i=i+1; b)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"55"* ]]
}

@test "while loop ir contains loop_header block" {
    run "$BIN" ir "i=1; while i <= 3 do (i = i + 1; i)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"loop_header"* ]]
}

@test "while loop ir contains conditional branch" {
    run "$BIN" ir "i=1; while i <= 3 do (i = i + 1; i)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"br i1"* ]]
}

@test "function definition and call" {
    run "$BIN" "fn double(x) = x * 2; double(21)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"42"* ]]
}

@test "function with two arguments" {
    run "$BIN" "fn add(a, b) = a + b; add(3, 4)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"7"* ]]
}

@test "function using if expression" {
    run "$BIN" "fn abs(x) = if x < 0 then -x else x; abs(-7)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"7"* ]]
}

@test "function called with expression argument" {
    run "$BIN" "fn square(x) = x * x; square(3 + 4)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"49"* ]]
}

@test "recursive fibonacci fib(10)" {
    run "$BIN" "fn fib(n) = if n < 2 then n else fib(n-1) + fib(n-2); fib(10)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"55"* ]]
}

@test "multiple function definitions" {
    run "$BIN" "fn square(x) = x * x; fn hyp(a, b) = square(a) + square(b); hyp(3, 4)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"25"* ]]
}

@test "function call in variable binding" {
    run "$BIN" "fn double(x) = x * 2; r = double(10); r + 1"
    [ "$status" -eq 0 ]
    [[ "$output" == *"21"* ]]
}

@test "undefined function exits non-zero" {
    run "$BIN" "nope(1)"
    [ "$status" -ne 0 ]
    [[ "$output" == *"undefined function"* ]]
}

@test "function ir contains define" {
    run "$BIN" ir "fn double(x) = x * 2; double(3)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"define double @double"* ]]
}
