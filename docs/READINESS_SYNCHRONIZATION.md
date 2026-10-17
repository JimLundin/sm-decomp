# Strict original-core synchronization

`just readiness-sync` allocates immutable evidence, verifies setup, builds the
original Compatibility core from the independently pinned official source tar
and verified private packages, captures 18,000 frames, builds this current
reference, and replays the identical clean-start P1 input. Each capture binds
its ROM/movie/canonical-input counts and hashes, seed/reset/profile settings,
actual-fetch observer, source inventory, compiler, build log and executable.

The independent gate compares all ten initialized semantic words at `$82:8948`
with original frame labels 272–18,000 inclusive. Multiple visits keep their
order. There is no offset, tolerance, event skipping or resynchronization.
A missing sidecar, incompatible identity, truncated completion, changed digest,
missing/reordered event or semantic disagreement fails. The report retains the
first mismatch and expected/completed scope. Timings and process exit codes live
in the immutable run logs. A synchronization-only pass does not accept readiness.

`just readiness-sync --original-run out/runs/<run-id>` reuses an original capture
only from a completed passing run, rehashing its observer and build/source
artifacts and checking pinned official origin. The current candidate is always
rebuilt and replayed cold. Original emulation source is verified as bytes against
the official tar with declared build compatibility edits; execution semantics
are never read to fit the candidate.

The historical full 129,712-frame capture and 36,000-frame candidate reproduce
later debt and are copied with identity links into each run. Their incomplete
legacy provenance prevents using them as readiness oracles. The first later
label disagreement remains event 15,292, original frame 18,195 versus 18,196;
first later gameplay disagreement is event 24,934, frame 35,203, X=44 versus 43.
See [the independent baseline](LSNES_BASELINE.md) for the historical probes.

An independently verified original capture can also be preserved as its own
immutable run with `cargo run -p pipeline --locked --offline -- readiness-sync --attest-original <run>`. This
rehashes all captured artifacts and reconstructs expected source bytes from the
official tar plus the maintained build-only recipe. It preserves historical
commands and configuration paths as recorded facts. The attestation links the
original capture owner as a retained dependency; preserve that run, its source
snapshot, packages and capture artifacts. The original run is never changed.
Use the resulting original-attestation run with `--original-run` to avoid another
clean build and original replay. This attestation certifies only the oracle.
