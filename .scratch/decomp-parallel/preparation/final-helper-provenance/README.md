# Retained final drill request helper

The corrected engine requires integration handoffs to name the exact held
`submitted_commit`. The originally frozen fixture harness omitted that optional
field. Its safely rejected original Start requests remain diagnostic evidence.
The versioned Rust helper adds the submitted commit from the unchanged ledger
and the actual calling session, then recovers the successful handoff pointer.
It leaves the claim, scope, engine, harness and frozen input unchanged.

`request-helper.diff` records the exact helper changes. The final actual A/B
launch records seal `a-corrected-start.md` and `b-integrator-recovery-v2.md`.
Independent review found only request preparation, pointer recovery and CLI
routing added. Queue admission still occurs through the maintained engine.

`build.toml` identifies the retained source and exact executable used by the
actual sessions. A separate locked/offline rebuild completed successfully and
matched the original executable SHA-256 byte for byte. Its stdout/stderr are
retained here. These are helper provenance receipts; actual drill validation
uses the final execution manifest and unchanged frozen engine.

The retained binary is outside Git and remains an evidence reader alongside
the frozen engine. Source, lockfile, manifest, diff, build receipt and logs are
sealed in `artifacts.sha256`. Cleanup removes only the separate rebuild target
after all writers finish and records measured bytes separately.
