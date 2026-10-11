# Family, fixtures, live and routine migration evidence

Recorded 2026-10-11 by the `family_live` section agent. Integration owner: `/root`.
Production sources are frozen while the integration owner runs final checks.
Tickets 03, 04 and 05 remain claimed until final integration and cleanup finish.

## Implemented native interfaces

Maintained family verification, fixture generation, callback/PLM adapters,
four-subject live orchestration, decompression acceptance/jobs, RNG jobs and
prefix capture now use typed Rust records, small TOML documents and binary or
numeric trace protocols. Contracts distinguish fresh evidence using
`rust-family-v2`, `rust-callback-v2`, `rust-plm-v2`, `decomp-offline-v2`,
`decomp-live-v2` and `sm-evidence-v2`. Historical sealed evidence was preserved.

The section audit found no `serde_json`, `json!` or `.json` references in maintained
`family_model`, `family`, `fixtures` (including host/projectiles), `plm_fixtures`,
`callback`, `plm_spawn`, `live_model`, `live`, `decomp_model`, `decomp_job`,
`decomp_acceptance`, `rng_job`, `decomp_prefix` or their owned integration tests.
All 142 maintained port/decomp JSON configuration originals were removed after
the exact recursive conversion audit of 141 documents. The acceptance contract
is the separately versioned intentional exception; its TOML digest is
`680bb87eaf8477861282545ad84ee6b07f361b690abb7800911631b70ff63ec4`.

Native golden tests cover all 44 fixture domains. The separate bounded synthetic
PLM allocation matrix retains 1,444 cases and ROM SHA-256
`66891cd1701badac52f5f1ac65a5e71c9b6c33c5c0497fa8b166111f8e69c457`.
That matrix is distinct from the ordinary PLM instruction family in the final batch.

## Compiler and adversarial controls

The integration owner's `/tmp/sm-root-clippy-auth-final.log` records successful
strict workspace Clippy with the bounded RNG authentication integrated.
`/tmp/sm-root-auth-integrated-controls.log` records all eight family integration
tests passing, including malformed matrices, Rust-before-C rejection, inventory
rebinding, stale candidate refusal, independent batch ownership, dispatcher
pointer banks and missing/tampered/stale live predecessor refusal. The focused
readiness feedback control also passes in that log.

`/tmp/sm-root-rng-auth-control.log` records rejection of changed, missing, extra,
symlinked and escaping RNG artifacts and empty inventories. RNG authentication
checks the exact recursive artifact set (including the nested live verdict),
current source and seven implementation identities, three binaries, exhaustive
65,536-seed tables, independent examples, sealed predecessor and all four
subjects across two reset epochs. Packed live reauthenticates this predecessor
and checks its digest again after replay.

## Fresh decomp acceptance and routine live evidence

The accepted pointer is
`out/decomp/native-final-acceptance-retry/decomp-acceptance.toml`, SHA-256
`60dc06f374ce8831c2bf025825b589c401b8adcd7b2c6b98b722f925ccaede73`.
Its run is `out/decomp/native-final-acceptance-retry/1791744699237190354-2`
and verdict SHA-256 is
`f9a84b05129932e4dcc113ba53184febcbaa74c513e8ae44de51b7f1f91717fd`.
It passed 63 fixtures (40 synthetic and 23 captured), three bounded original-ROM
routine calls and eleven rejected fault controls. The fresh typed capture is
`out/decomp/native-final-baseline-retry/1791744406352071262-2/replay/fixtures/manifest.toml`;
acceptance seals its exact digest and full ROM/WRAM/routine/source identities.

`out/decomp/native-final-routine-live/1791744935175187087-2/verdict.toml`
passed four independent subjects and two reset epochs at 9,000 frames, retaining
full checkpoint/effect identity. Every mixed epoch performed 23 Rust decompressions
and zero C decompressions; every C epoch performed 23 C decompressions and zero
Rust decompressions. Seven compression modes were observed, measured CPU fallback
was zero, seven ABI controls and two repeat calls passed. Relative XOR remains
covered synthetically and unexercised in this live prefix. This establishes bounded
C/Rust integration, with whole-game original-ROM parity explicitly separate.

The independent retained verifier authenticated 472 routine artifacts after cleanup
(`/tmp/sm-root-routine-retained-auth.log`). It also authenticated 1,069 artifacts
of the earlier typed RNG run (`/tmp/sm-root-rng-retained-auth.log`); that earlier
RNG run is retained evidence and cannot admit the current `rng_job.rs` source.

## Fresh authenticated RNG and family admission

The fresh RNG verdict is
`out/decomp/native-final-rng-authenticated/1791745975577862843-2/verdict.toml`,
SHA-256 `85b304d628308b92fa38a7108a49f14a299a1878d58d0f27e07dc8659f8fe34b`.
It reports accepted, all 65,536 seeds, 8,783 observed calls in the 9,000-frame mixed
prefix and zero measured CPU fallback. Its nested live verdict is
`live/1791746004381131510-2/verdict.toml`, SHA-256
`e6c68044b6b9732e5344fe65a0e9e357d8ca1a23ce575ea312584873c711d940`.
The integration owner completed the job with exit 0 and the separate native
`rng-authenticate` check passed (`/tmp/sm-root-rng-native-auth.toml`). Fresh RNG
cleanup and deduplication receipts are recorded below; subsequent native retained
reauthentication also returned the accepted verdict
(`/tmp/sm-root-rng-native-auth-after-cleanup.toml`) without an error log.

`out/port/native-final-batch/batch.toml` records successful offline families:

