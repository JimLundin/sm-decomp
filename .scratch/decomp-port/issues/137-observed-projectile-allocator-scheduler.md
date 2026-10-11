# Observed enemy projectile allocator and scheduler ownership

Status: claimed
Blocked by: 32, 37, 80, 81, 88
Triage: ready-for-agent

Close native allocation and scheduling for the actual nine-ID projectile
trajectory in ticket88, including combat beginning at input/frame36,895.
Translate missing Ridley fireball initializer8693CA and Ceres elevator
initializers86A2EE/86A31B with common86A301 setup; retain complete highest-free
allocation behavior. Add the missing native86950C no-op required by stationary
fireball explosion definitions. Integrate existing collision, split/repeat,
host-aware impact and elevator callbacks through native allocation/scheduling
ownership, including live support selection before any mutation.

## Authoritative execution and missing source

Ticket88 full129,712-input C diagnostic:
`out/port/projectile-inventory/20261011T072333.457801Z/verdict.json` and its sealed
`projectiles-decoded.tsv`. Read-only SHA256 verification on2026-10-11 confirmed
the decoded TSV, transformed`sm_86.c` and diagnostic binary against the verdict
inventory. This is execution evidence, with no Rust ownership/parity claim.

| Kind | Native address/ID | Hits | First frame | Last frame |
| --- | --- | ---: | ---: | ---: |
| Initializer | 8693CA Ridley fireball | 16 | 36895 | 70802 |
| Initializer | 86A2EE Ceres elevator pad | 4 | 8323 | 97830 |
| Initializer | 86A31B Ceres elevator platform | 4 | 8323 | 97830 |
| Active ID | 9642 fireball | 617 | 36895 | 70841 |
| Active ID | 9650 explosion | 50 | 36931 | 37069 |
| Active ID | 965E explosion | 50 | 70716 | 70851 |
| Active ID | 966C split | 208 | 36935 | 37089 |
| Active ID | 967A split | 78 | 36935 | 37074 |
| Active ID | 9688 split | 156 | 70720 | 70862 |
| Active ID | 9696 split | 182 | 70720 | 70872 |
| Active ID | A387 elevator pad | 528 | 8324 | 97962 |
| Active ID | A395 elevator platform | 524 | 8324 | 97961 |

Native definitions from actual`rom/sm.sfc` bank86:

- 9642: init93CA, pre940E, script9552, radius0606, properties5003.
- 9650/965E: init947F, pre950C, scripts95A0/95D3, radius0606, properties5003.
- 966C/967A: init94C8/94DC, pre950D, script9606.
- 9688/9696: init94A0/94B4, pre9522, script9606.
- A387: initA2EE, preA328, scriptA28D, radius0101, properties3000.
- A395: initA31B, preA364, scriptA299, radius0101, properties3000.

Current maintained`enemy_projectiles.rs` has no93CA/A2EE/A31B
validation/initializer dispatch; ordinary supported allocation refuses these
observed definitions. Current`enemy_projectile_instructions.rs` has no950C
preinstruction case, although native`CallEprojPreInstr` treats fnnullsub_85 as
an unconditional no-op. These gaps prevent the complete observed lifecycle
from executing solely through the maintained allocator/scheduler.

## Native source and existing dependencies

Pinned`deps/snesrev-sm/src/sm_86.c`:

- SpawnEprojInner868027, SpawnEprojWithRoomGfx868097,
  EprojRunAll868104 and EprojRunOne868125: lines113–180.
- EprojInit_9642_RidleysFireball8693CA: line1206. Copy
  eproj_unk1995(WRAM1995) to F; choose X±25 by parameter nonzero, Y−43 from
  enemy_data[0], gfx2560, X/Y velocities from Ram7800_Default[0].var19/var1A
  (WRAM7832/7834), and area-selected bank86 properties at9408/940A/940C.
  This uses enemy slot0 regardless of current enemy index; preserve that alias.
- Existing SetAreaDependentEprojPropertiesEx86932F and wrapper869402:
  lines1160/1220; area2 and5 select entries1/2, others entry0. Reuse the owned
  directional initializer behavior rather than inventing a second property rule.
- Ceres init86A2EE/common86A301/86A31B: lines1898/1905/1916. Pad sets
  Y=SamusY+28/E=60; platform setsY=97. Common resets X/Y fractions and
  velocities, clears gfx and copies SamusX. Preserve other fields.
