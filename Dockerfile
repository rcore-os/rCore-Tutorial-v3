# syntax=docker/dockerfile:1

# Stage 1 Build QEMU
# - https://www.qemu.org/download/
# - https://wiki.qemu.org/Hosts/Linux#Building_QEMU_for_Linux
# - https://wiki.qemu.org/Documentation/Platforms/RISCV

FROM ubuntu:20.04 as build_qemu

ARG QEMU_VERSION=7.0.0

RUN sed -i 's/archive.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list && \ 
    sed -i 's/security.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list && \ 
    apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y wget build-essential libglib2.0-dev libfdt-dev libpixman-1-dev zlib1g-dev ninja-build

RUN wget https://download.qemu.org/qemu-${QEMU_VERSION}.tar.xz && \
    tar xf qemu-${QEMU_VERSION}.tar.xz && \
    cd qemu-${QEMU_VERSION} && \ 
    ./configure --target-list=riscv64-softmmu,riscv64-linux-user && \
    make -j$(nproc) && \
    make install

# Stage 2 Set Lab Environment
FROM ubuntu:20.04 as build

WORKDIR /tmp

# 2.0. Install general tools
RUN sed -i 's/archive.ubuntu.com/mirrors.tuna.tsinghua.edu.cn/g' /etc/apt/sources.list && \ 
    apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y jq curl git python3 wget build-essential \
    # qemu dependency
    libglib2.0-0 libfdt1 libpixman-1-0 zlib1g \
    # gdb
    gdb-multiarch

# 2.1. Copy qemu
COPY --from=build_qemu /usr/local/bin/* /usr/local/bin

# 2.2. Install Rust
# - https://www.rust-lang.org/tools/install
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static \
    RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup 
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --no-modify-path --profile minimal --default-toolchain nightly

# 2.3. Build env for labs
# See os/Makefile `env:` for example.
# This avoids having to wait for these steps each time using a new container.
COPY rust-toolchain.toml rust-toolchain.toml
RUN rustup target add riscv64gc-unknown-none-elf && \
    cargo install toml-cli cargo-binutils && \
    RUST_VERSION=$(toml get -r rust-toolchain.toml toolchain.channel) && \
    Components=$(toml get -r rust-toolchain.toml toolchain.components | jq -r 'join(" ")') && \
    rustup install $RUST_VERSION && \
    rustup component add --toolchain $RUST_VERSION $Components

# 2.4. Set GDB
RUN ln -s /usr/bin/gdb-multiarch /usr/bin/riscv64-unknown-elf-gdb

# Stage 3 Sanity checking
FROM build as test
RUN qemu-system-riscv64 --version && \
    qemu-riscv64 --version && \
    rustup --version && \
    cargo --version && \
    rustc --version && \
    riscv64-unknown-elf-gdb --version