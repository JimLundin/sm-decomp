Status: resolved
Blocked by: None
Owner: `/root/ridley_captures`
Triage: ready-for-agent

Recover the unpatched natural Ridley fixture supplement from the authenticated
897-capture, 129,712-frame native inventory. All producer, checker and reporting
edits remain in this directory; the integration owner owns shared registration,
progress publication and live seams. Inventory counters and offline parity grant
no public live admission or original-ROM parity.

## Completion evidence

`ridley-natural.toml` contains 670 unpatched independent fixture pairs across
20 native entries. Generation authenticated the original inventory with frozen
`target/debug/sm-pipeline` SHA256
`9d91d2aa83942bbc30c8048dfa82db40fc3f5641d10a08355f922ae19fcc0ae1`.
The 227 direct internals excluded by the signature ABI are 141 `baby`, 80
`baby-state` and six `helper` captures. They remain sealed and are exercised
inside lifecycle fixtures. No shared source, manifest or registry was edited.

Formal actual-C family:
`family/1791860009281747-67154-0/verdict.toml`, status `offline-verified`, 670
pairs, 20 entries, seven pre-gameplay ABI refusals, eight measured actual-C
service calls, zero CPU fallbacks. `family-after-cleanup-authentication.toml`
reauthenticates every retained pair after both cleanup passes. No gameplay
commit or live acceptance is claimed.

The independent std-only `checker.rs`, compiled directly with `rustc`, matches
all 670 full 131,072-byte WRAM outputs, eight host words, raw normalized returns,
ordered effects, and full before-service snapshots before replaying their sealed
actual-C continuations. Three checker tests and the producer's native pointer
kind test pass. `public-support-parity.toml` and `.tsv` record per-capture results,
headers, areas, service counts and public admission. Source/evidence hashes are
retained in `source-evidence.sha256`; 591,494 bytes of small source snapshots
remain without Git metadata or build trees.

Public admission against the pinned original ROM: 574 selected captures admit;
96 refuse. The refusals comprise 92 fixture-only `CallRidleyFunc` states and four
`HandleCeresRidleyGetawayCutscene` boundaries. All 75 opcode captures, 230 script
frames and the other selected public native entries admit. Input ROM SHA256
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`
is distinct from the padded/patched native ROM-view SHA256
`40aaebb25b9646bea2dffdc5e1ae680768a39dd122de9353885e1c7456ceae27`.
The latter refuses every current public FNV admission check; the checker executes
fixtures against the native view and assesses admission against the original
image separately. This is evidence about the declared capture domain.

Cleanup completed after writers finished:
`target/debug/sm-pipeline cleanup --root family/1791860009281747-67154-0 --apply`
reclaims zero bytes; its `--deduplicate --apply` pass reclaims 292,814,848 bytes
and checks 3,350 snapshots. `cargo clean --manifest-path producer/Cargo.toml`
removes producer build artifacts; direct checker/test executables were removed.
No worktree was created. Final free space: 238,317,670,400 bytes. Retain the
original native inventory, generated manifest, source snapshots, family verdict
and capture companions, and shared pipeline/evidence store dependencies.

## Active PPU evidence gap

The formal family authenticator checks the genuine captured 20-byte PPU view,
but that view is `g_snes->my_ppu`, a different object from active I/O in this
headless run. `deps/snesrev-sm/src/snes/snes.c:38` allocates `my_ppu` and
`snes_ppu` separately; line 40 selects `ppu = snes_ppu`. Lines 181–183 and
204–206 route B-bus reads and writes through `snes->ppu`.
`tools/decomp/headless.c:114` selects `RM_MINE`; `sm_cpu_infra.c:1039` runs the
native frame without changing that pointer. `sm_cpu_infra.c:979` changes it only
in the separate comparison path. `sm_rtl.c:469` copies the active view into
`my_ppu` only when loading saves, which this captured run does not do.

The inventory's `ridley_subsystem_inventory.c.in:67` and fixture host's
`ridley_subsystem_host.c.in:171`/`:178` save and load `my_ppu`. Consequently 102
natural captures with real Mode7 activity preserve an unchanged sealed view.
The first checker diagnostics retain this mismatch honestly in
`first-rom-view-*` and `original-rom-*`. The corrected checker separately
reconstructs active register arithmetic and checks exact ordered writes to M7A,
M7A, M7B then MPYM low/high reads against captured effects. Native signed math
is pinned at `deps/snesrev-sm/src/snes/ppu.c:1208`, and latch writes at `:1473`.
Its independently computed active final state is recorded in the TSV but is
unsealed; it does not repair missing native state evidence.

The completed family establishes exact WRAM, host, ordered effects, services and
returns. Full active hardware parity requires an updated capture/restore contract
retaining an active-pointer discriminator and both relevant PPU instances, then
a refreshed native inventory and controlled/natural differential evidence. Root
assigned that follow-up to the isolated `ridley-hardware-recovery` context. Whole
live lifecycle wrappers, SpawnEnemy/item-drop continuation admission and shared
stage publication remain the integration owner's scope.