- Existing impact86940E allocates stationary947F explosion via explicit
  eproj_spawn_pt and queues2B; existing95BA/95ED/9620 spawn and repeat
  directional splits; existing950D/9522 perform terrain collision.
- Native950C is a no-op dispatcher leaf (`funcs.h:4743`,
  `sm_86.c:5065`), required even though sleeping scripts can clear it before
  later inventory preinstruction observations.

The actual Ridley caller`sm_a6.c:4498` Ridley_E90C(A6E90C) sets WRAM1995 then
calls SpawnEprojWithRoomGfx(9642, enemy0.ridley_var10); its enemy AI remains C
and is outside this ticket. Main gameplay`sm_82.c:863` calls EprojRunAll before
PLM, contacts and rendering. No translation of Ridley AI is a prerequisite.

Owned dependencies: highest-free allocator(ticket32), complete directional
split/repeat chain(ticket37), explicit eight-word ProjectileHost(ticket80),
host-aware940E collision/explosion scheduling(ticket81), and native terrain
collision/audio/elevator callbacks. Expanded bomb/ring/tube/sparks ownership
already exists and must continue working through shared dispatch.

## Required implementation and verification

1. Translate93CA/A2EE/A31B and950C through the maintained native allocation,
   initializer and scheduler entry points. Preserve saturation behavior:
   allocation writes only init parameter before returning on a full pool and
   does not read definition/initializer ROM or host state. Preserve every
   native partial reset, source/destination alias and descending slot order.
2. Provide explicit live support selection for complete allocation and
   scheduler calls, based on actual definition/init/script/callback support.
   Determine support before WRAM, parameter, RNG, sound or host mutations.
   Unknown definitions/unsupported whole scheduler frames may select native C
   before mutation; accepted Rust calls either succeed or terminate/report the
   error. Never retry C after a Rust error or partially mutated state. Validate
   canonical slots and required ROM/callback extents before accepting a call.
3. Bridge actual external eproj_spawn_pt/rect/r22/varE24 globals into the
   existing ProjectileHost for synchronous native live allocator/scheduler
   wrappers, including same-bank callers. Preserve and publish all eight
   words, not guessed WRAM aliases or discarded copies. Current ordinary
   `port_live.generate_family_abi` calls dispatch_rom and has no projectile-host
   bridge; host-aware admission requires its explicit tested ABI selection.
   Host state changed by940E must reach later allocation and subsequent frames.
4. Compare compiled actual C against full WRAM, raw returns and all eight host
   words for native allocation, direct initializer, actual-ROM scheduled frames
   and a chained fireball→impact→explosion→split→repeat→deletion lifecycle.
   Cover both horizontal/vertical explosion branches, F suppression,
   all free-slot choices and saturation, spawn reuse, lower/higher newly
   allocated slots, fractional/coordinate/speed boundaries, area2/5/default
   properties, enemy0 versus current-enemy aliases, all Ceres timer/height
   transitions, and disabled scheduler behavior. Retain native scripts/maps;
   label controlled chained inputs separately from live replay.
5. Add meaningful independent semantic/ABI tests: full-pool allocation with
   invalid definition/absent host reads only parameter, Ridley enemy0 alias
   despite a different current enemy, partial resets and nonzero parameter
   orientation, hostpoint persistence through impact/reuse,950C scheduling,
   Ceres common setup, support refusal with byte-identical WRAM/host and no
   C retry on Rust error. Verify current port tests, formatting, strict Clippy
   and relevant host/live harness controls after shared source freezes.
6. Refresh final actual-C seals and run four independent C/mixed processes
   through the combat interval (at least45,000 inputs; extend to full129,712
   for both split orientations). Require complete periodic/terminal state and
   ordered effects parity, zero CPU fallback, and nonzero exclusive Rust
   native allocation/scheduler counts with active9642 lifecycle captures.
   Report bounded support/refusal counts explicitly; do not infer ownership
   from empty traversals or prior offline seals.
7. Complete scoped run cleanup/deduplication and retained post-cleanup offline
   and live authentication before resolution. Preserve source snapshots,
   pinned ROM/scripts, chained inputs, eight-word host sidecars and shared
   packed-evidence dependencies.

## Comments

Selected read-only while ticket136 live runs are source-frozen. No gameplay or
harness source was changed and no verification run was launched for this
selection. All listed blocker tickets are resolved, so the ticket is ready.


