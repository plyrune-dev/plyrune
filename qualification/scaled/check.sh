#!/bin/sh
set -eu
corpus=$(dirname "$0")/corpus.py
d=$(mktemp -d)
trap 'rm -rf "$d"' EXIT
bad() {
    if python3 "$corpus" grade "$d/base" "$1"; then exit 1; else test "$?" = 1; fi
}
python3 "$corpus" materialize "$d/base"
git -C "$d/base" init -q
git -C "$d/base" add .
bad "$(git -C "$d/base" write-tree)"
sed -i s/legacy_endpoint/service_endpoint/g "$d"/base/config/group0/*
git -C "$d/base" add .
bad "$(git -C "$d/base" write-tree)"
sed -i s/legacy_endpoint/service_endpoint/g "$d"/base/config/group*/*
git -C "$d/base" add .
good=$(git -C "$d/base" write-tree)
python3 "$corpus" grade "$d/base" "$good"
if python3 "$corpus" --refreshed grade "$d/base" "$good"; then exit 1; else test "$?" = 1; fi
echo URL=service_endpoint > "$d/base/config/quiet.env"
git -C "$d/base" add .
python3 "$corpus" --refreshed grade "$d/base" "$(git -C "$d/base" write-tree)"
git -C "$d/base" read-tree "$good"
git -C "$d/base" update-index --chmod=+x docs/notes.txt
bad "$(git -C "$d/base" write-tree)"
