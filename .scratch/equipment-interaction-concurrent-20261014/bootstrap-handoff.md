# Concurrent equipment implementation

Owner: `/root/equipment_implementation`, launched by root on 2026-10-14.
Source: `/home/jim/sm-decomp-equipment-interaction-20261014/source`.
Branch: `equipment-interaction-bootstrap`.
Scaffold baseline: `d0e35c3768a202cc997c777f9912d6817694b481`.

The whole 29-routine boundary is assigned. The worker owns only
`crates/port/src/pause_equipment.rs`,
`crates/port/tests/pause_equipment.rs`, and
`tools/port/pause-equipment.toml`. Root owns scaffold, copied dependencies,
shared inventory and integration. The source repository is independent of main.

This is actual private source authoring ahead of verification preparation.
The baseline refuses unimplemented behavior and cannot yet satisfy Native
readiness. No Installation, model rewrite, production claim or acceptance result
is asserted. Review the authored source and complete fixed fixture domain before
preparing a confined private verification contract. The original deterministic
production route and held claims are unchanged.

Shared process budget is four. Coordinate heavy test execution with the
confinement/native benchmark windows. Persistent target:
`/home/jim/sm-decomp-equipment-interaction-20261014/target`.

Next: implement and independently test the complete scope, commit the three
owned files, publish the actual achieved stage, then run fresh actual-C checks
when the benchmark window is free. Cleanup follows each verification run.

## Source completion

Private HEAD: `847d76f60d8a373b7b051f0be5c441f6be2342f4`, clean.
The three owned files implement all 29 routines and declare 427 Fixture pairs
across 28 direct entries, including 181 complete-root cases. There are 411
independent roots and 16 continuation nodes across the two explicit chains.
Manifest SHA-256: `b08cf0c2693d353f1662f45f26e211aebce446a14554304918e89010f5e0aed7`.

Focused Cargo tests and the full private workspace suite each pass all 13
independent semantic controls; tests take under 0.01 seconds. Main-config
rustfmt and `git diff --check` also pass. Checks are under
`/home/jim/sm-decomp-equipment-interaction-20261014/checks`:

- `focused.log`: `2013c6206413ebb3aa7d40e02f6d92c7a9c3e0e5fa6f9abd69e2c4b1479a6f4b`.
- `workspace.log`: `c522ed20bc253e9331952ad2604952c219a52f80e5745f36072cba7daf120ec3`.

The exact source is retained without Git/build metadata at
`/home/jim/sm-decomp-equipment-interaction-20261014/source-snapshots/847d76f`.
Both scoped cleanup utility passes on checks succeeded with zero bytes
reclaimed; the disposable target was removed after measuring 54,501,376
allocated bytes. Source and logs remain, with no active producer.

The published stages are `translated` and `verified`. All 427 actual-C pairs
passed with the improved Native runner in 97.78 seconds using two fixture
workers, including unchanged replay and post-cleanup authentication. Native
cleanup reclaimed another 91,488,256 bytes. See
[the verification handoff](verification-handoff.md) for the exact source,
reader, helper and retained result closure. Both source-review axes pass with
no findings. Main integration, Live replay and original-ROM parity remain
pending. No held main claim or production source was changed.
