# Status: how to measure it

The numbers live in committed ratchet and floor files, never in a hand-written document; the
README's status is generated from them ([BUILDING](BUILDING.md#the-status-and-the-report)).
Read them, or rerun the checks below.

## The headline milestone

Progress is measured first against whole-movie acceptance (MODEL §1 decision 14): the engine
replays a whole movie from one start state with nothing fed back (no re-seeding, no recorded
state or timing, only the movie's inputs), and every compared cut and output byte matches. The
first target is power-on through Ceres: the any% movie from power-on to frame 15,428, where its
seed `any:12` loads the first room on Zebes. A divergence is localised by sparse checkpoints over
the whole run, then dense ones over the range they narrow it to; the run and its checkpoints are
platform work V2 of the [plan](architecture/domains.md#4-order).

The README's first row is generated from `[milestone]` in `docs/architecture/domains.toml`: the
stretch's frames, those the engine plays alone today, and the plan items still needed. Until the
whole run exists, "plays alone" counts the stretch's frames inside rows of the movie whose every
stream the free-run gate holds green, each row run from its own seed. The checks below serve it.
[domains.md §4](architecture/domains.md#4-order) holds the stretch: its frame ranges, what each
one runs, the lock rows that cover every frame of it and the order of work toward it. The
`any-power-on` row starts from power-on, which the timeline places in no movie, so its 568
frames don't count toward "plays alone" yet.

## The ratchets and floors

| File | Holds | Checked by |
| --- | --- | --- |
| `scenarios/locks/lockstep.ratchet.txt` | per row, Ticks fully equal in lockstep, and per field group the differing Ticks and first frame | CI `verify / lockstep-ratchet` |
| `scenarios/locks/lsnes.green.txt` | the free-run gate's green (row, stream) pairs against `scenarios/locks/lsnes.toml` | CI `verify / free-run-gate` |
| `crates/verify/src/harness/floors.txt` | per domain harness and row, the frames matched on every unmasked field | `verify`'s harness tests, over the rows cached in `SM_CACHE` |

Fully equal counts must not fall, differing counts must not rise, green pairs never go, and no
harness row falls below its floor.

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
SM_CACHE=CACHE SM_ROM=rom/sm.smc cargo test --profile verify -p verify samus_motion_holds_its_floors_against_the_cache
```

Commit a ratchet file only with the change that moved its numbers, and never edit a line by hand
to make a check pass. Speed: `cargo run --profile verify -p game --example empty_ticks` and
`cargo run --profile verify -p runtime --example compose_bench`, against ARCHITECTURE §12.

## Making the record cache

`CACHE` is made by `oracle corpus ROM scenarios/lock.toml DATA --oracle lsnes --native-seeds
BUNDLE/seeds --from-frame-pads PADS --cache CACHE --only ROW` (ORACLE §6): lsnes plays the row's
frame pads from its native seed and writes `CACHE/ROW/records.bin` and, last, `index.txt` with
the lock pins `verify::cache` checks. `DATA` is the old lock's decrypted bundles (corpus still
reads each row's seed and movie name there); `--cache-pictures` adds the pictures, which only
the compositor's checks read. The two CI rows were made this way, with pictures (330 KB a
frame); without them a frame is about 206 KB.

`tools/oracle/record_caches.sh ROWS BUNDLE DATA ROM CACHE [JOBS]` does it for a list of rows,
8 lsnes at a time, skipping rows already cached, one log line per row. `ORACLE` names the oracle
command (the Python DLL runner where Defender blocks executables). `tools/oracle/samus_motion_rows.txt`
lists 32 rows (18,331 frames, about 3.8 GB, about 2 minutes on 14 cores) for the Samus-motion
harness, whose test runs every row cached in `SM_CACHE` and holds the CI rows to their floors.
