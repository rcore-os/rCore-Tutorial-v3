#!/usr/bin/env bash

set -euxo pipefail

if [ -n "${TARGET:-}" ]; then
    rustup target add $TARGET
fi

if [ -n "${CHECK_BLOBS:-}" ]; then
    if [ ! -d gcc/bin ]; then
        mkdir -p gcc
        curl -L https://static.dev.sifive.com/dev-tools/riscv64-unknown-elf-gcc-8.1.0-2018.12.0-x86_64-linux-ubuntu14.tar.gz | tar --strip-components=1 -C gcc -xz
    fi
fi

if [ -n "${RUSTFMT:-}" ]; then
    rustup component add rustfmt
fi
