# Mutation sample (#242): why the runs failed, and a nightly design

Research lane `research/mutations`, base `3b37340`. No CI runner was used: the
findings below come from the two failed runs' job logs and from local runs on
Linux without the ROM.

## 1. Why the scheduled `Mutations` runs failed

| Run | Date | Head | Job | Step that failed | Time |
| --- | --- | --- | --- | --- | --- |
| 41301626024 | 2026-11-01 | `da5a55483` | 124833573764 | Kill every detached-core mutant | 30 min |
| 41446537055 | 2026-11-02 | `558131e72` | 125280931334 | Kill every detached-core mutant | 27 min |

Both runs failed for the same reason, and **the plumbing was broken.
Neither run had a surviving mutant or a timeout.**

- `player_detached/mutations.py` killed the baseline and the first 23 mutants:
  4 gameplay, 3 named gameplay, 11 visual and 5 sprite. It then raised on
  `sprite_capture_content_before_diagnostic`:
  `COMPILE ERROR in sm_game; not a killed mutant`.
  The mutant's replacement text names `Source::Loaded` and `record.native_word`
  in `sprite_objects.rs`, which no longer imports `visual_host::{Source, Record}`
  (`E0433`, `E0599`).
- The anchor still existed, so `--check-anchors` in Quality passed. That check
  can't tell whether a mutant still compiles.
- The script stops at the first error. The remaining 19 detached mutants
  (sprite, flags, actor contracts and collision contracts) never ran, and the
  workflow's next step, the 52 Session mutants, never started. So the last
  evidence for those suites is older than 2026-11-01.
- Windows GNU with `CARGO_BUILD_JOBS: 1` took 27 to 30 minutes to reach mutant
  24 of 42 in the detached suite. Each mutant rebuilds `sm_game` with
  `rustc -O` and no incremental cache.

