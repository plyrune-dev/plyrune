---
rfc: 2026-11-25-fixture-captures
title: "Resolve a retained fixture capture against an independently selected resource"
status: accepted
implementation: in-progress
authors: [Plyrune]
created: 2026-11-25
updated: 2026-11-25
issue: https://github.com/plyrune-dev/plyrune/issues/4
milestone: Next · Exact fixture captures
supersedes: []
superseded_by: []
blocked_on: []
---
# Exact fixture captures

A harness can remember `[start,end)` and the bytes it saw, yet present them to the wrong file. An equal substring, path, or snapshot digest alone is not a capture identity. On 25 November the owner selected an in-process Rust library for this milestone, not a portable handle protocol or a replacement CLI release.

**Authority.** The host binds an admitted immutable UTF-8 fixture to its own store (owner and non-recycled incarnation). Opening a path in that fixture yields an authoritative resource. Its resource ID is the zero-based ordinal in canonical path order, scoped to the entire snapshot, never a content hash. The resource supplies its own subject and bytes; neither comes from the presented capture. A SHA-256 collision or the host recycling its identifiers breaks these assumptions. The host, not an identifier, grants access.

**Mint and resolve.** A capture privately retains its v1 text subject and half-open byte offsets. Mint it only after validating boundaries. To resolve, check the presented capture against the independently opened resource: schema, artifact kind, owner/incarnation, resource ID, full snapshot key, bounds, then UTF-8 boundaries. Return a borrow of that exact immutable slice, including valid empty ranges. No reread, offset rebasing, search, 4096-byte display truncation, or write. Absent paths fail when opening the resource. No unchecked deserialization or cross-process import. An ordinal remains stable when input rows are merely reordered; it may change in a different snapshot, which is not substitutable.

**Evidence.** Prove success and first refusal over production `capture::admit_capture`; mutate schema, kind, store, resource, and snapshot checks. Test the independent path/ordinal mapping, digest conversion and actual returned bytes through the core library, including equal-content paths, reordered input, changed unrelated file, store restart and multibyte boundaries. Host uniqueness, collision resistance, and the byte-slicing bridge are not Lean results. [#5](https://github.com/plyrune-dev/plyrune/issues/5) owns complete literal selection including no-match resources; [#39](https://github.com/plyrune-dev/plyrune/issues/39) owns portable reports. Neither is implied by this contract.
