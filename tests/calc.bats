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