**A second break since then.** On today's main, the detached suite fails even
before its first mutant. `shared.rs` includes `#[path = "../../../../crates/sm_host/src/ram_map.rs"]`
(since `d5d52ef`, sm_host owns `ram_map.rs`). The script copies `sm/src` to
`<tmp>/<name>/src`, so in that copy the path resolves to `<tmp>/crates/sm_host`,
which doesn't exist. The baseline fails with `couldn't find file`.

Both are fixed on this branch in `sm/src/verification/player_detached/mutations.py`:

- the sprite mutant uses fully qualified paths (`crate::visual_host::Content::table`,
  `crate::visual_host::Record::native_word`);
- the run copies `crates/sm_host/src` to `<tmp>/crates/sm_host/src` once.

## 2. Mutants that run without the ROM: local results

Linux, 4 cores, `LIBRARY_PATH=/home/user/sm-native/lib`. The scripts refuse to
run outside Actions, so they were run here with `GITHUB_ACTIONS=true`.

### Detached cores (`player_detached/mutations.py`)

After both fixes, the baseline compiles and passes, and every mutant reached
before the time budget ran out was **killed by its named assertion**:
`ignore_positive_beam_radius`, `skip_collision_retirement`,
`ignore_plant_position`, `ignore_new_beam`, `award_to_private_inventory_copy`,
`discard_shared_shot_mark`, `stale_room_selected_after_spawn`,
`omit_fractional_heat` and `cache_heat_equipment_before_sound`. That's 9 of
42, with 0 survivors so far. Each mutant costs about 2 to 3 minutes here (a
cold `rustc -O` of `sm_game`), so the whole suite takes about 1.5 to 2 hours on
this machine. The fixed `sprite_capture_content_before_diagnostic` compiles as
part of `sm_mutants build`'s check style, but it wasn't reached in this run.
Running the rest of the suite is listed under Open.

### Session RAM mappings (`session_mutations.py`, 52 mutants)

These weren't run here. Each mutant rebuilds the `sm_game` test binary with
`rustc -O` and no cache. At the detached suite's local rate, 52 mutants would
take well over an hour on this machine. #242 notes that these mutants expire
along with the RAM encoder. The new tool's two decoder mutants (below) cover
their replacement.

### New catalogue (`sm_mutants`, Rust)

`crates/sm_tools/src/bin/sm_mutants/` holds a typed catalogue (`catalogue.rs`:
`Mutant { name, target, edits, proof }`, with `Target` naming the decoder,
Session compare, the lock or an engine stream) and its interpreter (`main.rs`).

- `sm_mutants anchors` checks that every anchor occurs exactly once.
- `sm_mutants build [SELECT]` applies each mutant and compiles it. This is the
  check that would have caught the 2026-11-01 break.
- `sm_mutants run [SELECT]` proves each mutant. `SELECT` is `--slot I/N`,
  `--slot today/N`, `--only NAME` or `--rom-free`.
- Each verdict is **killed**, **survived** or **broken**. A broken mutant
  (stale anchor, compile error, failing baseline, a test that didn't run) is
  plumbing and never counts as a kill. The run continues past it. Exit status
  is 0 when all are killed, 1 for a survivor and 2 for a broken mutant.
- Mutants are applied in place and restored by a drop guard. Cargo runs with
  `--cap-lints=warn` for both the baseline and the mutant. That way a mutant
  that leaves an item unused is judged by its tests rather than by
  `deny(warnings)`, which is how the Python scripts handle it with `-A unused`.
  Incremental builds make a unit mutant cost seconds, not minutes.

Local result, `sm_mutants run --rom-free`: **12 of 12 killed, 0 survived,
0 broken**. `sm_mutants build` compiles all 16, including the 4 replay-level
mutants, in 64 s.

| Mutant | Target | Killed by |
| --- | --- | --- |
| `decoder_minimap_reads_even_bytes` | decoder | `state_layout::session_round_trips_random_ram` |
| `decoder_countdown_repeats_first_digit` | decoder | same |
| `compare_skips_the_last_field` | Session compare | `every_field_is_compared` (**new**; it survived every existing test) |
| `compare_reports_no_field` | Session compare | `uncarried_fields_take_the_oracles_value`, `a_changed_field_changes_only_its_group` |
| `compare_path_stops_at_the_root` | Session compare | `the_path_names_fields_and_indexes_but_no_values` |
| `compare_counts_a_divergent_tick_as_agreement` | Session compare | `agreement_counts_each_side` |
| `streams_put_a_field_in_the_wrong_group` | Session streams | `a_changed_field_changes_only_its_group` |
| `streams_keep_a_changed_fields_old_digest` | Session streams | same |
| `lock_fold_drops_a_streams_tick` (`digest & !1`) | lock fold | `one_changed_tick_...` with a **new golden value**; it **survived** without one |
| `lock_combined_skips_a_machine_stream` | lock combined digest | `one_changed_tick_...` |
| `lock_writes_a_partial_stream` | lock entry | `a_stream_some_tick_lacks_has_no_running_digest` |
| `digest_drops_the_rotation` | digest | `digest::tests::the_digest_is_the_one_the_lock_recorded` (**new**; no test pinned the digest) |

**Survivors found and fixed.** Three mutants survived the existing tests:

- **Last field never compared.** `differing` could skip the last listed field
  (`wavy_samus`) without any test failing. The new test
  `session_compare::tests::every_field_is_compared` catches it.
- **Lossy fold.** The running fold could lose bit 0 of every Tick digest
  without any test failing. The trace test now asserts the exact running digest.
- **Digest function changed.** The digest function could change and pass every
  ROM-free test, because only the committed locks (ROM rows) depended on its
  exact values. The new `sm_data` test pins three values.

Run the tool from a copy of the checkout, never next to a running
`player_detached/mutations.py`. That script copies `crates/` for each of its
own mutants, and it would pick up an applied `sm_mutants` edit.

## 3. Nightly design

### What runs each night

Three tiers, rotated by day (`--slot today/N`), in one job:

1. **Every night, ROM-free:** `sm_mutants anchors` and `sm_mutants build` over
   the whole catalogue, plus `sm_mutants run --rom-free` over all of it (it's
   small and incremental: about 2 to 3 minutes after the first build). This
   tier fails on any survivor or broken mutant.
2. **Every night, replay-level:** one slot of the `Proof::Lock` mutants
   (`--slot today/2` over 4 today: 2 per night, each with 2 rows). Each mutant
   is a verifier build plus engine-only `check-lock` on its rows. The trial
   measured about 5 runner-minutes each. This tier fails on a survivor. A
   survivor's report carries the follow-up step: free run on the row, then a
   probe at the mutated site to classify it as *not reached*, *reached but not
   seen* or *no real change* (the classes Jim set in #242).
3. **Weekly, legacy suites:** the detached and Session Python suites, split
   across a 7-day rotation of `--slot`-style parts. That needs a small change
   to both scripts: take `--part I/N` and report every verdict instead of
   raising on the first. Until then they run whole on one weekday. Retire them
   with the RAM encoder, as #242 proposes.

### Cost (runner-minutes per night, ubuntu-24.04)

| Step | Estimate | Basis |
| --- | --- | --- |
| checkout, toolchain, cache restore | 2 | the CI jobs |
| `anchors` + `build` + `run --rom-free` | 3 to 4 | local: first build 35 s, then seconds per mutant |
| 2 lock mutants x (verifier build + 2 rows) | 10 to 12 | #242 trial: about 5 each |
| **Nightly total** | **about 16** | |
| weekly legacy part (Linux, not Windows) | 15 to 30 | 27 to 30 min on Windows for 24 mutants |

### Which mutants and rows

| Mutant | Target | Rows |
| --- | --- | --- |
| `ram_botwoon_spit_spread` | engine, RAM stream | `coverage-botwoon-spit`, `boss-botwoon` |
| `apu_ports_sound_off_by_one` | engine, APU ports (the port write only; `$064D` keeps the right sound) | `item-speed-booster`, `pickup-missile` |
| `video_brightness_over_sixteen` | engine, video stream only (`sm_runtime` brightness table; VRAM and CGRAM unchanged) | `area-crateria`, `door-transition` |
| `session_streams_regroup_player` | the Session streams (moves `player` to the Scratch group) | `samus-input-poses`, `door-transition` |

Rows should come from coverage data ("rows that reach this code"). Until
`sm_pipeline` exposes that per mutant site, the catalogue names them by hand.
The next step is a `reach: &[probe]` field that `sm_pipeline plan` resolves to
rows.

The video mutant is exercised only where the engine's frames reach the
`sm_runtime` renderer. That's the Gate 1 slice (`sm_replay`) for certain. In
`verify_dll --mode check-lock`, the engine's video stream comes from the
verifier's host, which needs checking. If it doesn't use `sm_runtime`, run this
mutant through the Gate 1 tier below instead.

### Gate 1 slice mutant

`sm_pipeline gate1` compares `sm_replay gate` digests with the lock for each
machine stream. The proposed mutant is
`video_brightness_over_sixteen` (or `ram_botwoon_spit_spread`), built into
`target/release/sm_replay` and run with `sm_pipeline gate1 <row>`. A forced latch at one Tick already
showed the runner reports and locates a mismatch (Gate 1 run 42014394657). It must
print `DIFFERS in video`. That needs the same ROM and bundles as the Gate 1
workflow, so it belongs in that job's nightly slot rather than in this one.
`Proof::Gate1 { rows }` is the catalogue variant to add, with the same verdict
rules as `Proof::Lock`.

### Harness mutants still to add (need the ROM)

- **Decoder, at replay level:** apply `decoder_*` and run *lockstep* (the
  oracle's Session is decoded). Require a `Session divergence in` line. Session
  compare is report-only, so `0 failing frames` alone isn't a kill. This is
  the case #242 asks about: a mutated decoder that still reports no divergence.
  It needs a `Proof::Lockstep { rows, require: SessionDivergence }` variant.
- **Disable one stream:** mutate `compare_lock` in `tools/verify_dll.py` so it
  drops `apu`, then pair it with `apu_ports_sound_off_by_one`. The pair *should
  survive*. That shows the APU stream is the only witness. A pair that is
  still caught shows redundancy. Either way it's a measurement, not a gate.
  `compare_lock` and `gate1.check` are Python and have no unit tests. Porting
  `compare_lock` into `sm_tools` (Rust, next to `trace::Running`) would give
  them the same ROM-free mutants as the lock fold.

### Proposed workflow (not added; no workflow was changed or run)

```yaml
name: Mutations
on:
  schedule:
    - cron: '43 3 * * *'
  workflow_dispatch:
    inputs:
      slot:
        description: I/N, or today/N
        default: today/2
