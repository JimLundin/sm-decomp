# Local verification pipeline

Rust owns process leases, authenticated compiler caches, cleanup, ordinary
offline families, dispatcher/callback/PLM verification, fixture generation,
packed live replay, decompressor acceptance and RNG jobs. Several worktrees share one capacity configured by
`SM_PIPELINE_WORKERS` (default 4) under the Git common checkout's `out/pipeline`.
Set the same `SM_PIPELINE_ROOT` to share an explicit directory.

```sh
cargo run -p pipeline --locked --offline -- batch verify --families system audio --jobs 2 --case-jobs 4
cargo run -p pipeline --locked --offline -- batch status
cargo run -p pipeline --locked --offline -- batch catalog
cargo run -p pipeline --locked --offline -- family-verify --manifest tools/port/system.toml --output out/port/system --case-jobs 4
cargo run -p pipeline --locked --offline -- family-authenticate --verdict RUN/verdict.toml
cargo run -p pipeline --locked --offline -- plm-fixtures --output-dir out/port/plm-fixtures
cargo run -p pipeline --locked --offline -- fixtures skree-particles out/port/skree-fixtures.toml
cargo run -p pipeline --locked --offline -- batch replay --batch BATCH/batch.toml --frames 1000
cargo run -p pipeline --locked --offline -- routine-live --accepted ACCEPTANCE/decomp-acceptance.toml --output out/decomp/live
cargo run -p pipeline --locked --offline -- decomp-jobs --accepted ACCEPTANCE/decomp-acceptance.toml --output out/decomp/jobs
cargo run -p pipeline --locked --offline -- lsnes-replay movies/super-metroid-any-3653.lsmv --frames 390 --out out/sync/lsnes.csv
cargo run -p pipeline --locked --offline -- batch verify --families plm-instructions --case-jobs 4
cargo run -p pipeline --locked --offline -- live-authenticate RUN/verdict.toml
cargo run -p pipeline --locked --offline -- cleanup --root out/port --apply
cargo run -p pipeline --locked --offline -- cleanup --root out/port --deduplicate --apply
```

Native batch workers own distinct immutable run directories; atomic batch saves
use a per-batch OS lock. Reuse authenticates current dependencies and full retained
outputs. Each case runs bounded Rust before C, then compares complete canonical
WRAM and return values. Frozen live authentication verifies all four subjects
through their retained reader and pointer seals.

Readiness, bounded sessions, pattern proof/promotion, retained demonstrations,
original-prefix diagnostics and storage studies also run natively. Small metadata and configuration use TOML; material states and ordered traces use versioned binary formats.

Commands without options accept a complete TOML document on stdin, terminated
by EOF, and print a TOML document. The interactive `cache` command reads a
TOML request and emits TOML replies prefixed by a four-byte little-endian byte
length. A cache miss names the pending artifact; writing `commit\n` publishes
it only after source identities are checked again. `lease ROOT WORKERS` emits
a length-prefixed TOML receipt and holds capacity until stdin closes.
Historical sealed evidence keeps its original format. JSON ingestion is limited to external reference corpora.
The no-Python integration test checks maintained source, recipes and CI.
Final migration controls and cleanup are tracked in `.scratch/rust-tooling`.
