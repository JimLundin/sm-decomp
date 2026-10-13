# Samus projectile update and render subsystem

Date: 2026-10-12
Status: submitted; merged-source and live integration pending
Owner: `/root/independent_gameplay`
Worktree: `/tmp/sm-samus-projectile-frame`
Branch: `decomp-samus-projectile-frame`
Baseline: `6f1f97d`
Commit: `0f59de1c6d8b6fa1bb50b2f66b954424406d0f62`

## Submitted scope

The coordinator approved this adjacent coherent Translation scope after ticket139
projectile trails. Only three new files are included in the clean implementer
commit: `crates/port/src/samus_projectile_frame.rs`, its matching Rust test file,
and `tools/port/samus-projectile-frame.toml`. Public module registration, shared
pipeline changes, tickets, progress integration and live replay remain with the
integration owner. Claimed tickets78/87/94/96/97 remain untouched.

The module composes existing `projectiles`, `projectile_flight`,
`projectile_special`, `power_bomb` and `projectile_trails` translations. It owns
the complete native descending ten-slot `HandleProjectile` boundary and the
three scheduled rendering boundaries `DrawPlayerExplosions2`, `sub_9382FD` and
`DrawBombAndProjectileExplosions`. It includes the native bank81 unsigned-Y
spritemap emitter, ordinary/Ceres Mode7 point arithmetic, echo composition,
flicker, signed clipping and exact OAM write/read ordering and aliases.

PLM and HDMA operations use the existing explicit host contracts and the
power-bomb actual-C verification adapter. No new per-routine repository tool or
callback adapter was added. The immutable selector simulates private WRAM with
a refusing host, conservatively leaving callback frames under C ownership. A
host failure aborts ownership; callers must never retry C after a mutating host.

## Actual-C verification

The final complete Offline family passes **476 fixtures across four routines**:
264 update,75 player render,70 elevated render,67 bomb/explosion render.
Evidence is `out/port/samus-projectile-frame/offline/1791841155185563-2-0/verdict.toml`.
It took42.636seconds. Twelve independent Rust semantic tests and strict Clippy
pass. Candidate source SHA256 is
`d6f6a8e93448f882fc3d49b35e687a856473c172389d900ab984745ce517b712`;
manifest SHA256 is
`1467e1fc4604fa69730e0749c6f6ee058c6c4a690d1976b65a6fdb16ee1e5dc0`.

The family covers all native preinstruction mappings, all ten directly mapped
Wave_Shared directions, native-ROM timer/script chains, eight mixed ten-slot
projectile pools with eight continuation frames, rendering cursor/clipping
aliases, eight echo-before-frozen-trails combinations and fourteen retained
naturally active trail captures at each render boundary. Callback requests and
full canonical WRAM/raw returns match the actual C execution. Eleven cases
contain30 ordered actual-C host requests (14PLM,16HDMA), equal byte-for-byte for
both subjects. The native host remains an explicit dependency, preserving
`native_rust_spawn_calls=0`. Natural Rust ownership and original-ROM parity
remain untested for
the newly composed whole subsystem until merged live integration.

The fixture manifest was compacted from18.7MB to669,579bytes by preserving
unchanged fill bytes and referencing existing hash-pinned natural WRAM captures.
A Rust byte-for-byte input comparison checked all457 original cases before
nineteen additional composition cases were added. The natural captures remain
at `out/port/projectile-trails/natural-9000-corrected`; their SHA256 references
are in the manifest and their source diagnostic artifacts remain sealed.

## Preserved oracle boundaries and failed diagnostics

Pinned `DrawPlayerExplosions2` assigns `CalcExplosion_Mode7().y` only to its
visibility temporary, then calls the emitter with an uninitialized `r18` local.
All three actual-C Ceres player fixtures differed at WRAM0x371. Selected Ceres
player draws now refuse before publishing any WRAM change; the elevated render
has no such Mode7 branch and remains verified with Ceres flag fixtures.

Pinned `DrawBombAndProjectileExplosions` discards the Mode7 point and reads
uninitialized draw coordinates. That selected branch also refuses; initializing
a previous type0x500 slot did not establish a stable compiled-C oracle. The
native C was not patched. Ordinary explosions and direct Ceres type0x300/0x500
coordinates remain translated. Independent Rust tests lock both refusal paths.

Mixed-pool failures were reduced through retained before.wram inputs. Timer2
with an unadvanced initial script pointer caused trail creation to read the
preceding native frame; priming the initial script corrected the synthetic
entry state. A synthetic odd low byte became an invalid super-missile linked
slot after acceleration; valid native entry state corrected that fixture.
Existing leaf translations were not changed to accept malformed inputs.

All rejected runs remain diagnostic evidence, including
`1791839850293920-2-0`, `1791840077645225-2-0`,
`1791840883128868-2-0` (missing header in a source snapshot) and
`1791841102079781-2-0` (manifest changed before its producer finished).

## Completion evidence and run cleanup

Both required cleanup utility passes completed for every finished run. Rejected
runs report zero bytes and retain failure artifacts. The first passing457-case
run reclaimed112,984,064bytes; the compacted457-case run reclaimed112,984,064bytes;
the final476-case run reclaimed115,474,432bytes across1,548 snapshots. Total
finished-run reclamation is **341,442,560bytes**; all ordinary passes report0.
Receipts are `.scratch/decomp-port/samus-projectile-frame-*-cleanup.toml` and
`samus-projectile-frame-*-dedup.toml`, plus the original passing-run receipt
`samus-projectile-frame-dedup.toml`.

