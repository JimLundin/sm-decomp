# Independent projectile trail scope

Date: 2026-10-12
Status: assigned
Owner: `/root/independent_gameplay`
Worktree: `/tmp/sm-projectile-trails`
Branch: `decomp-projectile-trails`
Baseline: `b6f30bd`

No existing dependency-ready, unclaimed gameplay ticket is available outside
Ridley ticket138. Claimed tickets78/87/94/96/97 are preserved. Root approved a
new ticket139 for the following naturally grounded independent closure.

## Behavior and ownership

Translate native `HandleProjectileTrails` ($90B6A9), including complete eighteen
slot descending left-then-right timer/script/render execution and its three
movement opcodes B525/B587/B5B3. Creation/placement is already owned by
`projectile_flight` (ticket16); the remaining native closure has no gameplay host
callbacks or MMIO. Preserve zero/one/wrapping timer behavior, native script
termination, frozen render-only execution, wrapping camera coordinates, signed
OAM capacity checks and partial/unaligned word writes. OAM writes can alias
later trail state; preserve native write/read order.

Writable files are new `crates/port/src/projectile_trails.rs`,
`crates/port/tests/projectile_trails.rs`, `tools/port/projectile-trails.toml` and
isolated Rust inventory/fixture producer files. Root owns public `lib.rs`,
catalog, live/family/main registration and integration. No Ridley or
enemy-projectile source ownership is shared.

## Grounded observation

Retained actual-C flight replay:
`out/port/flight-fixed-live/20261010T204643.443282Z/verdict.json`.
Both C baselines record19 `SpawnProjectileTrail` and19 `ProjectileTrail_Func5`
calls. This proves natural creation/placement, not whole trail-frame counts.
New native active-frame instrumentation is required before live admission.

Read-only identity check on2026-10-12:

- Verdict SHA256: `367bf4ce28ff4519581ab52e1411097c3184099c180e90f8a7469080c8f174be`.
- C0 routine log SHA256 matches sealed artifact:
  `1552d0578ea9ba61df9e068e91888c493a928621bff55c4685c2f54b8c6d5a81`.
- Pinned/current `sm_90.c` SHA256 matches recorded source:
  `ad9f103509160abd8a9e0f85f3c3c96187c89f31dd770cf533fa32a97925247a`.
- Pinned/current `sm_93.c` SHA256 matches recorded source:
  `886d3503d26ab29bf712f185e359f9fcb732c42d24f5ccdfddc53b7563a7aa79`.

Native `DrawProjectiles` and `sub_9382FD` in `sm_93.c` call the whole trail
boundary. The new actual-C diagnostic will count total/nonempty/script/OAM
frames, first/last natural activity and opcode use, preserving full before/after
WRAM plus the exact native ROM view for captured active frames.

## Verification and cleanup

Use existing Rust `family-verify` against actual compiled pinned `sm_90.c`, full
WRAM/raw return, direct callbacks and actual-ROM lifecycle chains. Include all
18slots, timer0/1/2/FFFF, movement wrap, all native scripts, left/right ordering,
frozen rendering, clip0/255/256/FFFF, OAM0/508/509/511/512/8200/FFFF and metadata/
future-slot aliases. Independent controls must detect partial writes and
unsupported-script refusal before mutation.

Run four fresh C/mixed processes with nonzero exclusive Rust whole-frame
ownership and naturally active material evidence; compare state/effects and
require visible zero CPUfallback for the selected prefix. Root retains its
broader current projectile evidence and serializes combined live integration.

Shared worker root is `/home/jim/sm-decomp/out/pipeline`, capacity4; experiments
initially at most60seconds. No Python entry point runs. Clean only the owned
finished `out/port/projectile-trails*` runs with ordinary/deduplicate utility
passes and reauthenticate; root preserves source snapshots before worktree
removal. Research so far produced no verification run or disposable output.
Available disk recorded by `df -B1`:212,257,050,624bytes.

## Submitted implementation and completed offline evidence

Commit `cf5f2d8` implements the complete native trail script/render frame and
three movement opcodes. Ten independent semantic tests and strict Clippy pass.
Actual compiled pinned C versus Rust passes all 3,316 controlled cases across
four routines at
`out/port/projectile-trails/offline/1791838049353069-2-0/verdict.toml`.
The controlled family took 147.633 seconds; wrapper/bank90/worker builds took
1.191/1.636/0.364 seconds. Candidate SHA256 is
`9b4fcd099ced710581da54fac03b3478aa81daee21b90c194bd1b7c53181108f`.

The fresh actual-C 9,000-input inventory at
`out/port/projectile-trails/natural-9000-corrected` records 3,824 native whole
frames, 167 active frames, 47 script frames and 160 OAM-changing frames.
Natural activity spans input frames 2,311 through 8,773. Fourteen full before/
after WRAM captures pass the actual-C/Rust family at
`out/port/projectile-trails/natural-offline/1791838495334191-2-0/verdict.toml`
in 19.634 seconds. None of the three movement opcodes occur in this prefix;
their coverage remains controlled offline evidence. Natural whole-frame Rust
ownership requires the integration owner's fresh merged live replay.

## Cleanup and retained source dependencies

Scoped ordinary cleanup reclaimed zero bytes. Controlled-family dedup reclaimed
700,579,840 bytes across 9,948 snapshots; natural-family dedup reclaimed
3,670,016 bytes across 56 snapshots. Total reclaimed is 704,249,856 bytes.
Both families authenticate after cleanup. Authentication receipts are
`.scratch/decomp-port/projectile-trails-offline-authentication.toml` and
`.scratch/decomp-port/projectile-trails-natural-authentication.toml`.
The natural diagnostic authenticates all 352 sealed artifacts; its ordinary
and deduplicate utility passes both report zero bytes because its
`diagnostic-complete` format is outside the cleanup utility's supported run
formats. Every compiler object, log and capture is sealed, so all remain.
The failed original diagnostic and stale-tooling rejection runs are retained;
their scoped utility passes reclaimed zero bytes.

The integration owner must remove `/tmp/sm-projectile-trails` as a Git worktree
and restore the staged exact source snapshot from
`/tmp/sm-projectile-trails-retained` at the original path. It contains 43 exact
source/manifest files totaling 2,706,017 bytes, plus links to retained pinned C
and its archive. These source dependencies include the original controlled
family's pipeline `main.rs` and `family.rs`, which differ from current main.
The natural family's exact tooling/dependency context is independently frozen
at `/tmp/sm-projectile-trails-natural-source`, with 42 files/773,801 bytes.
Use `family-authenticate --checkout /tmp/sm-projectile-trails-natural-source`
for its durable authentication while retaining the candidate's absolute path
under `/tmp/sm-projectile-trails`. Both snapshots have `retention.toml` hash
lists and no Git metadata or build trees.

The natural inventory producer authenticates against the natural source
snapshot. Worktree removal and disposable target/binary cleanup remain with
the integration owner, followed by both family reauthentications. Available
disk recorded after verification is 207,863,308,288 bytes.
