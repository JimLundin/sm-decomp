# Status: how to measure it

The numbers live in committed ratchet and floor files, never in a hand-written document; the
README's status is generated from them ([BUILDING](BUILDING.md#the-status-and-the-report)).
Read them, or rerun the checks below.

## The headline milestone

Progress is measured first against whole-movie acceptance (MODEL §1 decision 14): the engine
replays a whole movie from one start state with nothing fed back (no re-seeding, no recorded
state or timing, only the movie's inputs), and every compared cut and output byte matches. The
first target is power-on through Ceres: the any% movie from power-on to frame 15,196, its seed
`any-opening:7`, after the flight to Zebes and before the Landing Site loads. A divergence is
localised by sparse checkpoints over the whole run, then dense ones over the range they narrow it
to; the run and its checkpoints are platform work V2 of the [plan](architecture/domains.md#4-order).

The README's first row is generated from `[milestone]` in `docs/architecture/domains.toml`: the
stretch's frames, those the engine plays alone today, and the plan items still needed. Under it,
the ladder's rungs (`[[rung]]`, MODEL §1 decision 18) give the same three figures for shorter
stretches of the movie, Ceres first. Until the whole run exists, "plays alone" counts the
stretch's frames inside rows of the movie whose every compared stream (all but `wram`, which is
recorded, never compared) the free-run gate holds green, each row run from its own seed. A row
the gate can't start yet (a room, species or game state the engine doesn't know) is reported as
not starting and counts toward no pair.

