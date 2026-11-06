#!/usr/bin/env bash
# Builds the web version into dist/web. Used by CI and by Vercel, whose build image has no Rust.
# Installs what is missing: the Rust toolchain, the wasm target, and the wasm-bindgen CLI that
# matches Cargo.lock. Rune's build script is taken from the revision Cargo.toml pins.
set -euo pipefail
cd "$(dirname "$0")/.."

export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
if ! command -v rustup >/dev/null; then
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable --no-modify-path
fi
# Current stable: Vercel's image ships an older default toolchain that Rune doesn't build on.
rustup toolchain install stable --profile minimal --target wasm32-unknown-unknown --component rust-src
export RUSTUP_TOOLCHAIN=stable

v=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/version = "\(.*\)"/\1/p')
if [[ $(wasm-bindgen --version 2>/dev/null) != "wasm-bindgen $v" ]]; then
    curl -sSLf "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/$v/wasm-bindgen-$v-x86_64-unknown-linux-musl.tar.gz" |
        tar xz -C "${CARGO_HOME:-$HOME/.cargo}/bin" --strip-components=1 "wasm-bindgen-$v-x86_64-unknown-linux-musl/wasm-bindgen"
fi

rev=$(sed -n 's|^rune = .*rev = "\([0-9a-f]*\)".*|\1|p' Cargo.toml)
rune=target/rune
if [[ $(git -C "$rune" rev-parse HEAD 2>/dev/null) != "$rev" ]]; then
    rm -rf "$rune"
    git clone --filter=blob:none -q https://github.com/anomalyco/rune "$rune"
    git -C "$rune" checkout -q "$rev"
fi

"$rune/scripts/web-build.sh" runesweeper --out dist/web --no-html
cp web/index.html dist/web/index.html
cp packaging/runesweeper.png dist/web/icon.png
