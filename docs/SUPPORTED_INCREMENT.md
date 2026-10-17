# One supported routine/width increment

`just supported-increment --selection SELECTION_RUN` consumes a complete measured
selection matching the current source. It allocates immutable preparation evidence
before setup, retains the accepted source/config and exact chosen keys, and builds
an isolated proposal using the production generator. It then applies only the
reviewed functions configuration and starts a separate exact-source validation run.
The resulting candidate stays available on failures; accepted evidence is preserved.

`--prepare-only` creates the isolated proposal without changing reviewed source.
`--preparation PREPARATION_RUN` later applies that proposal only if the current
source still exactly matches its preserved pre-mutation snapshot. User comments
and unrelated edits survive; intervening source/config changes require reselection.
ROM-derived bodies remain ignored local evidence. `--original-run`, `--timeout`
and `--jobs` carry through to existing readiness gates.

The candidate has exactly the baseline compiled keys plus the selected observed
new keys. Every other new CFG key is explicitly recorded as deferred and excluded
from the actual resumable table by source-owned `[[excluded_step]]` declarations
in reviewed functions configuration. Each exclusion names an address, width and
reason. Generation removes that body and dispatch row, retaining other widths
and legacy baseline keys. Trace merge preserves exclusions. A coverage-list change
cannot substitute for removal from generated execution. If cold execution reveals
an unexpected uncovered key, that failed candidate must be preserved and a new
pruned candidate prepared and validated; completed reports cannot be amended. `--prune-uncovered FAILED_RUN` prepares
a fresh retry excluding only added/unexecuted cold keys; affected legacy bodies
remain mandatory and require diagnosis.

The unchanged readiness pipeline requires candidate check/lint, all declared
instruction/timing evidence, fresh generated/reference parity over all 18,000
any% frames, complete changed/affected-key execution, focused instruction/bus
comparison and cold-validated paired cache evidence. Independent gates reuse only
verified exact dependencies. Warm focus targets 60 seconds and each cold replay
1200 seconds; overruns are reported and actual timeouts remain incomplete.

Iteration validation additionally records `increment-progress`. Expansion must
have positive observed generated-instruction gain over identical ROM, movie,
inputs, hardware profile, reset, seed and interval. Instruction totals exclude
interrupt/wait/stop dispatch. The gate is recomputed from sealed coverage and
accepted predecessor counts before atomic pointer replacement. Correctness fixes
must explicitly declare their kind, rationale and digest-bound passing focused regression linked to a preserved failure, passing
predecessor/disabled controls and the same repaired mandatory focus interval;
zero gain is then reported honestly. No failed, uncovered or incomplete increment
can advance acceptance or count toward a session target.

A readiness baseline refresh preserves predecessor lineage while validating
current baseline coverage without pretending source-only implementation changes
are a selected increment. Its unexecuted legacy keys remain unvalidated debt.
Supported expansion always requires exact new and affected body-key coverage.

Public-seam tests generate a synthetic ROM proposal to verify physically absent
deferred keys, retained neighbors and widths, source preservation, and rejection
of zero-gain or incomparable expansions. Actual commercial-domain acceptance is
performed by the final integration demonstration, not claimed by these fixtures.