A rung counts only the streams its needs claim: `video` once it needs the line program (E3),
`apu` once it needs the sound requests (U1), as `[[stream]]` in `domains.toml` lists them; a rung
without that work is measured on the other compared streams. The milestone needs every stream.
Beside "plays alone", each rung shows figures that move before it is reached, over the lock's
rows that start inside its stretch: the rows the free-run gate starts, their green (row, stream)
pairs over the streams the rung counts, and three figures from the domain free run
(`free_floors.txt`). It accounts for each of a row's frames after its seed once, per domain:
**compared** (run from a seed, held or diverged; **masked** when another writer moved some of
its fields, compared on the rest), **out of scope** (the domain doesn't run in it, or another
domain owns it: the door passage for every domain but world-doors, play for world-doors),
**unreached** (the domain answers for it but doesn't compare it: its model doesn't take it yet,
or the harness can't decode it) or **folded** (a record the domain folds into the frame before
it, such as lag). Each harness names which of its skips are out of scope (`Uncompared` in
`crates/verify/src/harness.rs`); a domain's **scope** is its compared and unreached frames.
"Least held" is the domain that holds the smallest share of the frames it compares, so a domain
that skips or folds frames isn't capped by them; "least compared" is the domain that compares
the smallest share of its scope, so the frames a domain's proof doesn't reach yet stay in view
and a domain that exists for a few frames is measured over those. "Least gain over the null
model" is the domain whose held frames exceed those the null model's free run holds by the
smallest share of the frames it compares ([below](#the-models-gain-over-the-null-model)).
The report's [ladder section](REPORT.md#the-ladder) gives every domain's figures by reason and
the frames no model domain compares, its gap: game-time, the clock, compares every frame, so it
is left out of the gap, which would otherwise always be empty. The checks below serve the milestone
and its rungs. [domains.md §4](architecture/domains.md#4-order) holds the stretch: its frame
ranges, what each one runs, the lock rows that cover every frame of it and the order of work
toward it. The `any-power-on` row starts from power-on, as the movie does, so the timeline
places it at the movie's frame 0 (a power-on row belongs to the movie its id's first word names);
its 568 frames count once the gate runs it, which today runs only from seeds.

## The models' gain over the null model

A match rate means only what it gains over a null model, a step that does nothing (MODEL §4):
on a frame where the domain's fields don't move, doing nothing matches the original as well as
a correct model does, so the frame proves nothing. The README's models table therefore
headlines each model's **gain over null**: of the frames its per-frame harness compares, those
it matched on every unmasked field and verified, less those the null model matched, with that
gain as a share of the compared frames. Matched, null and compared stand beside it, from
`crates/verify/src/harness/coverage.txt` (the report's
[oracle coverage](REPORT.md#oracle-coverage)). A model in progress whose gain is under 10 %
of its compared frames is flagged under the table: its rows hardly exercise it, so its floors
can't tell it from an absent model.

Per catalogue entry the report says the same: an entry is proven by the rows only by its gain,
the frames that exercised it and matched beyond those the null model matched; an entry the
null model matches as often as the model is **unproven**, however many frames it matched, and
one no compared frame exercises is checked by its **vectors only**.

The ladder reads the domain free run the same way. Beside each domain's free run the test runs
the null model free from the same seeds, carrying the seed's fields unchanged and ending its
segments at its own divergences, masks and skips, and `free_floors.txt` holds its held frames
exactly (`null-held`). A rung's "least gain over the null model" is the domain whose held
frames exceed the null model's by the smallest share of what it compares; a domain that holds
no more than the null model gains nothing. So "100 % held" beside a small gain says the rows
don't move that domain's fields, not that its model is proven.

## The ratchets and floors

| File | Holds | Checked by |
| --- | --- | --- |
| `scenarios/locks/lockstep.ratchet.txt` | per row, Ticks fully equal in lockstep, and per field group the differing Ticks and first frame | CI `verify / lockstep-ratchet` |
| `scenarios/locks/lsnes.green.txt` | the rows the free-run gate starts (`ROW starts`) and its green (row, stream) pairs, against `scenarios/locks/lsnes.toml` | CI `verify / free-run-gate` |
| `crates/verify/src/harness/floors.txt` | per domain harness and row, the frames matched on every unmasked field | `verify`'s harness tests, over the rows cached in `SM_CACHE` |
| `crates/verify/src/harness/free_floors.txt` | per domain and row, the frames its free run holds from its seeds (a floor), those the null model's free run holds (exact), and how it accounts for the rest, in and out of its scope (exact); per row, the frames no model domain (all but game-time) compares (exact) | `every_domain_runs_free_against_the_cache`, over the rows cached in `SM_CACHE` |

Fully equal counts must not fall, differing counts must not rise, a row that starts keeps
starting, green pairs never go, and no harness row falls below its floor.

Each command needs the ROM (`rom/sm.smc`) and data from ci-data, decrypted as
[ORACLE §7](ORACLE.md#7-data-the-rom-bundles-and-the-cipher) shows: `BUNDLE` is the relock bundle's
`relock-v2-lsnes/` directory, `CACHE` the unpacked lockstep record cache (one directory per row).
Build with `cargo build --profile verify -p tools`; where Defender blocks fresh executables, run the
tests through `tools/lane/run_tests.sh` instead (the ratchet has a test form).

```sh
# Lockstep: one row's report, then its ratchet lines.
target/verify/tools lockstep scenarios/locks/lsnes.toml CACHE rom/sm.smc item-morph-ball
# The lockstep ratchet: exit 1 when an equal count falls or a group's count rises; prints the
# lines to commit when counts improve.
target/verify/tools lockstep --check-ratchet scenarios/locks/lsnes.toml CACHE rom/sm.smc \
  scenarios/locks/lockstep.ratchet.txt
# The same check as CI runs it, as a test.
SM_CACHE=CACHE SM_ROM=rom/sm.smc cargo test --profile verify -p verify committed_ratchet_holds_against_the_cache
# The free-run gate: fails when a listed pair isn't green; --update adds new green pairs.
target/verify/tools gate scenarios/locks/lsnes.toml BUNDLE rom/sm.smc \
  scenarios/locks/lsnes.green.txt area-brinstar item-morph-ball
# The free-run localiser: each field group's first differing frame in a free run.
target/verify/tools localise scenarios/locks/lsnes.toml BUNDLE CACHE rom/sm.smc item-morph-ball
# The domain harness (MODEL §4): the clock's model against every cached oracle frame.
target/verify/tools harness game-time scenarios/locks/lsnes.toml CACHE area-brinstar item-morph-ball
# The harness as tests, each domain held to its floors over every row cached in CACHE.
SM_CACHE=CACHE SM_ROM=rom/sm.smc cargo test --profile verify -p verify the_clock_holds_its_floors_against_the_cache
SM_CACHE=CACHE SM_ROM=rom/sm.smc cargo test --profile verify -p verify movement_holds_its_floors_against_the_cache
# The domain free run (MODEL §1 decision 17): every domain fed its own State over every cached
# row, each row's segments and first divergences printed, held to free_floors.txt.
SM_CACHE=CACHE SM_ROM=rom/sm.smc cargo test --profile verify -p verify every_domain_runs_free_against_the_cache -- --nocapture
```

Commit a ratchet file only with the change that moved its numbers, and never edit a line by hand
to make a check pass. Speed: `cargo run --profile verify -p game --example empty_ticks` and
`cargo run --profile verify -p runtime --example compose_bench`, against ARCHITECTURE §12.

The same CI job plants the faults in [`tools/mutants.toml`](../tools/mutants.toml), one at a
time, with `SM_CACHE=CACHE SM_ROM=rom/sm.smc python3 tools/check_mutants.py`. It first requires
each fault's recorded floor or ratchet tests to pass on the unchanged model, then requires
all its exact named tests to fail on the changed model. A missing test or a build failure
cannot kill a fault. The source is restored byte-for-byte after each fault, and CI reports
the killed faults per domain. Movement, camera, appearance and door passage each exercise
their harness and an independent domain free-run test; the Ceres doors exercise both and the
engine lockstep ratchet.
The runner prints M0 (`chain-any-0003-DFD7`) domain floor counts separately: `ok` means that
row's held-frame floor survived, even when the test failed on another row or on accounting.
Other domains' checks have no standing planted-fault coverage yet.

## Making the record cache

`CACHE` is made by `oracle corpus ROM scenarios/lock.toml DATA --oracle lsnes --native-seeds
BUNDLE/seeds --from-frame-pads PADS --cache CACHE --only ROW` (ORACLE §6): lsnes plays the row's
frame pads from its native seed and writes the lean tier, `CACHE/ROW/lean.bin` and, last,
`lean.txt` with the lock pins `verify::cache` checks: about 2 KB a frame, each frame's memory
changes with the whole memories every 256th frame. `DATA` is the old lock's decrypted bundles
(corpus still reads each row's seed and movie name there); `--cache-pictures` writes the full
tier instead (`records.bin`, `index.txt`, 330 KB a frame with its picture), which only the
compositor's checks need. `tools cache-lean` turns a full tier into a lean one.

`tools/oracle/record_caches.sh ROWS BUNDLE DATA ROM CACHE [JOBS]` does it for a list of rows,
8 lsnes at a time, skipping rows already cached, one log line per row; with `WATCH_PCS` set it
passes `--watch-pcs` too, for the `hits.txt` the blocks, progress and HUD harnesses read.
`ORACLE` names the oracle command (the Python DLL runner where Defender blocks executables).
`tools/oracle/movement_rows.txt` lists 32 rows (18,331 frames, about 3.8 GB, about 2 minutes
on 14 cores) for the Samus-movement harness, whose test runs every row cached in `SM_CACHE` and
holds the CI rows to their floors.
