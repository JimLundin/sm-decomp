# Planned historical-integrator preparation and cutover

Status: requests and observed lineage prepared; no authority or ledger mutated.
Owner: session-tools worker `01a13474-5fcc-7861-84f8-5c353938bdd2`.
Integration reviewer: `01a13460-4aa9-7fe0-865e-a8f54ffca310`.

`prepare-request.toml` preserves the Ceres coordinator token at generation1.
Its integrator registration includes sealed `legacy-generation-zero-transfer`
evidence accounting for all three currently held claims: compact-controller
submitted, shared-game-host submitted, integrate-shared-game-host running.
Observed child completion or failed parent does not complete or release them.
The ready unclaimed integrate-compact-controller job remains unclaimed.

Fresh tool observations at 09:22 UTC report original integrator01a133eb failed,
compact implementer idle/completed, host implementer notLoaded with completed
terminal turn, and host reviewer idle/completed. Replacement and reporting
roots both remain active. Process/proc inspection found no Ceres worktree/run/
ledger writer; active main progress/prototype/build processes are retained under
reporting ownership. `ownership-boundaries.toml` keeps main Git/index ownership
separate from the proposed assignment integrator role.

Review all source schemas and recheck real owners, ledger/bridge hash and claim
writers before using a request. Prepared authority leaves canonical production
inactive. Activation templates deliberately contain missing source/drill seals
and false controls. Root fills them only after frozen integration source and
the actual disposable two-session drill pass. This worker executes none of
these requests. Legacy claim result admission and gameplay remain separate.

The aggregate `artifacts.sha256` authenticates all prepared TOML and retained
inspection outputs. The legacy bridge itself is unchanged; no ledger or main
source/index mutation occurred. Only read-only inspection and small `.scratch`
files were created; no verification/build run cleanup is needed,0 bytes.
