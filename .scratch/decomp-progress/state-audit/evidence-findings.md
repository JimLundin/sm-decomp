# Retained progress evidence audit, 2026-10-13

Status: read-only audit complete; reporting changes belong to the coordinator
Owner: `/root/progress_evidence_audit`
Scope: existing source, local issues, retained verdicts and authentication receipts
Checkout observed: `d8a90451695408bbf72017ea04e7f4e915b06dd9`

The registry omits the completed bounded actual-C shared-host checks. The one
new translated and verified native definition is `DisplayViewablePartOfRoom`;
the four Samus projectile roots were already translated and verified. An
independent source review confirms the room-display implementation was missed
by the original map's extractor. Additional retained Ridley v2 evidence
would improve provenance and show its corrected PPU domain, without adding
unique verified definitions or establishing whole-lifecycle live acceptance.
No gameplay, fixture, replay, test, build, integration or Git mutation was run
for this audit. Only this findings file was authored.

## Registry baseline and evidence integrity

The inspected [registry](../registry.json) has 257 observations, 16 stale IDs,
one superseded ID and 165 notices. Directly recomputing unions after excluding
stale and superseded IDs gives 1,460 translated, 924 verified, 38 integrated,
121 live-exercised and 222 translating definitions. Current-checkout context
has 1,460 translated / 38 verified / 38 integrated / 26 live; historical-frozen
context has 911 translated / 911 verified / 106 live. Contexts overlap; these
are independent stage observations rather than a full-game completion measure.
The pinned lexical inventory remains 6,053 definitions / 75,229 C body lines,
revision `578f90b3cc49557bb70060ad033bb90b8cf8ac50`, inventory SHA-256
`7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`.
The registry still describes 834 declared definitions. The earlier
[coverage audit](../../decomp-strategy/coverage-audit.md) owns the historical
122-selected-run/118-authenticated-run audit and the stage interpretation;
those expensive checks were not repeated here.

All 324 unique `(evidence path, SHA-256)` contracts across 323 registry paths
were read and hashed. Exactly two historical hash versions differ, and every
observation using them is already excluded:

| Retained path | Original version that fails | Current bytes | Affected observations |
| --- | --- | --- | --- |
| [recovery-live-ticket-audit-20261013.md](../recovery-live-ticket-audit-20261013.md) | `ea6f058fd61871ce27be2699291c2f7b7ed5363bbf269ad522196adc24cb3658` | `b6e9b9dcba902952ac124eacc9f6e16b2d7e1d454638a230ead4251ae6e4a1be` | `projectile-trails-root-6053-live-20261013`, already stale |
| [ticket-authenticated-internal-mappings.json](../../decomp-strategy/ticket-authenticated-internal-mappings.json) | `b59f7ee8f423aa61c8e6ee4ef0a4af41e47bc42aba2eee6e94a23c6c221fa63c` | `6737fa270cea550bf64ad30e129bebe0fb61b62c0c6595a5173406111189f12a` | `historical-dispatch-1` through `-5`, both translated and verified, plus `-6` translated, all with suffix `-6053-…-accuracy-20261013`, already stale |

The later dispatch records using `6737fa…` match. The fresh trail observation
`projectile-trails-root-6053-live-context-20261013` uses the actual verdict,
post-cleanup authentication and cleanup receipts; it does not reference the
edited ticket-audit note. Four other stale observations use an older inventory.
The superseded observation remains
`ridley-whole-6053-recovered-translated-20261013`; its immutable reviewed
replacement preserves qualified translation credit. No newly stale evidence
or decrease in reported unique counts was found. Preserve original receipt
bytes and the existing exclusions; do not repair history by replacing hashes.

## Shared host: one missing translated/verified definition and outdated assignment state

The report's only host observation is
`shared-game-host-ceres-20261013-translating`, covering `HandleProjectile` and
`DisplayViewablePartOfRoom`, with pending-focused-checks/integration limitations.
That is an assignment-time record, not the latest state. The original
[implementation receipt](../../ceres-continuation/host-implementation.toml)
records submitted-and-merged source `73547105d870f8b05f8c6bcf55b768c223889fdb`,
31 focused Rust tests and explicitly delegated C dependencies. Git merge
`e8c9c3e10ac71bf0377dc00d7de5764a4263b445` includes that source; adapter commit
`3fcc7fd4cbd185d6e922d3b7eec0afd760ead0bf` precedes it.

