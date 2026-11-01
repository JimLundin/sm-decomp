# Scenario harness (2026-10-31)

Status: **accepted by Jim, 2026-10-31; admitted under
[REVIEW's lane S conditions](REVIEW.md#scenario-harness-lane-s-jim-2026-10-31);
steps 1 and 2 landed (the standalone recorder is open); step 3 in progress: engine-only
free run against the committed oracle digest lock is the frequent gate (Jim, 2026-10-31,
Decision 4).** It expands
[Validation · Verification strategy](target-architecture/VALIDATION.md#verification-strategy)
into a concrete design. It is PLAN's one runner: the harness grows inside the current
verifier and then replaces it, and never runs beside it as a second framework. Its code
counts as verification, reported separately from production lines.

## Why

Today's verifier is good at one thing: it finds the first frame where the recovered
code differs from the ROM. The problem is how it does that. It compares all of WRAM,
VRAM and SRAM byte for byte, and then adopts the oracle's state on every frame. That
design has three consequences:

1. **It ties the engine to the SNES RAM layout.** Every owner moved into the Session
   still has to re-encode itself into SNES bytes so that the comparison can see it
   (`validate_normalized_projection`). Some places can't be made to agree, and those
   are hidden by 34 normalization ranges whose only stated reason is "existing
   exceptions".
2. **It can't show drift.** Because the oracle's state is adopted every frame, the
   native engine never runs freely. A small difference is counted once and then wiped
   out, so we never see what it grows into.
3. **It's slow to extend.** Scenario data is opaque and maintained by hand:
   - inputs are hex text tied to controller reads;
   - SHAs are typed into TOML by hand;
   - bundles are mapped by `if` chains in `plan_corpus.py`;
   - reports are free text that the tools scrape;
   - each row starts a new Python process, which re-hashes the source tree.

The harness below makes a **scenario** the unit of verification. It compares typed
**observations** instead of memory, and supports both lockstep and free-running
comparison. Full-game routes, boss fights, tricks and glitches are all just scenarios.

## Concepts

| Term | Meaning |
| --- | --- |
| **Tick** | One game frame: one controller read and one advance of the game state. The oracle's lag frames are Ticks with no input poll, so they appear explicitly in the input stream (see below). |
| **Start** | What both backends are primed with: `boot`, `seed:<capture>/<room>` (an original-emulator snapshot) or `checkpoint:<id>`. |
| **Scenario** | Start + input stream + the facts it must demonstrate + the observation domains it checks + run budget. Stored as data, not code. |
| **Backend** | Something that can `open(start)`, `step(input) -> TickOutcome` and `observe(domains) -> Observation`. There are two: the **oracle** (original ROM on the emulated machine) and the **engine** (our code). |
| **Projection** | A function from a backend's own state to an `Observation`. The oracle's projection decodes RAM through `ram_map.rs` and lives with the oracle in `sm`. The engine's projection (`sm_tools`) reads typed Session fields only, never through `SessionRam`, `RawRam` or `CompatibilityRam`. A domain the Session does not own as typed fields is not projected; it stays covered by the whole-RAM comparison, and every report lists each domain's coverage. |
| **Observation** | Typed, per-Tick record, split by domain (below). Hashable; serialisable as a compact binary record. |
| **Fact** | A named thing a scenario must show, e.g. `room:A59F`, `boss:kraid-dead`, `probe:samus.shinespark.start`, `samus.speed>=0x0700 for 60 ticks`. A trick or glitch earns a scenario by naming the fact that shows it worked. |
| **Divergence** | The first (Tick, domain, field) at which the two projections differ, with both values and the last equal Tick. A structured record, not text. |

### Observation domains

Each domain defines its fields' width, signedness, units, wrapping, and the phase in
the Tick at which they are sampled. Equality is exact unless the domain records a
reviewed tolerance.

| Domain | Fields (initial) |
| --- | --- |
| `input` | polled pad word, lag flag, Tick number |
| `samus` | x/y position + subpixel, velocity, pose, movement type, health/ammo, timers (invulnerability, charge, shinespark), equipment |
| `camera` | layer 1/2 scroll, door transition phase |
| `actors` | ordered list: species, slot, x/y, health, AI state words, frozen/invulnerable flags; projectiles and enemy projectiles as separate lists |
| `room` | area, room, door, room-state, PLM list, scrolls, FX type and level |
| `progress` | items collected, events, bosses, map tiles, save slot data |
| `rng` | RNG word and call count, if it can be observed |
| `effects` | ordered sound requests (Cue trace), palette/animation program spawns, earthquake, HDMA program spawns |
| `video` | per-line hash of the rendered line (full pixels only on divergence); optional sampled PPU model state |
| `audio` | Cue trace; optional PCM hash per Tick block |
| `ram` | the legacy whole-RAM comparison, kept for domains that are still RAM-backed (see Migration) |

Adding a domain or field is a normal code change in one place. The field table also
generates the reports, so field names and units always match the code.

## Run modes

1. **Lockstep (fault localisation).** Step both backends; compare; on divergence
   adopt the oracle's state and continue, counting failing Ticks. This is today's
   behaviour, kept because it pinpoints where a bug is.
2. **Free run (drift).** Step both backends independently from the same Start and
   never adopt. Report the first divergence per domain, then keep running to the
   budget and report how far the divergence spreads (which domains, how many Ticks).
   This is the mode that shows drift, and the one that lets a refactor be checked
   without the engine having to stay in the RAM layout.
3. **Determinism.** Run the engine twice from the same Start, optionally from a
   Checkpoint taken mid-run, and with observation on or off. Observations and the
   state hash must be identical. No oracle is needed, so it's cheap enough for every PR.

When a free run diverges, the harness automatically re-runs that stretch in lockstep
from the nearest checkpoint, so the report says both where the run *first* drifted
and which routine is wrong.

In the verifier today (step 2), each side runs alone in its own process
(`--oracle-only` or `--engine-only`), taking one record per Tick: a digest of each
machine stream lockstep compares (WRAM with the normalized ranges masked, SRAM,
VRAM, CGRAM, OAM, PPU registers, the rendered frame, the APU ports) and the typed
observation. `verify_dll.py --corpus R --mode M` runs seeded and boot rows:

- `free-run`: the oracle records a trace; the engine runs against it Tick by Tick.
- `determinism`: the engine records its own trace, then reruns the first half with
  typed observation off, saving a checkpoint, and the second half from it.
- `lock --oracle faithful --lock F` writes the oracle digest lock (REVIEW, lane S
  condition 8); `check-lock --lock F` runs only the engine against it and names the
  differing streams and the last matching 1,000-Tick mark. It then reruns the first
  unmatched 1,000-Tick span live in faithful lockstep: the oracle alone replays to the
  mark and saves a checkpoint, and both sides run the span from it, which names the
  first differing Tick and field (or reports that lockstep masks the drift).

A single-side run takes the row's whole budget (or its inputs, if they end first): the
oracle cannot see probes, so neither side stops on a probe or expectation tail, and
only the engine checks the required hits.

The lock is `scenarios/lock.toml`: every seeded and boot corpus row and every room
of both routes, keyed on the oracle's start (seed payload digest, or `boot`) and the
input digest and budget. `.github/scripts/full_route.py run JOB` passes its
arguments to the corpus run, so the routes record and check the same way.
`.github/scripts/lock_gate.py` lists the scenarios that stay on patched lockstep
(their `docs/LEGACY.md` entries) and fails on a stale lock: an entry whose start,
inputs or budget changed, an entry for no scenario, or a scenario neither locked
nor listed. The `lock-gate.yml` workflow runs it, then the engine over every
locked scenario on each pull request and push; its console carries ids, stream and
field names, ticks and hashes only.

On 2026-11-01 the lock holds 619 of the 660 lockable scenarios: 99 of the 132
seeded and boot corpus rows and 520 of the routes' 528 rooms. The other 41 stay on
lockstep, listed in LEGACY.md. The 23 corpus rows that start from a checkpoint are
not lockable yet: they need an oracle-side prefix first.

## Turning divergences into scenarios

`scenario cut <run> --at <tick> --window <n>` writes a new scenario. Its Start is a
checkpoint a few Ticks before the divergence and its inputs are the next *n* Ticks.
Its fact is the divergent field's oracle value. The checkpoint stays in the encrypted
bundle; only metadata is committed. So anyone who sees drift in a playthrough, or
wants to pin a trick, records a movie and gets a small permanent scenario out of it.

Authoring stays text-first:

- **Recorded movies** (LSMV/BK2) are converted once into the input stream.
- **Authored inputs** use the existing `authored_inputs.py` language (holds, presses,
  repeats, named moves, parameter sweeps). It is a good fit for tricks: for example,
  "mockball at every entry speed from 0x0400 to 0x0700" becomes a sweep that produces
  one scenario per value.
- `scenario lock` computes and writes every hash and bundle mapping, so nobody edits
  a SHA by hand.

## Formats

No JSON (repo rule). ROM-derived bytes never leave the encrypted bundles. Committed
files and CI logs carry ticks, domain and field names and hashes, never values.

- **Placement:** the scenario model, observation types, the engine projection, the
  comparison and the report are `crates/sm_tools`. It forbids unsafe and depends only
  on `sm_data`, `sm_game` and `sm_runtime`; CI rejects any target package depending on
  it, and it depending on `sm`. The oracle projection stays in `sm`.
- **Scenario manifest:** TOML, one table per scenario, grouped by file
  (`scenarios/route-100.toml`, `scenarios/tricks.toml`, `scenarios/bosses.toml`, …).
  A committed scenario holds only its `id`, its `start` as a reference by digest, its
  `facts` and `domains`, plus `ticks`, `modes`, `tags` (e.g. `trick`, `glitch`,
  `route`, `boss:ridley`) and `pending`. The digests live in a generated
  `scenarios/lock.toml`.
- **Input stream:** the current hex text, with lag Ticks written explicitly as `-`
  lines. Explicit lag Ticks mean a hardware-correct timing fix no longer shifts the
  movie by silently consuming inputs. That shifting is exactly what made the 8
  corpus rows drift this week. Input streams, seeds and checkpoints stay in the
  encrypted ci-data assets (PLAN: never commit movies or derived data); `scenario cut`
  and `scenario lock` update them through that route.
- **Observation trace** (only on demand or on divergence): a binary file with a header
  holding the format version, then one length-prefixed record per Tick: input frame,
  Tick, machine stream digests and the encoded typed observation
  (`sm_tools::trace`). A `trace show` command will render it as text or CSV. Traces
  are artifacts and are never committed.
- **Oracle digest lock:** per scenario, its oracle-side start and input digests, the
  format version, the Tick count, one running digest per stream and the combined
  machine digest every 1,000 Ticks: under a kilobyte, revealing nothing.
- **Divergence report:** a TOML artifact per scenario (tick, domain, field, oracle
  value, engine value, last equal tick, coverage per domain, agreement with the RAM
  comparison), written to a path the run names (`verify_dll.py --divergence-report`,
  or `--divergence-report-directory` for a corpus). It is never committed. CI prints
  only the tick, domain and field names.

## Performance

The target is the full corpus plus both routes, in both modes, at least as fast as
today's corpus run alone.

- **One Rust binary** (`sm_tools scenario run`) runs many scenarios in one process,
  spread across threads. There is no Python in the run path and no re-hash per row;
  the binary checks the source stamp once at start.
- **Cheap comparison:** each domain is hashed per Tick, and a field-by-field diff only
  runs when the hashes differ. Video compares line hashes. Pixels, PPU model checks
  and PCM are sampled every N Ticks, the way SM_PPU_MODEL_CHECK works today.
- **Free run needs no snapshot or restore per Tick.** That removes today's 3 snapshots
  and 2–3 restores per frame. Lockstep pays that cost only inside a localisation
  window.
- **Shared prefixes:** checkpoints are cached by (start, input-prefix hash), as the
  corpus already does, so scenarios with a common route prefix don't replay it.
- **Sharding:** a CI job takes a slice of scenarios weighted by recorded duration.
  Durations are recorded in the run's artifact so that the next plan balances on them.

## Coverage

Coverage counts three things, and the report shows all three:

1. **Code:** probe hits (`crate::probe::hit`) per scenario, as `require_hit` does
   today, collected automatically from every run.
2. **Content:** the Gate 5 inventory (rooms, enemies, bosses, items, events, sequences,
   effects, sound programs) against the scenarios whose observations touched each.
3. **Techniques:** the Gate 2 list (movement, weapons, speed and shinespark, grapple,
   slopes, liquids, damage) plus named tricks and glitches, each linked to the
   scenarios whose facts show it.

A missing entry in any of these is a gap that should become a scenario.

## Migration

Each step keeps the existing comparison path working until the new one covers the same
ground (VALIDATION.md).

1. **Observations inside the current verifier.**
   - Define the observation structs, field tables and per-domain hashes in `sm_tools`,
     the oracle projection in `sm` and the typed engine projection in `sm_tools`.
   - Run them next to the unchanged RAM comparison inside the verifier DLL, in
     lockstep, comparing each projected domain by hash and diffing fields only when
     the hashes differ. The RAM comparison stays authoritative; a Tick where only one
     of the two fails is reported, and an observation-only failure is its own category.
   - While the Session is not retained between frames (today's lockstep), the engine's
     typed state is each frame lease's Session: the projection reads it as the lease
     releases it, and the Tick's last lease is compared. A retained Session is read
     directly.
   - No new `sm_game_ram` sites; at most about 2,000 lines, counted as verification.

   This step alone gives structured divergence reports. *Exit:* chain rooms 1, 6,
   100, 200 and 320 pass, and a one-field mutant is reported at the right tick, domain
   and field.
2. **Free-run mode** in the same DLL, for routes and corpus rows. First nightly, then
   on every PR once it is stable.
3. **`scenario` manifests** generated from `movies/segments.toml` and the route CSVs,
   so the formats change without losing a row. Add `scenario cut` and `scenario lock`.
4. **The `sm_tools` binary** takes over orchestration from `verify_dll.py` and
   `full_route.py`. The DLL build stays, so the oracle remains reachable while it is
   still linked.
5. **Retire RAM comparison one domain at a time.** When a domain's state is owned by
   the Session and its observation projection covers it, the same change drops that
   domain's bytes from the `ram` domain and deletes what it supersedes: the matching
   `normalize_snapshot_ram` ranges, their `LEGACY(verifier)` rows and the checks it
   replaces. The RAM ratchet then gains a sibling measure: *domains compared by
   projection versus by RAM*. By Phase 3's exit, verification is no larger than at
   `e0b47fe` (57,802 lines).

## Decisions (Jim, 2026-10-31)

1. The reviewer updates PLAN and REVIEW to allow this harness in place of the
   "reuse the existing runner" rule (done: PLAN's "One runner", REVIEW's lane S
   conditions).
2. No tolerances: every divergence is fixed or listed in [LEGACY.md](LEGACY.md), as now.
3. Step 1 starts now as its own lane, in parallel with the moves into the crates.
4. **Free run against the lock is the frequent gate.** Engine-only free run against
   the committed oracle digest lock runs on every pull request and push, even though
   on the five step-1 rooms it measured only about 1.4x faster than lockstep. This
   overrides the 2x adoption bar in REVIEW's lane S condition 8. A locked scenario
   needs only its digests, runs deterministically, and costs no oracle frames, and
   scenarios will keep being added. Engine optimization is expected to widen the gap.
   Lockstep stays: it localises a fault and confirms a failure (a lock mismatch reruns
   its failing 1,000-Tick span in lockstep), and the corpus and route lockstep jobs keep
   running. Which events lockstep runs on is decided later.