Claimed by root integration owner on 2026-10-11. Parallel ownership: native
initializer integration, bounded ownership/support module, and C/Rust live host
bridge. Root owns final fixture orchestration, source freeze, evidence and cleanup.

Integration update on 2026-10-11: native initializers/common setup and950C are
implemented. Immutable support selection recognizes the exact nine native
definitions and script continuations, preflights terrain extents/extensions and
sound cursors, and permits saturation/disabled no-host short circuits. The
live bridge carries actual external eight-word globals transactionally and
terminates accepted errors with exit4, without C retry.

Preliminary actual-C evidence (not final current-source seals):
- `out/port/observed-projectile-initializers-preliminary/20261011T132206.715644Z/verdict.json`:3,825cases/sixroutines.
- `out/port/observed-projectile-lifecycle-preliminary/20261011T132517.239553Z/verdict.json`:1,840cases/threeroutines, native scripts, both impact branches,
  peak8splits, deletion and host-point reuse. Each chained step inherits matching
  previous fullWRAM and all8hostwords; controlled inputs are explicitly separate
  from live replay.

Final Rust checks currently pass293tests and strict all-target Clippy. New Rust
files pass rustfmt; workspace format check retains preexisting projectile/PLM/room
formatting discrepancies. Relevant Python harness run passes36tests with1opt-in
skip; batch controls pass5tests. Final actual-C seals and live runs are pending.

Preliminary cleanup completed with `tools/port_cleanup.py --root <each above
preliminary root> --apply` then `--deduplicate --apply`:0ordinary bytes,
783,417,344initializer snapshot bytes and555,614,208lifecycle snapshot bytes
reclaimed. Sealed snapshots/binaries/verdicts remain.

Final source-frozen actual-C seals:
- `out/port/observed-projectile-final/initializers/20261011T132755.906611Z/verdict.json`:3,879cases.
- `out/port/observed-projectile-final/ownership/20261011T132800.186971Z/verdict.json`:1,840chained steps.
- `out/port/observed-projectile-final/boundaries/20261011T133554.778757Z/verdict.json`:577frame coordinate/speed/fraction boundaries.
- Refreshed rendering/contact detector contracts in
  `out/port/observed-projectile-final/batch.json`:5,798cases.

All12,094actual-C cases authenticate after cleanup against frozen current Rust
and harness sources. `observed-projectile-offline-authentication.json` retains
the six family pointers. Native chain inspection is recorded separately in
`observed-projectile-lifecycle-inspection.json`. Latest batch includes dedicated
`projectile`/`offline-projectile` kinds and `observed-projectiles` profile; no
standalone leaf hit count substitutes for native scheduler ownership.

Live45000 started with packed four-process native allocator/scheduler plus
rendering/contact detectors, run20261011T133252.262548Z. An earlier command used
incorrect manually copied detector timestamps and was rejected before build;
its retained run20261011T133244.878534Z cleaned with0bytes reclaimed. Correct
command is produced from the batch's exact stored verdict paths. Full129712
run20261011T133721.233571Z is building independently; both use the same frozen
sources, exact ROM/movie and packed evidence store. Live admission pending.

Read-only actual live-C capture preflight:44/44native allocation/scheduler calls
admitted and matched complete WRAM/all8hostwords, zero refused/errors. Includes
first8active/first89642native calls and first2scheduled frames for each of six
IDs observed by45000. Summary/member/source hashes retained in
`observed-projectile-live-capture-preflight.json`. Remaining three IDs and
four-process route parity require full129712 replay. Extracted22,030,273bytes
and temporary compiled audit reader removed; sealed packed observations retained.

Final cleanup so far: batch automatic dedup1,370,357,760bytes; explicit final
initializer dedup795,213,824bytes; boundary dedup151,519,232bytes; final whole
batch dedup524,288bytes; ordinary cleanup0bytes. Disposable Rust check/clippy
builds524,242,430bytes removed after utility reported0/0 (outside recognized
run format), verification logs retained. `observed-projectile-build-cleanup.json`
records these disposable outputs and remaining free space. No implementer
worktrees were created; unrelated pipeline/review/trace worktrees preserved.

