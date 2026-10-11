# Native pipeline commands and cleanup

Status: resolved
Blocked by: None

Provide Rust process execution/Cargo-cache entry points, replacing Python-only
adapters as callers migrate. Port cleanup and its meaningful filesystem safety
tests to Rust. Switch active cleanup users and instructions to Rust.

## Completion evidence

Final implementation, passing regression/guards and completed cleanup are in
[completion.md](../evidence/completion.md). This supersedes pending notes below.

Native cleanup, leased execution and Cargo artifact cache entry points implemented.
The decomp archive cache built on a miss and returned `hit: true` on retry with
key `43d342f6b7c7d645b6c73b95a51a1f36b7d1dc9640706e2fe0c118850b6a9e1a`.
Rust tests reject malformed evidence, symlinks, escaping paths and changed bytes;
process tests cover concurrent worktrees, cache corruption and killed lease owners.

Cleanup: native finished-run cleanup reclaimed zero unsealed bytes; the final
batch deduplicated 1,008,861,184 bytes and the follow-up whole migration run pass
deduplicated 158,466,048 more. Current final-source batch reused all five retained
verdicts after cleanup. Shared packed live evidence also authenticated after
cleanup. See `../evidence/`; final build-output cleanup and commit pending.