| Family | Cases / routines | Verdict |
| --- | --- | --- |
| system | 475 / 17 | `families/system/evidence/1791745462166260-2-0/verdict.toml` |
| PLM instruction family | 1,184 / 96 | `families/plm/evidence/1791745462184008-2-1/verdict.toml` |
| collision callbacks | 1,752 / 63 | `families/collision/evidence/1791745462315232-2-2/verdict.toml` |

These batch verdicts seal the earlier `main.rs` digest
`cd7e49a2539e07c7f9fd6d7882550dbbec685c29def49f7267628fb08c58e7d2`.
Current `main.rs` is `98786aff81e378ff76ead5336b351eb61cdf051a372ced3e9c8d7aa97bc68c6d`,
so they cannot admit a new current-source live run. The integration owner completed
fresh system verification at `out/port/native-frozen-system/1791746261570451-2-0`
with 475 cases / 17 routines and verdict SHA-256
`d1bf7c846a5dbea994d532c1912ec8f28803394345ba0dce5d2d00749a737ede`.
The minimal final packed run uses that fresh ordinary system family, fresh RNG
and acceptance. The batch PLM instruction verdict is `rust-family-v2`, so it is
not a `--plm-verdict` allocator predecessor, which requires `rust-plm-v2`.

## Exact remaining packed command

From `/tmp/sm-decomp-pipeline`, the integration owner launched:

```sh
target/debug/sm-pipeline live-verify \
  --family-verdict out/port/native-frozen-system/1791746261570451-2-0/verdict.toml \
  --accepted out/decomp/native-final-acceptance-retry/decomp-acceptance.toml \
  --rng-predecessor out/decomp/native-final-rng-authenticated/1791745975577862843-2/verdict.toml \
  --output out/port/native-final-packed-system \
  --frames 9000 --live-jobs 4
```

Keep the integration owner's shared `SM_PIPELINE_ROOT` / `SM_PIPELINE_WORKERS`
settings. The CLI has no `--timeout` option; each 9,000-frame capture has a
240-second timeout. Four fresh packed subjects must match complete state/effects
and satisfy positive exclusive family and predecessor execution. The actual run is
`out/port/native-final-packed-system/1791746319479907826-2`. Its published verdict
reports `mixed-C-Rust-integration`, `live_exercised = true`, 9,000 frames,
`state_effects_match = true`, packed `sm-evidence-v2` storage and zero measured
CPU fallback. Every C subject performed 23 C decompressions / 8,783 C RNG calls
with zero Rust predecessor calls; every mixed subject performed 23 Rust
decompressions / 8,783 Rust RNG calls with zero C predecessor calls. The verdict
SHA-256 is `b37175079604dd3a027167844523f9e882e64e99bef6038e08b48bbd93cb04ab`;
the host binary SHA-256 is
`5ec9c0d173e55afadaf92c74dad28b7c2bc6f1dd684db09b7ffea0416fab1543`.
The recorded capture phase took 385.680367739 seconds. Current predecessor
reauthentication passed before this verdict was sealed. The integrated command
completed with exit 0 after final observation authentication
(`/tmp/sm-root-packed-native-final.toml`, empty companion error log). Packed cleanup
is recorded below; a separate post-cleanup `live-authenticate` receipt is pending.
This closes the bounded migration integration scope and makes no full commercial
readiness or full-corpus claim.

## Cleanup and retained dependencies

The integration owner completed final batch cleanup and deduplication with zero
bytes reclaimed, 10,265 snapshots checked and no remaining targets:
`/tmp/sm-root-cleanup-batch-final.toml` and
`/tmp/sm-root-cleanup-batch-dedup.toml`. The earlier typed RNG cleanup also returned
zero bytes/no targets (`/tmp/sm-root-cleanup-rng-final.toml`). At recording time
`df -h /tmp/sm-decomp-pipeline` reported 203 GiB available.

For the fresh accepted RNG root, cleanup protects recursively inventoried artifact
paths, including nested verdicts, `c`, `rust`, `libnext_random.a`, exhaustive tables,
WRAM snapshots, timelines and logs. Deduplication only hardlinks inventoried
snapshots after byte/hash verification; filenames and verdict bytes are preserved.
Apply cleanup and deduplication to the completed RNG root, then rerun
`rng-authenticate` to establish retained authentication. Write cleanup/auth logs
outside that sealed root: extra files correctly invalidate exact artifact admission.
The integration owner's fresh RNG cleanup returned zero bytes/no targets
(`/tmp/sm-root-cleanup-rng-authenticated.toml`); deduplication reclaimed 176,081,290
bytes from 737 checked snapshots
(`/tmp/sm-root-cleanup-rng-authenticated-dedup.toml`). Retained native reauthentication
returned accepted in `/tmp/sm-root-rng-native-auth-after-cleanup.toml`.
Fresh system cleanup also returned zero bytes/no targets
(`/tmp/sm-root-cleanup-frozen-system.toml`).

The successful packed run cleanup reclaimed 47,304,570 bytes from two unsealed
archives: `libfamilies.a` (23,658,334 bytes) and `librng.a` (23,646,236 bytes).
The exact receipt is `/tmp/sm-root-cleanup-packed-final.toml`. Separate retained
`live-authenticate` passed after cleanup in
`/tmp/sm-root-packed-auth-after-cleanup.toml`:4subjects,148artifacts each,
binary valid, `sm-evidence-v2`, matching verdict seal. Follow-up dedup reclaimed0bytes.

Packed live additionally requires all four `evidence.toml` pointers, its retained
`sm-evidence` reader and referenced shared store objects. Preserve those objects
through final cleanup. Source snapshots referenced by retained evidence must
remain at their sealed paths if implementer worktrees are removed. The integration
owner owns remaining cleanup, disposable harness build removal and ticket resolution.