permissions:
  contents: read
concurrency:
  group: mutations
  cancel-in-progress: false
jobs:
  rom-free:
    runs-on: ubuntu-24.04
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@v4
      - run: rustup toolchain install stable --profile minimal
      - uses: actions/cache@v4
        with:
          path: target
          key: mutants-${{ hashFiles('Cargo.lock') }}
      - run: cargo build --release -p sm_tools --bin sm_mutants
      - run: target/release/sm_mutants anchors
      - run: target/release/sm_mutants build
      - run: target/release/sm_mutants run --rom-free
  replay:
    needs: rom-free
    runs-on: ubuntu-24.04
    timeout-minutes: 40
    steps:
      - uses: actions/checkout@v4
      # The same ROM, native dependencies and row bundles as verify.yml's replay
      # jobs (restore steps elided here; copy them from verify.yml).
      - run: cargo build --release -p sm_tools --bin sm_mutants
      # Exit 1 on a survivor, 2 on a broken mutant: either fails the job.
      - run: target/release/sm_mutants run --rom --slot "${{ inputs.slot || 'today/2' }}"
```

`--rom` keeps only the mutants that need the ROM, so the slot rotates over
the replay-level mutants alone.

## Open

- The detached suite's local run was cut short by the time budget (see the
  counts above). Run it whole in CI or overnight on a Linux runner after this
  branch lands.
- `spawn_scratch` XOR 1 (#242 trial) is still unclassified. It's a good
  `Proof::Lock` entry once a row reaches a door-closing PLM after Samus's tiles
  are set.
- Fix the `session_mutations.py` docstring (#242). It already says no workflow
  runs the mutants. When the nightly lands, point it at `sm_mutants`.