The final family authenticates after cleanup against exact retained source
context. Receipt:
`.scratch/decomp-port/samus-projectile-frame-476-after-cleanup-authentication.toml`.
The frozen tooling context is `/tmp/sm-samus-projectile-frame-source-final`:
41 exact source/header files totaling779,213bytes, `retention.toml` hashes, and
pinned C/archive links; no Git metadata or build outputs. This context preserves
the main.rs seal from verification before the coordinator's progress registration
edit. Final authentication command:

```sh
/tmp/sm-assignment-target/debug/sm-pipeline family-authenticate --checkout /tmp/sm-samus-projectile-frame-source-final --verdict /home/jim/sm-decomp/out/port/samus-projectile-frame/offline/1791841155185563-2-0/verdict.toml
```

Before integrated worktree removal, the coordinator must restore the staged
snapshot `/tmp/sm-samus-projectile-frame-candidate-retained` at the original
`/tmp/sm-samus-projectile-frame` path. It contains exactly the three committed
source/test/manifest files,697,646bytes, plus a retention hash list. That absolute
candidate/manifest path is a required evidence dependency. Reauthenticate after
restoring it. The implementer worktree is clean and must remain until integration.

Disposable outputs are `/tmp/sm-samus-projectile-frame-target`,
`/tmp/sm-samus-projectile-fixtures-target`, the temporary Rust fixture helper and
`/tmp/sm-samus-projectile-frame-retain`; `/tmp/sm-samus-projectile-frame-source`
is an abandoned184,320byte partial snapshot and has no evidence dependents.
The integration owner handles integrated worktree cleanup as required by
`docs/agents/run-cleanup.md`. Latest disk reading before disposable cleanup:
196,760,637,440bytes available.

`cargo clean --target-dir /tmp/sm-samus-projectile-frame-target` removed247files,
92.9MiB reported, with96,604,160allocated bytes measured before cleanup. The
target directory is gone. The Rust fixture adaptation source is retained for
diagnostic provenance at
`.scratch/decomp-port/samus-projectile-frame-fixture-adaptation.rs`.

`cargo clean --manifest-path /tmp/sm-samus-projectile-fixtures/Cargo.toml
--target-dir /tmp/sm-samus-projectile-fixtures-target` removed682files,
275.6MiB reported. The target was264,122,368allocated bytes at the last earlier
measurement and the final incremental helper build changed it, so that number
is a lower bound rather than an exact reclamation receipt. The target directory
is gone. A disposable26,691,352byte finalizer was copied to
`/tmp/sm-samus-projectile-fixture-finalize` solely to publish progress updates.
Latest available disk after both Cargo clean passes is198,238,670,848bytes.

After all progress updates finished, the temporary Rust fixture-helper directory,
finalizer binary, obsolete retention binary and abandoned partial source snapshot
were removed. Their measured allocated sizes were49,152,26,660,864,4,792,320 and
184,320bytes, totaling31,686,656bytes. Required candidate and tooling snapshots
remain. The family reauthenticates after disposable removal; final receipt is
`.scratch/decomp-port/samus-projectile-frame-final-retention-authentication.toml`.
Final available disk is198,252,969,984bytes. Only the clean unintegrated worktree
and exact evidence/source dependencies remain for the coordinator.

## Integration handoff

Merge/cherry-pick the exact submitted commit; register the public port module
and four native boundaries through existing shared live support. Publish
`translated` and `verified` progress observations only for this submission.
The integration owner publishes `integrated` and `live-exercised` after merged
checks, evidence authentication, all required cleanup and worktree removal.
Combine fresh live replay with the already integrated trails and parallel
Ridley/camera work; exercise actual natural Rust whole-frame ownership and
compare state plus ordered hardware effects. The conservative callback selector
and both Ceres refusals are explicit limits, not whole-game completion.

Shared progress publication completed using the new locked Rust command:
`samus-projectile-frame-0f59de1-translated` and
`samus-projectile-frame-0f59de1-verified` observations name the four scheduled
boundaries and commit/evidence/limitations. The same lane published
`projectile-trails-cf5f2d8-translated` and
`projectile-trails-cf5f2d8-verified` for ticket139. All four updates succeeded;
the initial registry has8 explicitly observed definitions at each of those
two stages,4 observations and0stale observations. No integrated/live stages
were claimed. Update inputs and command receipts are retained as
`.scratch/decomp-port/*progress*.toml` and `.txt`; shared inventory/dashboard are
`.scratch/decomp-progress/registry.json` and `index.html`. The initial refresh
spent over8minutes parsing267MB of legacy manifests before publication. All
lane-owned progress/update processes have finished.

The original coordinator crashed with a stream/decryption error; later messages
to `/root` failed with an agent thread limit error and it disappeared from the
live roster. The active Ridley lane was notified. Recovery tooling and shared
main changes continued to appear, so this durable report preserves the complete
submission without taking over shared integration ownership. Parent messaging
then recovered; the completed commit, source/evidence retention and all progress
observations were sent to `/root` for integration. The maintained source remains
clean in the submitted worktree.
