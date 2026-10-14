# Measured Fixture format decision

Binary fixtures are authoritative, as requested. Use an uncompressed indexed container
with typed binary metadata, contiguous write records and shared payload bytes. Compile
overlapping root writes into merged spans at import/generation time. Continuations keep
every explicitly touched byte and never bridge holes containing inherited state.

The Rust study used the actual Manifest contract and compared full WRAM for 14,573
Fixture preparations, including 4,032 continuations with three synthetic inherited
states. All metadata was compared. Selected inputs did not include captured before_file
fixtures; production validation must cover them separately.

| Offline family | TOML read/hash/decode | Merged binary read/hash/decode | TOML including state | Binary including state |
| --- | ---: | ---: | ---: | ---: |
| floater-spike-motion | 2,924.57 ms | 6.30 ms | 2,937.88 ms | 17.11 ms |
| mother-brain-bomb-update | 1,069.33 ms | 11.50 ms | 1,083.31 ms | 21.18 ms |
| ridley-subsystem | 635.93 ms | 10.83 ms | 693.85 ms | 56.12 ms |

These are warm-cache medians over three repetitions; other machine work was active.
Every timed input was SHA-256 checked. These results measure input preparation, not
complete actual-C verification or gameplay. The prototype retained a JSON metadata
section; production replaces that parser with typed binary metadata.

Exact nonzero spans fragmented floater-spike-motion into 1,906,552 writes. Merging
root gaps of up to 16 known fill bytes avoided that problem. Full root images required
excessive I/O and hashing. Compression results varied, so the execution representation
will remain uncompressed. The three selected merged binaries total 7,947,018 bytes.

The Fixture module owns validation, span access and import/export. Its interface
provides locality for format changes and leverage across ordinary, callback, PLM and
Ridley adapters. Discovery reads binary metadata; per-case declarations retain compact
identities; prepared native work seals fixture references rather than expanded payloads.
Fresh admission and reuse still authenticate current inputs.

Measurements: results-first.json and results-merged.json. The retained executable and
nine final binary variants are in out/fixture-format-performance/20261014T191818Z.
The original family model is snapshotted beside the benchmark sources.

## Production integration

The active corpus now consists of 177 authoritative `.smf` files: 40,045,625 bytes,
down from 272,739,363 bytes (85.3% smaller). All 149,120 cases retained their metadata
and modeled input state, including 6,956 continuations and 1,123 captured cases.
The audit compared every WRAM byte under three varied inherited/capture inputs.
The original 5,016,816 writes became 1,550,905 shared binary spans.

| Offline family | Original TOML read/hash/decode | Final production read/hash/decode |
| --- | ---: | ---: |
| floater-spike-motion | 2,924.57 ms | 2.52 ms |
| mother-brain-bomb-update | 1,069.33 ms | 3.20 ms |
| ridley-subsystem | 635.93 ms | 2.50 ms |

The final series used the final codec source, five warm-cache repetitions and
the installed source fixtures. It ran after this task's tests finished; other
machine work remained active. Exact samples and machine load are recorded in
`results-final-loading.json`. The earlier production series is preserved in
`results-production-migration.json`. Neither series measures complete verification
or gameplay, and the full corpus was not executed against C in this change.

The captured Ridley family had one dependency on the renamed authoritative source.
`results-final-corpus.json` records its redirect from `ridley-subsystem.toml` to
`ridley-subsystem.smf`, both binary hashes and confirmation that every other field
and write remained unchanged. Historical capture descriptions and external
capture files were preserved; the modeled migration did not reauthenticate them.

Verification covered the codec, all checked-in matrices and current generators,
discovery/progress, prepared native hydration, ordinary/callback/PLM actual-C
execution, reuse and tamper rejection. The new public authoring regression covers
relative paths, byte-identical inspect/import, implicit case order and failed
imports preserving the prior binary. Standalone camera output is byte-identical
to the checked-in binary; the trail generator preserves all cases and differs in
family metadata because it binds its source path to the supplied checkout.

`out/fixture-format-performance/20261014T202443Z/actual-c/` retains two compiled-C
runs of root/continuation inputs in fresh and fork modes. Every before/after WRAM
byte is independently asserted, including explicit zero patches and inherited
holes; compact case identities and post-cleanup authentication also pass.
The exact reader and final measurement executable/source are retained alongside
those runs. `source-original/` reconstructs the initial study source files at
their original pre-migration hashes, with relocations recorded in
`original-study-relocations.json`. The initial `retained.sha256` is a historical
checkpoint; `retained-final.sha256` is the current retained-file inventory.