The original actual-C verdicts own the exact achieved fixture domains:

| Family and original verdict | Cases / native definitions | Material callback behavior | Verdict SHA-256 |
| --- | --- | --- | --- |
| [game-host-projectile](../../../out/port/ceres-continuation/game-host-projectile/1791873394532541-258459-0/verdict.toml) | 7 / 4 | Two ordered HDMA requests in each of two fuse cases; one PLM request in the terrain case; frozen fuse and three drawing passes have no callbacks | `18f355479a34ef10be6196e2c1507f6ded34d522693f068feb1313078492abad` |
| [game-host-room-scroll](../../../out/port/ceres-continuation/game-host-room-scroll/1791873418836128-259326-0/verdict.toml) | 3 / 1 | 17 synchronous VRAM flushes per case, covering foreground/background columns, disabled background and wrapped camera edge | `ddb49cbd79a7249b45745044f9f6c95a7e98671fa74e0b313a7f81bf8c869d1d` |

Exact routine sets:

```text
game-host-projectile:
HandleProjectile
DrawPlayerExplosions2
sub_9382FD
DrawBombAndProjectileExplosions

game-host-room-scroll:
DisplayViewablePartOfRoom
```

Both verdicts state `offline-verified`, `live_exercised = false` and
`original_rom_parity = "not-tested"`. Their full-WRAM, return and ordered
callback comparison retains zero CPU fallback. They do not translate PLM,
HDMA or VRAM dependency behavior out of C. The original
[projectile post-cleanup authentication](../../ceres-continuation/projectile-after-cleanup.stdout)
and [room post-cleanup authentication](../../ceres-continuation/room-after-cleanup.stdout)
are byte-identical to their respective verdicts; the earlier authenticate
stdout files match as well. This audit independently rehashed all 490 sealed
artifacts across these two verdicts and every listed current Rust dependency
and host harness input; all match. This is retained-evidence validation, not
new fixture execution.

Cleanup evidence is
[projectile-cleanup-final.toml](../../ceres-continuation/projectile-cleanup-final.toml),
[projectile-dedup-final.toml](../../ceres-continuation/projectile-dedup-final.toml),
[room-cleanup.toml](../../ceres-continuation/room-cleanup.toml) and
[room-dedup.toml](../../ceres-continuation/room-dedup.toml). The final projectile
passes reclaimed zero bytes; room dedup reclaimed 13,762,560 bytes. The
[checkpoint](../../decomp-parallel/checkpoint-20261013.md) now records that the
integrated host worktree was removed and scoped infrastructure cleanup finished.
The [host ticket](../../ceres-continuation/issues/02-shared-game-host.md) still
says claimed/submitted, and the
[adapter integration ticket](../../ceres-continuation/issues/03-host-adapter-integration.md)
plus [assignment receipt](../../ceres-continuation/started-host-integration.toml)
still leave durable integration completion pending reconciliation.

### Independent room-display source correspondence

The original [source map](../../decomp-strategy/source-coverage-map.json)
explicitly places `DisplayViewablePartOfRoom` at `room_scroll.rs:336` in
`unresolved_mappings` because its selector is an `if symbol != …` guard, and
lists `display` at line 300 among unmapped Rust helpers. The original registry
has only the host `translating` observation for this symbol. It was not already
source-mapped or credited as translated.

Direct review of [native bank80 A176](../../../deps/snesrev-sm/src/sm_80.c)
(line 1657), [native WRAM definitions](../../../deps/snesrev-sm/src/variables.h)
and [room_scroll.rs](../../../crates/port/src/room_scroll.rs) establishes a
reviewed owned alias for the Rust `display` body:

