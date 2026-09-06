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

@test "-O flag produces optimized IR" {
    run "$BIN" -O "1 + 1"
    [ "$status" -eq 0 ]
    [[ "$output" == *"2"* ]]
}

@test "optimized and unoptimized JIT produce the same result" {
    run "$BIN" "a=3; b=4; a*a + b*b"
    [ "$status" -eq 0 ]
    local unopt="$output"
    run "$BIN" -O "a=3; b=4; a*a + b*b"
    [ "$status" -eq 0 ]
    [ "$output" = "$unopt" ]
}

@test "ir subcommand prints IR and exits without running" {
    run "$BIN" ir "3 + 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"define i32 @main()"* ]]
}

@test "ir subcommand respects -O" {
    run "$BIN" ir -O "3 + 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"define i32 @main()"* ]]
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

@test "ir -O folds variables away" {
    run "$BIN" ir -O "x=5; x"
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
