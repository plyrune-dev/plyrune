# Snapshot-bound fixture inspection · developer preview

Tracking: [#42](https://github.com/plyrune-dev/plyrune/issues/42). Scope: first runnable demo of proved subject binding, 24 November 2026. This preview is not the retained-selection `inspect(handle)` API.

Given an independently supplied 64-digit lowercase fixture digest, exact resource path and half-open byte offsets, `plyrune inspect DIGEST PATH START END` reads one fixture on stdin. Admit the fixture and resolve its exact path (including empty files). For a resolved resource, the kernel must refuse unequal snapshot keys **before** considering text ranges; on equal keys preserve the existing text-range decision. Limit displayed bytes to 4096. Emit one JSON object with snapshot identity, path, offsets and text; errors exit 2 with no successful output. `demo` exercises the same path on embedded input without a checkout.

Prove admission and refusal over the extracted function, including mismatch in either half of the key. Check digest framing, conversion from bytes, path lookup, UTF-8 slice and CLI dispatch separately at the installed entrypoint. SHA-256 collision resistance is assumed. No handles persist; presentation of a digest is not authorization. A simple substring on the supplied bytes alone would miss stale subjects.

Delivery: install a version-pinned, checksum-verified Linux x86-64 preview binary to a user directory. No host Rust, Docker or Writfold needed for consumers. The script and binary are trusted distribution inputs; a checksum is not a signature.