- Rust computes `(BG1SC - BG2SC) * 256 & 0xF800` and stores at WRAM `0x098E`,
  matching native `size_of_bg2`; `BG1SC/BG2SC` are the same bytes `0x0058/59`.
- Both call the native-corresponding block-coordinate setup and then prepare
  exactly 17 columns. Rust `select_blocks` uses the same foreground/background
  logical and VRAM block coordinates, and `column` implements their uploads.
- Background preparation has the same low-bit `layer2_scroll_x` guard at
  `0x091B`. Each iteration calls synchronous `Host::flush_vram` before
  incrementing `layer1_x_block`, `bg1_x_block`, `layer2_x_block` and
  `bg2_x_block` with native 16-bit wrap.
- `room_scroll::dispatch_host` (line 328) selects this executable body despite
  its guarded selector. [game_host.rs](../../../crates/port/src/game_host.rs)
  (lines 126 and 166) selects the typed `RoomDisplay` entry and forwards to
  that body; the adapter alone is not the translation proof.

The source correspondence remains bounded by admitted tile-source extents and
the synchronous host's documented WRAM/effect contract. Actual
`NMI_ProcessVramWriteQueue` hardware behavior stays a C dependency; it does not
inherit translation credit. The three retained actual-C room fixtures above
independently verify that bounded executed domain. The native definition has
29 C body lines and SHA-256
`237dd9a75df93ec6948097713ccef76bbe9ec9b240ff37b1595c5de044b4961b`.
The old source-map/receipt bytes must remain unchanged; record this as a new
source-review observation instead of modifying the frozen historical map.

Reporting recommendations:

- Publish a `translated` observation for `DisplayViewablePartOfRoom` with this
  independent reviewed alias, exact maintained module source hashes and
  admitted host/tile-source limitations. Translated union and current-context
  translated rise from 1,460 to 1,461; translated body lines rise from 16,498
  to 16,527. The original 1,452-map-plus-eight-inline summary stays an earlier
  observation; this adds one newly reviewed mapping.
- Publish a `verified` observation for the five exact roots above with
  `current-checkout` context tied to the authenticated observation-time source
  and original verdicts. `DisplayViewablePartOfRoom` is currently absent from
  every verified set; the other four roots already count. Verified union rises
  from 924 to 925, and current-context verified from 38 to 39. Verified body
  lines rise from 10,074 to 10,103; current-context verified from 775 to 804.
- Record the host as source merged and bounded actual-C checked, while noting
  the incomplete durable integration receipt. Existing source/cleanup evidence
  supports a coordinator review of an `integrated` observation for this host
  domain, but do not infer completed integration solely from unit tests or
  change the infrastructure ledger during this reporting-only audit. If the
  integration owner accepts those existing final checks and cleanup as the
  completed bounded integration, the integrated union becomes 39; no new
  implementation or run is needed to describe the already achieved state.
- Refresh declaration metadata: the registry currently gives
  `DisplayViewablePartOfRoom.declared_in = []`, even though the retained
  [room manifest](../../../tools/port/game-host-room-scroll.toml) declares it.
  [game-host-projectile.toml](../../../tools/port/game-host-projectile.toml)
  is also newer than this catalog. This metadata gap is separate from achieved
  stage evidence. Keep the lexical inventory as the denominator.
- Add no `live-exercised` stage from this host work. The
  [continuation spec](../../ceres-continuation/spec.md) explicitly preserves
  live admission; neither verdict claims live exercise.

## Ridley: later bounded evidence exists; broad stages remain pending

[Ticket 141](../../decomp-port/issues/141-ridley-whole-lifecycle-live.md) remains
claimed for whole-lifecycle integration and admission. The report already
correctly grants qualified source translation to 220 native definitions, not
verification of all 220. Its bounded attack-script scope from ticket 138 is
separate. [Tickets 137–140](../../decomp-port/issues/137-observed-projectile-allocator-scheduler.md)
have final completion sections and their bounded current/live roots are already
published; earlier pending comments are historical records.

The latest isolated contracts are described by the source-owning
[hardware report](../../decomp-port/ridley-hardware-recovery/report.md) and
[whole-candidate handoff](../../decomp-port/ridley-live-finish-handoff-20261013.md):

