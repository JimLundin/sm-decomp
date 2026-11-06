# Status: how to measure it

The numbers live in two committed ratchet files, never in a document. Read them, or rerun the
checks below; issue #177 holds the live summary.

| File | Holds | Checked by |
| --- | --- | --- |
| `scenarios/locks/lockstep.ratchet.txt` | per row, Ticks fully equal in lockstep, and per field group the differing Ticks and first frame | CI `verify / lockstep-ratchet` |
| `scenarios/locks/lsnes.green.txt` | the free-run gate's green (row, stream) pairs against `scenarios/locks/lsnes.toml` | CI `verify / free-run-gate` |

At `5622d9e` (2026-11-06): lockstep `area-brinstar` 476 / 476 Ticks equal, `item-morph-ball`
1,372 / 1,427; free run `area-brinstar` 477 / 477 and `item-morph-ball` 811 / 1,428 frames
equal; 8 green gate pairs. Fully equal counts must not fall, differing counts must not rise, and
green pairs never go.

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
```

Commit a ratchet file only with the change that moved its numbers, and never edit a line by hand
to make a check pass. Speed: `cargo run --release -p game --example empty_ticks` and
`cargo run --release -p runtime --example compose_bench`, against ARCHITECTURE §12.
