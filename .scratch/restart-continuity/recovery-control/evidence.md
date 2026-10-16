# Existing local recovery client

Prepared by continuity01a143c7 on2026-10-16. This is an exact copy of the
previously retained /tmp/sm-recovery-rpc/client.rs, not a new protocol/service.
The standalone Rust source uses the documented local app-server-control Unix
socket and initialize/initialized handshake. Source2446bytes; binary4609256bytes.

Compiled with rustc --edition=2021 in0.072851293seconds. At17:16:32 the only
RPC invoked was thread/read with root UUID01a144ca-27ba-7130-a9e2-a615c7c80e3a
and includeTurns=false. It succeeded with error=null, the correct root ID,
active status and0returned turns. Receipt: read-only-check.json. No interrupt,
resume, compaction, fork or source mutation was performed by continuity.

SHA256:

- client.rs:4a2fe1eeae4d67e3136ff1dec3c388565f8d8accebebc06585831f8ca9cd8f6d
- client:2e50c828248a487fd72af304908a8b85d75bb37ef45fe870eac4645aa183485d
- read-only-check.json:cd6be8ab87b0e9b7e9e8a9b197cbd9c85c7c76732f296035ae0fa28fb444496b

Official source fetched using the OpenAI Docs skill:
https://developers.openai.com/codex/app-server/ . It documents turn/interrupt
and a terminal turn/completed status interrupted. It does not establish a cause
for the current service stalls or show that compaction would fix them.

The sole watch01a144fc retains control ownership. Continuity authorized one
guarded intentional cancellation only after fresh exact-turn checks, at least
5minutes without model/tool output, an unanswered checkpoint for60seconds,
clean retained root source and no root producers. Children retain claims and
are not interrupted. Confirm the old turn terminal before one same-owner
continuation, and skip that continuation if queued input already started a
healthy new turn. If progress or an actual error resolves the state first,
do not cancel. No source transfer, new writer or generic periodic cancellation.

## Cleanup

After the read-only check, the installed sm-pipeline cleanup command was run
on only this directory, once with --apply and once with --deduplicate --apply.
Both succeeded with bytes=0 and targets=[]; dedup snapshots_checked=0.
Source, exact executable and read-only receipt remain as operational recovery
dependencies. There is no build tree, daemon, worker worktree or intermediate.
Allocated source/binary/receipt bytes4620288; available disk at17:18:26 was
100415918080bytes. Project/installed production source and binaries unchanged.