| Evidence | Achieved domain | SHA-256; matches retained post-cleanup authentication |
| --- | --- | --- |
| [hardware-v2 controlled verdict](../../decomp-port/ridley-hardware-recovery/controlled/1791860698541096-105766-0/verdict.toml) | 6,457 pairs / 33 entries / 305 actual-C callbacks; both PPU views, active discriminator and SNES open bus | `78b129866e34ab71834a10e9563bd6541ededaaeff9234b563996eb6ffbd4c58` |
| [hardware-v2 natural verdict](../../decomp-port/ridley-hardware-recovery/natural/1791861212110218-129925-0/verdict.toml) | 670 genuine unpatched pairs / 20 entries / eight actual-C callbacks, all Ceres header E13F and area 6 | `addb9a189d44f1066dbe6ad723bacd193548bcdc240e34eaf022d167a8b5b9aa` |
| [whole-candidate final Offline verdict](../../../out/recovery-session/ridley-whole-final-offline/1791866846960194-145753-0/verdict.toml) | 6,457 pairs / 33 entries / seven ABI controls / 305 C callbacks | `def8416e90f8e39d6974f4d4afdc3778af2978061737833c8e5bd18a9e82ec69` |
| [candidate native137 Offline verdict](../../../out/recovery-session/ridley-native137-final-offline/1791866852674692-145775-0/verdict.toml) | 1,840 pairs / four declared entries / zero C gameplay callbacks | `a5faf27ff2ebe02a8ed7a82bc1636849148f55645991889643af28e187eb589e` |

Authentication owners are
`ridley-hardware-recovery/controlled-after-cleanup-authentication.toml`,
`ridley-hardware-recovery/natural-after-cleanup-authentication.toml`,
`ridley-live-whole-final-post-cleanup-auth.toml` and
`ridley-live-native137-final-post-cleanup-auth.toml`, all under
`.scratch/decomp-port/`. This audit compared their bytes/hashes with the
original verdicts, without reexecuting their evidence reader or all bulky
artifact checks. Source snapshots/readers remain dependencies.

Exact 33-root set in the final controlled verdict:

```text
CallRidleyFunc
CeresRidley_Hurt
CeresRidley_Init
CeresRidley_Main
HandleCeresRidleyGetawayCutscene
ProcessEnemyInstructions
Ridley_A2F2
Ridley_Func_2
Ridley_Func_70
Ridley_Hurt
Ridley_Instr_1
Ridley_Instr_2
Ridley_Instr_3
Ridley_Instr_4
Ridley_Instr_5
Ridley_Instr_6
Ridley_Instr_7
Ridley_Instr_8
Ridley_Instr_9
Ridley_Instr_10
Ridley_Instr_11
Ridley_Instr_12
Ridley_Instr_13
Ridley_Instr_14
Ridley_Instr_15
Ridley_Instr_16
Ridley_Main
Ridley_Powerbomb
Ridley_Shot
RidleysExplosion_Init
RidleysExplosion_Main
sub_A6DF59
sub_A6DF60
```

The 20 natural roots are `CallRidleyFunc`, `CeresRidley_Init`,
`CeresRidley_Main`, `EnemyInstr_Sleep`, `HandleCeresRidleyGetawayCutscene`,
`ProcessEnemyInstructions`, `Ridley_A2F2`, `Ridley_Func_70`, `Ridley_Instr_1`,
`Ridley_Instr_2`, `Ridley_Instr_5`, `Ridley_Instr_6`, `Ridley_Instr_10`,
`Ridley_Instr_11`, `Ridley_Instr_12`, `Ridley_Instr_13`, `Ridley_Instr_14`,
`Ridley_Instr_15`, `Ridley_Shot` and `sub_A6DF59`.
The native137 set is `EprojInstruction`, `EprojRunAll`, `SpawnEprojWithGfx`
and `SpawnEprojWithRoomGfx`; `EprojInstruction` is a synthetic fixture entry,
not another lexical C definition. The already published reviewed scheduler
boundaries own its native mapping; do not count it as a new routine.

