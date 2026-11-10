# Status: how to measure it

The numbers live in committed ratchet and floor files, never in a hand-written document; the
README's status is generated from them ([BUILDING](BUILDING.md#the-status-and-the-report)).
Read them, or rerun the checks below.

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
Build with `cargo build --release -p tools`; where Defender blocks fresh executables, run the
tests through `tools/lane/run_tests.sh` instead (the ratchet has a test form).

```sh
# Lockstep: one row's report, then its ratchet lines.
target/release/tools lockstep scenarios/locks/lsnes.toml CACHE rom/sm.smc item-morph-ball
# The lockstep ratchet: exit 1 when an equal count falls or a group's count rises; prints the
# lines to commit when counts improve.
target/release/tools lockstep --check-ratchet scenarios/locks/lsnes.toml CACHE rom/sm.smc \
  scenarios/locks/lockstep.ratchet.txt
# The same check as CI runs it, as a test.
SM_CACHE=CACHE SM_ROM=rom/sm.smc cargo test --release -p verify committed_ratchet_holds_against_the_cache
# The free-run gate: fails when a listed pair isn't green; --update adds new green pairs.
target/release/tools gate scenarios/locks/lsnes.toml BUNDLE rom/sm.smc \
  scenarios/locks/lsnes.green.txt area-brinstar item-morph-ball
# The free-run localiser: each field group's first differing frame in a free run.
target/release/tools localise scenarios/locks/lsnes.toml BUNDLE CACHE rom/sm.smc item-morph-ball
# The domain harness (MODEL §4): the clock's model against every cached oracle frame.
target/release/tools harness game-time scenarios/locks/lsnes.toml CACHE area-brinstar item-morph-ball
# The harness as tests, each domain held to its floors over every row cached in CACHE.
SM_CACHE=CACHE SM_ROM=rom/sm.smc cargo test --release -p verify the_clock_holds_its_floors_against_the_cache
SM_CACHE=CACHE SM_ROM=rom/sm.smc cargo test --release -p verify samus_motion_holds_its_floors_against_the_cache
```

Commit a ratchet file only with the change that moved its numbers, and never edit a line by hand
to make a check pass. Speed: `cargo run --release -p game --example empty_ticks` and
`cargo run --release -p runtime --example compose_bench`, against ARCHITECTURE §12.

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
