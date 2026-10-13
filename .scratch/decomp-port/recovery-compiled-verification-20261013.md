# Recovered live ABI and admission controls

Status: passing compiled controls; runtime acceptance and final cleanup pending.

`cargo test -p pipeline --locked --offline --test live_frames` passes all three
compiled controls. They execute the actual Rust ABI and C wrappers, proving
pre-mutation refusal, accepted-error termination without C retry, complete
WRAM/eight-word host publication, native pointer-kind recording, attack-frame
parity, camera-frame parity and internal allocation before/after observation.
Log: `recovery-live-frames-tests.txt`.

`cargo test -p pipeline --locked --offline --bin sm-pipeline live::tests` passes
seven selected tests, including bounded native roots/signatures, ownership
counter exclusivity, missing-material rejection and full-route allocation
admission. Log: `recovery-live-admission-tests.txt`.

The compiled test helpers remove their temporary directories after success.
The shared workspace target remains needed for ongoing verification. No
independent Git worktree was created. `git diff --check` passes. Current-source
Offline and four-process live gates remain required for integration admission.

The five integrated port suites now pass 46 tests (camera8, trails10,
Ridley attack7, room9, Samus12); log `recovery-port-integrated-tests.txt`.
Strict all-target port Clippy passes in `recovery-port-clippy.txt`.
Pipeline-bin Clippy passes with the four previously documented baseline
exemptions (`filter_next`, `collapsible_if`, `chunks_exact_to_as_chunks`,
`too_many_arguments`); log `recovery-pipeline-clippy.txt`. This does not claim
an unqualified strict workspace Clippy result.