None of these later verdict paths appears in the inspected registry. All
native symbols they substantiate already have a verified observation. A new
`historical-frozen` verified observation could expose the later v2 domain and
retain its exact source/reader references, with zero unique-count change.
Do not label isolated `/tmp/sm-ridley-whole-live-source` or
`ridley-hardware-recovery/source` gates current-checkout or integrated.
The natural report separately records only 574 of 670 fixture calls admitted
by public original-ROM selection; 92 direct fixture-only states and four
getaway boundaries refuse. Broadening the description to all natural calls
as accepted public live roots would be wrong.

The `/tmp/sm-ridley-whole-live-probe-38000-r7` handoff reports matching four
subjects and measured material Ceres writes, but explicitly calls it a
diagnostic. Its full 129,712 gate was not launched, real boss/explosion inputs
are absent, and seven of eighteen supported opcode counters remain zero.
Do not publish whole-Ridley `live-exercised` acceptance from this diagnostic.
Retain it as bounded candidate evidence/limitations. Original-ROM parity and
credits progression remain unproved.

## Controller diagnostic: latest frontier belongs in status, not gameplay totals

The [stopped worker handoff](../../ceres-continuation/controller-handoff-20261013.md)
and [submission receipt](../../ceres-continuation/controller-submission-evidence.toml)
record commit `223f8a1b0c7382817b1290d13c7a41f4e722d5e1` submitted with
integration pending. Its clean worktree remains intentionally retained.

- [2,240-frame verdict](../../../out/port/ceres-continuation/controller/1791873256094852042-256891/verdict.toml):
  1,736 matching causal completed updates, changed causal RNG detected,
  46.710140354 seconds, status `nonaccepting`, `accepted = false`; SHA-256
  `79c00cb7bb32a120f981b1c14f99b639e773d88f5f544bdb41e964409ce7a916`.
- [Final 8,600-frame diagnostic](../../../out/port/ceres-continuation/controller/1791874177761738742-271877/verdict.toml):
  7,740 matching updates through original frame 8,317/event 63,264; first
  divergence ordinal 7,741 at frame 8,508/event 63,799, reason
  `preceding controller poll count differs`. Status `error`, `accepted = false`;
  SHA-256 `051a7195bf0ed3461eae81a9789f052a01f5c616d0d7ed716a37848c2a59d07e`.
  The checkpoint reports 7,960 C versus 7,959 native polls. The retained
  elevator-shaft checkpoint is unmatched. No scheduler repair was made.

The new compact profile omits APU handshake/read timing, primitive fixtures,
raw poll-per-host-step/omitted-wait controls and most full WRAM captures. The
old 1,097-update/frame-1,600 note in ticket 78 remains an earlier frontier;
the later partial diagnostic should be visible as the current research state.
It does not prove ordinary-reset Ceres escape sequence 32 → 33 → 34,
original-ROM parity, credits, or any new gameplay routine stage. Use a status
note with original evidence links rather than inventing a translated/verified
gameplay scope for controller infrastructure.

## Publication boundary and limitations

The safe minimum reporting correction is the independently reviewed
room-display translated observation, the five-root shared-host verified
observation, a fresh declaration catalog, and a current status note separating
merged host work, submitted controller infrastructure and isolated Ridley
candidate evidence. All earlier snapshots/receipts are observation-time
history; append newer reporting records rather than changing their bytes.
The independent `translating` count is not an active-worker or remaining-work
count: it includes retained Ridley assignments plus the old shared-host
assignment. Workers can be stopped/submitted while those historical stages
remain visible.

Missing stage evidence is not missing source implementation. This audit did
not revalidate the entire current checkout, every historical fixture family,
compressed replay contents or a new playable game path. The previous
historical checks and exact frozen readers/sources remain the owners of those
claims. Source at observation and receipt freshness must stay distinct from
current-source acceptance after future edits.

Cleanup for this audit: no run, build tree, executable, worktree, browser
profile or disposable artifact was created. No cleanup target was needed;
all original files were read without modification, and the only new artifact
is this Markdown findings file.
