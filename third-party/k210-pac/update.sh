#!/usr/bin/env bash
set -x
set -e

rm -rf src
mkdir src
svd2rust --target riscv -i k210.svd
mv lib.rs src/
cargo fmt
