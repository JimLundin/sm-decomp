# Separate ticket-control validator handoff

Status: validator source and binary submitted; tests and scoped cleanup complete; frozen-subject validation and integration-owner retirement pending
Blocked by: None
Owner: `/root/session_tools_finish`, `01a13474-5fcc-7861-84f8-5c353938bdd2`
Commit: `f54fbed0f30f2c84e2c6aba171dcc54e96099c9a`
Worktree: `/tmp/sm-decomp-ticket-validator-20261013`

## Validator and executed subject

Use `ticket-validator-evidence/bin/sm-pipeline-validator` (SHA-256 `becfd038af486b51e94f454eca84cfd05fcaf96ae77875a1185275ddfe9978d3`) to validate the unchanged frozen drill source checkout `/tmp/sm-decomp-preparation-integration-20261013`. `validator-build.toml` retains both versions explicitly:

- Executed engine: commit `c8799f326ffe198b652301f08866f1b7a7bd9ebc`, source fingerprint `4d87e457e895a03f82f5a7ccdd891f7769b83d25d48caf2fd3a55432e5929338`, engine SHA-256 `89e3c46596ea556011cfcbb679cace5b96277bbf56abb8a7a2a61db7048268d9`, harness SHA-256 `fabe7ff02289fee04e23707b80bb592daef6523802c12d226a11650a47237e15`.
- Validator: the separate commit above, source fingerprint `cde89f07dcf1a897abc81938581d893a3b037257bf23ec7939b5f9c11de0d352`.

The frozen engine, source, harness and executed command receipts were preserved. `checked_drill` authenticates the named subject checkout and executed binary; the validator binary is a separately retained inspection tool. These are local source/build/hash records, with no platform attestation. The subject was neither replayed nor relabeled.

## Semantic correction

The actual queue rejects a same-effort competing ticket as `ticket is claimed` before reaching its held-ticket comparison. The validator's earlier literal `ticket ownership` path remains supported. The new fallback requires all of:

- A successful, earlier linked reservation with exact claim id/revision/coordinator and original job.
- A successful competing `infrastructure-next` with no assignment, an exact `ticket is claimed` defer, and one matching candidate in its sealed reviewed scope request.
- The candidate's safe explicit ticket equals the reserved job's ticket, its id differs, and its nonempty writable paths are disjoint from that holder's paths.
- The competing output retains the exact original claim tuple as reserved/running/submitted, with `stale = false` and no pending revision.

The actual `duplicate-ticket` control must link the original B reservation before the pure competing next command. Schemas, control names, authority and adoption semantics are unchanged. A stopped integration child remains ineligible for surviving-child adoption.

## Completion evidence

Only `crates/pipeline/src/drill_evidence.rs` and `crates/pipeline/tests/session_authority.rs` changed; owned Git paths are clean. Evidence is in `ticket-validator-evidence/`, including source snapshots, source-file fingerprint maps, separate build metadata, logs and hashes.

Locked/offline tests used a private target and shared process budget (`SM_PIPELINE_WORKERS=4`, `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`) with the temporary local RustySNES patch. The receipt-path regression failed on the old literal implementation (`red.stdout`), then passed after correction (`green.stdout`). The full authority suite passed 11 tests with one ignored subprocess helper (`authority.stdout`). Adversarial cases reject absent reservation/holder, changed attempt, stale or complete holder, unrelated ticket, overlapping ownership, reversed receipts, identical job ids, and unsafe ticket paths. These fixtures are tests, not evidence of actual independent Codex execution.

`cargo build -p pipeline --bin sm-pipeline --locked --offline` produced the retained validator; exact local patch and binary/source hashes are in `validator-build.toml`. Both source fingerprints were recomputed with the project's path list, sorted relative-path/SHA-256 TOML map, and SHA-256 of that map. The old source matched the original frozen fingerprint.

After writers exited, the retained validator ran `cleanup --root out/ticket-validator --apply` and `cleanup --root out/ticket-validator --deduplicate --apply`; both succeeded and reclaimed 0 run bytes. The private target measured 1,305,174,746 bytes and was removed. The temporary Rust fingerprint audit build/source was removed. Remaining filesystem space was 222,736,785,408 bytes. The retained validator binary remains a dependency for the root readiness check. The integration owner retires the registered validator worktree after frozen-subject validation and source integration, retaining any referenced source dependencies.
