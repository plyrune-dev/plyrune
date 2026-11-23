#!/bin/sh
# Run from the repository root. Use a temporary, pinned upstream checkout.
set -eu
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
git init -q "$tmp/source"
git -C "$tmp/source" fetch -q --depth=1 https://github.com/writfold-dev/splicery 632a90fd96e920fbac7ee5818f989739c1617062
git -C "$tmp/source" checkout -q --detach FETCH_HEAD
cat > "$tmp/compose.yaml" <<'YAML'
services:
  splice:
    image: ghcr.io/tauten-dev/proofs-toolchain@sha256:5c1fd168f399e028db85e4a76c8e742abc7bdf93207e5270e54b36636b40a2d8
    platform: linux/amd64
    volumes: ['./source:/src:ro']
    cpus: 4
    mem_limit: 8g
    working_dir: /work
    command: [bash, -ec, 'cp -r /src/crates /src/Cargo.toml .; cp -r /src/proofs/. proofs/; timeout 5400 sh proofs/gate.sh']
YAML
docker compose -p plyrune-splice-proof -f "$tmp/compose.yaml" run --rm splice
