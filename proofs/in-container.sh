#!/bin/sh
# Executed by Tauten from its private archive of HEAD (or an explicit live run).
set -eu
ln -s /work/proofs/.lake/packages proofs/.lake-packages-tmp
mkdir -p proofs/.lake
mv -T proofs/.lake-packages-tmp proofs/.lake/packages
exec python3 proofs/gate.py "$@"
