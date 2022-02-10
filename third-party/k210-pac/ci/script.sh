#!/bin/bash

set -euxo pipefail

cargo check --target $TARGET
cargo check --target $TARGET --features rt
