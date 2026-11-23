#!/bin/sh
# Reproduce the cross-crate translation experiment inside `compose run dev`.
# Stage unchanged Rust files at stable paths in a disposable workspace.
set -eu
root=$(pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"
git init -q source
git -C source fetch -q --depth=1 https://github.com/writfold-dev/splicery 632a90fd96e920fbac7ee5818f989739c1617062
git -C source checkout -q --detach FETCH_HEAD
mkdir -p proofs crates/probe/src
cp "$root/proofs/extract.sh" proofs/
cp -r source/crates/splicery-splice crates/
cp source/Cargo.toml Cargo.toml
cat >crates/probe/Cargo.toml <<'EOF'
[package]
name = "probe"
version = "0.0.0"
edition = "2024"
[dependencies]
splicery-splice = { path = "../splicery-splice" }
EOF
cat >crates/probe/src/lib.rs <<'EOF'
#![no_std]
use splicery_splice::Cut;
pub struct ResourceId(pub u64);
pub fn accepts_resource(expected: ResourceId, actual: ResourceId, a: &Cut, b: &Cut) -> bool {
    expected.0 == actual.0 && splicery_splice::splice::compatible(a, b)
}
EOF
sh proofs/extract.sh crates/probe "$work/out" -- -abort-on-error
