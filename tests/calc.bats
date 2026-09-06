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

@test "parse error exits non-zero" {
    run "$BIN" "bad @@"
    [ "$status" -ne 0 ]
    [[ "$output" == *"parse error"* ]]
}
