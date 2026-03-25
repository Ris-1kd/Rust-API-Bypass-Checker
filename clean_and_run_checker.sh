#!/bin/bash


cd "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"

cargo clean

cargo build


./target/debug/mir-checker ./tests/get/src/main.rs --entry main --domain interval --widening_delay 5 --narrowing_iteration 5 