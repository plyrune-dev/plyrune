#!/bin/sh
# Entry point of the tool services (compose.yaml): run the command as the
# owner of the mounted checkout, not as root. Adapted from tauten's
# docker/entry.sh at bac9fdd (MIT, tauten-dev/tauten; #42): a gate that runs
# as root passes tests a developer's host fails, and a proof record written as
# root would be a file the developer cannot replace. Running as root is an
# explicit, recorded opt-in: PLYRUNE_GATE_ROOT=1.
set -eu
if [ "$(id -u)" = 0 ] && [ "${PLYRUNE_GATE_ROOT:-0}" != 1 ]; then
    uid=$(stat -c %u /work/plyrune)
    gid=$(stat -c %g /work/plyrune)
    if [ "$uid" != 0 ]; then
        # The cache volumes and Charon's cache were created by root; hand them
        # over once.
        for dir in /cache/target /opt/cargo/registry /opt/cargo/git /opt/charon-cache; do
            mkdir -p "$dir"
            [ "$(stat -c %u "$dir")" = "$uid" ] || chown -R "$uid:$gid" "$dir"
        done
        HOME="/tmp/home-$uid"
        mkdir -p "$HOME"
        chown "$uid:$gid" "$HOME"
        export HOME
        exec setpriv --reuid="$uid" --regid="$gid" --clear-groups "$@"
    fi
fi
exec "$@"
