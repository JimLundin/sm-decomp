# Independent review: production persistence installation

Reviewer: `01a145c8-9bc8-78a2-b0f9-8519ad5f75f5`, 2026-10-16.
Disposition: **producer26 closes the ordinary-runtime installation gap and
supports the declared compact composition scope.** No new translated body,
gameplay-driven Save/Load, file reopen/continue or original-ROM stage follows.

Frozen candidate: `9868614012ae0ee0df6c2b2a40ed16778a0f0bac`.
Evidence root: `/home/jim/sm-decomp-restart/restart/out/persistence-runtime/`.
Reviewed bridge/control/runtime sources match that candidate; subsequent current
HEAD had no differences in those files at inspection.

## Installation and demonstrated calls

The public `persistence_runtime::install` uses the already-verified24 codec,
forwards the existing disk service, maps Done/Loaded/Refused exactly and propagates
errors. There is no second codec, shadow SRAM or production raw-FFI adapter.
The composition example imports this same production function.

Normal CLI selection inherits `--gameplay`, supports independent `--persistence`
overrides and rejects invalid/prepare-only combinations. It installs the bridge
immediately after authenticated `Game::boot`, before any host step, preserving
the owned output/saves cwd. It retains bridge/codec source snapshots, reports
configured owner and installed status separately from typed call counters, and
records final SRAM identity. This is the production wiring missing from24.
Existing focused logs show19runtime tests passing, including the selection
matrix; shared Clippy/build success is recorded in the producer verification note.

The control invokes the ordinary exported `SaveToSram` and `LoadFromSram`
symbols six times, rather than substituting the diagnostic persistence dispatcher:

- Disabled versus installed representative Save→Load: full input/output WRAM,
  SRAM, effects, observer bytes and typed output compare byte-for-byte.
- Both representative Save results also match the six declared retained24
  input/expected artifacts. The bridge reused existing codec evidence.
- Disabled versus installed area6 Save: full state/effects/observations agree;
  the installed path records one unchanged refusal followed by native C.
- Saved files match canonical8192-byte SRAM in all four isolated case directories.
  Backups match the retained A5-filled prior save. Both Loads return Loaded(0).

The independent retained-byte comparisons pass. Each complete output is
131,072-byte WRAM and8,192-byte SRAM. Aggregate counters are:

| Counter | Save, Load, Pack, Unpack |
| --- | --- |
| Entries | 4, 2, 0, 0 |
| C entries | 3, 1, 0, 0 |
| Rust entries | 2, 1, 0, 0 |
| Rust refusals | 1, 0, 0, 0 |
| Rust completions | 1, 1, 0, 0 |

Disk requests/completions are4/4. Source assertions check the exact
Begin/DiskBegin/DiskEnd/End Save order and Begin/End Load order, including full
post-codec state at disk entry. Helpers remain local implementation calls; zero
separate Pack/Unpack callback counts do not mean missing codec work.

Eight sibling owners, including player motion, are registered. The control
asserts their counters stay unchanged and reports `other_owners_exercised=false`.
This proves the installation composes with those registrations, not that their
gameplay behavior ran. Area6 is a seeded safe-domain example, not a live Ceres
blackout. The result explicitly reports `live_gameplay=false` and
`file_reopen_continue=false`.

## Reuse, identity and cleanup

The actual selected native library is player25's `f06c662c…` recipe, SHA256
`c02576a7672daca9cf737b18055a2fc08bc8f7591ee96b6b33622345eb8380cd`.
It matches control inputs and the declared dependency seal. No old24 library is
misrepresented as the new one. Both candidate archive/control executable seals
and all seven retained reference/native dependencies pass after retirement.
The new source/control is scoped to the changed installation; the previous
30-body/15-fatal matrices were not rerun.

Key reviewed identities:

| Artifact | SHA-256 |
| --- | --- |
| `control-01/result.toml` | `07fe8aea30a4c040bc7c5426b9a118f5cf167b6595f0f5af5c7cd50b0701e188` |
| `control-01/counters.toml` | `3a8c1ac681f78757ab5245d4ac78b3284815ef67da88e4cb39e30bd008dcef14` |
| Retained production bridge | `ea7aa93775c1e957eee3716bbcb179c6def5757e90c4960fa40966282aa51143` |
| Retained composition source | `c2e6a1d87b3e89488d0e436c164bcbbac5d13fcd729ebade76c3ba0cafc7ed05` |

The runtime-test log records3.76seconds compilation and0.04seconds test execution.
The supplied compact result records no total control wall time; no speedup or
complete delivery-time estimate is inferred. This review adds only read-only
artifact comparison and source/receipt inspection, with no native/game/test run.

Both cleanup passes for control01 and the whole scope report zero disposable
bytes. Worker retirement reduces allocated bytes from222,556,160 to53,248,
reclaiming **222,502,912 bytes**. The worktree is absent from Git registration
and contains no `.git`; its retained bridge/control sources match worker-source
copies. Available disk at retirement is98,490,286,080bytes. The shared25 native
library/source/manifest, retained24 references and root warm target remain needed.

The report is ready for the integrator's immutable copy and enclosing final
seal. Root owns publication; this review does not claim an unpublished final
manifest passed. Only this report and local tracking were authored; reviewer
cleanup is0bytes, with no disposable outputs. The old runtime-hook limitation
is now closed for installed call paths. Gameplay saving/reopening still requires
its own later evidence, and producer25 retains its independent acceptance work.
