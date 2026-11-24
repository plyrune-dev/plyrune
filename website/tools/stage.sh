#!/usr/bin/env bash
# Copy exactly the allowlisted site files from website/ into an empty staging directory.
# Nothing else under website/ is ever copied. No build step: the staged tree is the deployable site.
#
# Usage: website/tools/stage.sh DEST_DIR
# Then (by hand, never automatically): copy DEST_DIR into the site repository and push.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"
list="$here/publish-allowlist.txt"
dest="${1:?usage: stage.sh DEST_DIR}"
[ -d "$dest" ] || { echo "dest dir does not exist: $dest" >&2; exit 2; }
existing="$(find "$dest" -mindepth 1 | head -1)"
[ -n "$existing" ] && { echo "refusing to stage into non-empty dir: $dest" >&2; exit 2; }
total=0
count=0
while IFS= read -r rel || [ -n "$rel" ]; do
  case "$rel" in ''|'#'*) continue;; esac
  src="$root/$rel"
  [ -f "$src" ] || { echo "allowlisted file missing: $rel" >&2; exit 1; }
  [ -L "$src" ] && { echo "refusing symlink: $rel" >&2; exit 1; }
  mkdir -p "$dest/$(dirname "$rel")"
  cp -p "$src" "$dest/$rel"
  size="$(stat -c %s "$src")"; total=$((total + size)); count=$((count + 1))
  printf '%8d  %s\n' "$size" "$rel"
done < "$list"
printf '%8d  total (%d files, uncompressed)\n' "$total" "$count"
