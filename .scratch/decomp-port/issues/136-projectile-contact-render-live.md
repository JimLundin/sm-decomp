# Enemy projectile contact and rendering live admission

Status: resolved
Blocked by: None
Triage: ready-for-agent

Install the ordinary source-frozen rendering and four contact family contracts
into the existing independent C/mixed replay. Require complete periodic and
terminal WRAM/SRAM, ordered hardware effects and gameplay-progress parity across
two fresh C and two fresh mixed processes. Require nonzero matching exclusive
Rust execution for each credited native owning routine. Report zero-hit routines
as offline-only. Original-ROM parity and whole-game ownership remain separate.

## Comments

Claimed by live_admission subagent. Root owns shared Rust integration and final
source freeze; this ticket waits for its authenticated frozen family verdicts
before starting the live run. Ordinary native ABI remains usable without shared
Rust changes. Root added tested opt-in active-pool captures to the live tool
for direct material output inspection.

Existing ordinary contracts already declare native ABI-compatible signatures:
`DrawLowPriorityEprojs`/`DrawHighPriorityEprojs` (void, no arguments),
`EprojSamusCollDetect`/`EprojProjCollDet` (void, no arguments),
`HandleEprojCollWithSamus` (void, one uint16 argument), and
`HandleEprojCollWithProj` (void, two uint16 arguments). Each seals the actual ROM
and maintained module; `port_family.verify_family_evidence` authenticates all
retained artifacts, complete fixture coverage and every shared Rust dependency
before `port_live.py` builds the mixed subject. The direct rendering fixture
manifest contains synthetic packed adapters and is excluded from live admission.

Native call-site evidence supports a first 9,000-input prefix:
`GameState_8_MainGameplay` in pinned sm82 calls both contact detectors and
`DrawSamusEnemiesAndProjectiles` each normal gameplay frame. The latter calls
both priority drawing passes unconditionally. This reaches the owning traversals
without guessing a late boss route. If measured active projectile coverage is
insufficient, inspect retained captures and extend to 18,000 inputs. A passing
empty traversal does not establish spritemap emission or contact-hit coverage.

Existing authoritative ticket88 diagnostic
`out/port/projectile-inventory/20261011T072333.457801Z/projectiles-decoded.tsv`
records first active pad/platform IDs A387/A395 at frame8,324 and first combat
ID9642 at frame36,895. Thus9,000 can exercise active Ceres drawing but18,000
does not extend into combat. After a successful9,000 ABI/parity run, use a
45,000-input run to measure native contact-hit execution. For that second run
install only the rendering contract and two hit-handler contracts, allowing
the native C detectors to call the measured Rust handler wrappers. The first
run owns detector traversal; the second can independently own actual hits.
The existing soft-reset correction is required near frame43,188 and is already
part of the retained headless comparison host; no replay-state edits are made.

Detector-owned internal Rust calls bypass the separate native handler wrappers.
Therefore detector replacement can exercise handler behavior transitively while
the native handler wrapper counter remains zero; direct handler live ownership
credit requires its own measured wrapper execution. Retained before/after
detector captures can establish active-slot/contact/emission behavior without
inventing direct-handler counters.

Root declared maintained Rust source and tested live harness frozen. All five
ordinary final family verdicts authenticate against current source; exact
family paths/case counts are recorded in
`.scratch/decomp-port/contact-render-live-family-authentication.json`.
The live harness now supports opt-in non-mutating first8 active-pool before/after
captures per installed owning symbol, enabling direct material output evidence
alongside normal first8 unconditional captures. Default capture behavior remains
unchanged.

Started9,000 command (session96750):

```sh
python3 tools/port_live.py --frames 9000 --storage packed --capture-active-eprojs \
  --accepted out/port/decompressor-offline/decomp-acceptance.json \
  --family-verdict out/port/projectile-contact-render-final/rendering/20261011T125652.830322Z/verdict.json \
  --family-verdict out/port/projectile-contact-render-final/detect-samus/20261011T125652.838381Z/verdict.json \
  --family-verdict out/port/projectile-contact-render-final/detect-projectiles/20261011T125652.831698Z/verdict.json \
  --output out/port/projectile-contact-render-live
```

The accepted predecessor pointer is the existing immutable
`out/port/decompressor-offline/decomp-acceptance.json`; its referenced evidence
is checked by `decomp_routine.verify_accepted` before the replay build. Existing
Rust RNG acceptance is separately checked by the runner. No historical gameplay
family verdict is reused as current-source evidence.

## Verification required

