#!/usr/bin/env bats

REPO="$(dirname "$BATS_TEST_DIRNAME")"
BIN="$REPO/target/debug/llvm-calc"

setup_file() {
    cargo build --manifest-path "$REPO/Cargo.toml" >&2
}


@test "bare number" {
    run "$BIN" "7"
    [ "$status" -eq 0 ]
    [[ "$output" == *"result: 7"* ]]
}

@test "addition" {
    run "$BIN" "3 + 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"result: 7"* ]]
}

@test "subtraction" {
    run "$BIN" "10 - 3"
    [ "$status" -eq 0 ]
    [[ "$output" == *"result: 7"* ]]
}

@test "multiplication" {
    run "$BIN" "3 * 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"result: 12"* ]]
}

@test "division" {
    run "$BIN" "20 / 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"result: 5"* ]]
}

@test "mul takes precedence over add" {
    run "$BIN" "2 + 3 * 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"result: 14"* ]]
}

@test "parens override precedence" {
    run "$BIN" "(2 + 3) * 4"
    [ "$status" -eq 0 ]
    [[ "$output" == *"result: 20"* ]]
}

@test "div takes precedence over add" {
    run "$BIN" "10 / 2 + 3"
    [ "$status" -eq 0 ]
    [[ "$output" == *"result: 8"* ]]
}

@test "nested parens" {
    run "$BIN" "3 + 4 * (2 - 1)"
    [ "$status" -eq 0 ]
    [[ "$output" == *"result: 7"* ]]
}

@test "output contains LLVM IR" {
    run "$BIN" "1 + 1"
    [ "$status" -eq 0 ]
    [[ "$output" == *"define double @eval()"* ]]
}

@test "parse error exits non-zero" {
    run "$BIN" "bad @@"
    [ "$status" -ne 0 ]
    [[ "$output" == *"parse error"* ]]
}