Full C baseline reached129712 with allnine observed IDs and zeroCPUfallback.
Actual second-orientation captures exposed6supportrefusals solely from an
overconservative nonwrapping collision rectangle (x41 with envelope43, then
x37/32). Direct Rust dispatcher already matched all6fullWRAM/8host outputs.
Correction inspects wrapped row/column origins plus contiguous native scan
increments while retaining extent/cycle validation. Eight semantic ownership
tests and strict Clippy pass; all6captured calls now admit. Original sourcehash,
refusal and C/Rust comparisons preserved in
`observed-projectile-full-capture-preflight.json`. Previous current-source
seals are superseded; corrected final batch regenerating all broad dependencies.

Partial45000runner exited143 before mixed gameplay while waiting storelock.
Root recovered diagnostic rejected verdict at
`out/port/observed-projectile-live-45000/20261011T133252.262548Z/verdict.json`;
cleanup reclaimed47,444,398unsealed targetbytes,0deduplicate. Both packed C
subject pointers retained/authenticated separately; no four-process claim.

Corrected final batch completed12,094actual-Ccases, six families, all current
source checks and retained post-cleanup authentication pass. Batch dedup reclaimed
2,241,724,416bytes. Corrected tests294/strictClippy/14focusedABIbatchcontrols pass.
Old complete129712 C baseline packed/authenticated; root stopped superseded
runner after durable pointer. Full capture member hashes corroborate its packed
inventory; removed2,076,419,097authenticated stagingbytes. Diagnostic rejected
verdict retained, separate from ownership admission. Fresh corrected45000 and
129712 four-process live runs use gzip to progress independently of shared-store
packing; native allocator/scheduler plus previously accepted RNG/decompressor
are installed, while rendering/contact family behavior remains evidenced in
the corrected offline batch and previous live tickets.

Corrected45,000 live run
`out/port/observed-projectile-corrected-live-45000/20261011T135145.693993Z/verdict.json`
passes mixed-C-Rust-integration. Four fresh processes match607state/effect
artifacts; each mixed process executes8native allocations and31,305scheduler
calls exclusively Rust,0supportrefusals/0CPUfallback,296activeframes/1039642
frames. Six observed IDs each have nonzero exclusive Rust scheduler counts and
fullWRAM/8host captures; later3IDs are not observed in this interval. Accepted
predecessors execute166Rustdecompressions/43,276RustRNG calls per mixedsubject.
Scoped cleanup/authentication pending record; full129712stillrunning.

User requested a repo commit and handoff checkpoint. Resume instructions are
`../checkpoint-137.md`; ticket137 remainsclaimed while the corrected full
129712four-process run continues. Both Cbaselines finished; firstmixedsubject
is running.45kpostcleanupauthentication passed607artifacts/all4subjects,
377,696,038snapshotbytes reclaimed. Checkpoint is an intermediate verified
implementation commit, not ticketresolution or whole-game acceptance.


## Completion evidence

Intermediate implementation checkpoint: `91c513c` on `decomp-port-integration`.
Ticket remains claimed. Refreshed handoff: `../checkpoint-137.md`.

Read-only acceptance audit found a remaining native allocation gap: live and
observed offline native-entry contracts cover `SpawnEprojWithRoomGfx`, while
Ceres calls three-argument `SpawnEprojWithGfx` from `sm_90.c`. Current live Rust
allocation counts therefore cover Ridley only; all-nine scheduler admission does
not establish exclusive Ceres allocation. After the current frozen replay
finishes, extend support/host ABI/same-bank routing and actual-C native fixtures,
add per-ID allocation captures/counters, and verify four pad plus four platform
allocations exclusively Rust in a fresh four-process full-route replay.

Current full run: `out/port/observed-projectile-corrected-live-full/20261011T135146.556016Z`.
Both C baselines completed 129,712 inputs, matching counters/progress and zero
CPU fallback. Mixed 0 passed 89,000 inputs with zero fallback; mixed 1 pending.
Later 965E/9688/9696 captures match full WRAM/all eight host words across both C
subjects and mixed 0, including point (22,120) and vertical splits at
(22,106)/(22,134). Partial inspection record:
`../observed-projectile-live-full-inspection.json`.

Completed-run cleanup/authentication remains recorded in the existing cleanup
and authentication JSON records. Full-run cleanup is pending because processes
still write the run; the handoff contains exact scoped cleanup/dedup/authentication
commands. This refresh created no disposable build or capture extraction; the
inspection script was removed, reclaiming 4,316 bytes. Remaining free space:
257,736,871,936 bytes. Required sealed artifacts, shared store dependencies and
unrelated worktrees remain retained. No full-route or complete allocation
acceptance is claimed yet.