Frozen-source ordinary oracle authentication; four complete independent replay
subjects; exact matching nonzero exclusive owning-routine counts; active pool
and material output/contact coverage inspection; zero measured CPU fallback;
scoped ordinary cleanup then deduplication; retained live authentication after
cleanup. Preserve shared `out/evidence` objects, subject `evidence.json` pointers,
sealed source/binary files, the retained `sm-evidence` reader and predecessor
dependencies. Record reclaimed bytes and remaining disk space before resolution.

## Completion evidence

Live9,000 run
`out/port/projectile-contact-render-live/20261011T130027.565661Z/verdict.json`
finished session96750 with exit0, status mixed-C-Rust-integration,
live_exercisedtrue/state_effects_matchtrue. Two fresh C and two fresh mixed
subjects match all190state/effect artifacts. Each mixed subject executes
DrawLowPriorityEprojs, DrawHighPriorityEprojs, EprojSamusCollDetect and
EprojProjCollDet676times exclusively in Rust, with zero measured CPU fallback.
Accepted predecessor replacements remain exclusive23Rust decompressions and
8,783Rust RNG calls per mixed subject.

Active retained low-priority emission observes native A395/A387 slots32/34,
properties3000 and native spritemaps846D/B1BA. First pair advances OAMcursor0to32
and modifies33OAM bytes. High-priority capture has active low-only pool and
preserves OAM. Detector captures set native processing stages11/12, while
Samus flags exclude contacts and zero player count excludes projectile hits;
no hit behavior is claimed by this9,000 run. Packed members are restored using
the retained reader and checked against sealed SHA256 inventory.

Scoped cleanup apply0bytes and deduplicate0snapshot bytes; runner automatically
removed unsealed target/evidence-build and static archives. Post-cleanup
`tools/port_authenticate_live.py` succeeds for all4subjects,190artifacts each,
binary and reader. Evidence:
`.scratch/decomp-port/contact-render-live-9000-authentication.json`.
Remaining disk270,501,183,488bytes. Retain shared out/evidence store, subject
evidence.json pointers, sm-evidence reader, live binary/host/transformed source,
accepted predecessor and frozen offline contracts. No worktree created.

45,000 rendering+two native hit-handler replay
`out/port/projectile-contact-hit-live/20261011T130615.144077Z/verdict.json`
finished session6662 exit0 with mixed-C-Rust-integration/live_exercisedtrue/
state_effects_matchtrue. Two fresh C and two fresh mixed subjects match448
state/effect artifacts. Both render passes execute31,713times exclusively in
Rust per mixed subject; accepted predecessor calls remain exclusive166Rust
decompressions and43,276Rust RNG calls. CPU fallback is zero.

Native C detectors remain installed and both contact-hit handlers have zero
calls in every subject. Therefore neither hit handler receives live ownership
credit; their bounded offline equivalence remains separate. Extending into
actual combat projectile lifetime did not produce the hypothesized collisions.
Pinned ROM definitions9642..9696 have properties5003 without player-shot
contact gate8000, supporting the observed zero shot contacts. No movie state
or input was modified to force a hit.

Mixed0 active-pool16before/after rendering pairs restored in memory through
the retained reader and checked against verdict SHA256 hashes:
`.scratch/decomp-port/contact-hit-live-45000-capture-inspection.json`.
All8low-priority pairs emit material OAM; all8high-priority pairs preserve the
low-only Ceres pool and OAM. High-priority material emission remains unexercised
despite verified ownership of its filtering traversal. The9,000 inspection of
32pairs is retained at
`.scratch/decomp-port/contact-render-live-9000-capture-inspection.json`.

45,000 scoped cleanup apply0bytes and deduplicate0snapshot bytes; runner
automatically removes target/evidence-build and unsealed archives. Post-cleanup
`tools/port_authenticate_live.py` session36177 exit0 authenticates all4subjects,
448artifacts each, retained binary and reader. Evidence:
`.scratch/decomp-port/contact-hit-live-45000-authentication.json`.
Remaining disk269,825,470,464bytes. No run producer/collector remains, no
implementer worktree was created and no disposable local build remains.

Packed storage metrics retain2,114,228,922logical bytes from9,000 with4,996,217
added shared-pack bytes;45,000 retains8,837,580,414logical bytes with11,717,006
added shared-pack bytes. These are storage measurements, not cleanup reclaim.
Preserve shared out/evidence objects, all subject evidence.json pointers, both
retained sm-evidence readers, live binaries/host/transformed sources, frozen
five-family oracle contracts and accepted decompressor/RNG predecessor evidence.
Current ordinary family authentication and independent root integration checks
cover280port tests, strict Clippy and Python harness suites. Original-ROM
parity, actual contact-hit live exercise, high-priority emission and whole-game
Rust ownership remain explicitly open; this ticket resolves measured admission
and its zero-hit classification as specified.
