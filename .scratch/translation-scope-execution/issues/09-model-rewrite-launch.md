# Launch deterministic execution with a model rewrite

Status: resolved
Blocked by: None
Triage: ready-for-agent
Owner: `01a136a0-49bb-7690-a360-48def3c2c09b`

The user requested a runnable command whose complete orchestration is Rust-owned,
with model-backed source rewriting as its only nondeterministic operation.
Use the Responses-compatible Foundry provider and existing `FOUNDRY_TOKEN`.

## Acceptance

- A bounded Rust HTTP client sends only a frozen Rewrite capsule and strict
  Source replacement schema; the model has no tools or host capabilities.
- Durable requests, responses and stop records prevent accidental reevaluation.
  Credentials are never retained or included in output.
- One maintained launch command prepares and advances an ordered approved scope
  plan, authenticates completion and preserves stopped/ambiguous operations.
- Integration, prescribed actual-C gates, cleanup and recovery remain owned by
  the existing trusted Rust supervisor. Historical proofs remain immutable.
- A concrete local configuration and real model/actual-C evidence make the
  command usable. Preserve production held claims and source until the separate
  reviewed switch from ticket 08.

## Comments

Startup inspection: executor HEAD `767db67`; clean tracked state. Production
authority, Ceres ledger, canonical ledger and progress hashes match checkpoint
06. The production integrator thread's last turn is completed. No gate or probe
writer remains. Root owns this isolated worktree and its Git/index. Existing
returned agents are completed/errored; no replacement agents were launched.

Provider preference changed from existing Codex login to direct Responses API
at the user's instruction; the user identified Foundry token and URL. User Codex
configuration already declares the Foundry Responses base and `FOUNDRY_TOKEN`.

## Completion evidence

Implementation is committed at `93e9d55b19630ac8fa2e3526ba963296385ea516` on
`architecture/translation-scope-execution-20261013`. The bounded Rust Foundry
Responses adapter accepts only the frozen Rewrite capsule and Source replacement
schema. Immutable intents and exact helper/process identities recover original
responses and preserve ambiguous delivery without resend. The ordered Rust
launcher owns preparation and the existing supervisor lifecycle; historical
installation identities remain unchanged when no provider is installed.

The [concrete request](../launch-foundry.json) is prepared and reports `ready`.
The [operator guide](../../../docs/TRANSLATION_EXECUTION.md#one-command-foundry-launch)
gives the single launch command and deliberate recovery commands. Its scope is
the approved isolated enemy-jump instructions; production migration remains
under ticket 08.

The [checkpoint](../model-launch/checkpoint.md) and
[exact reference hashes](../model-launch/checkpoint-sha256.txt) record four
passing transport tests, three passing ordered-launch tests, the real Foundry
readiness exchange, a complete real Foundry/native CLI lifecycle and unchanged
retained authentication. The real lifecycle used one model call and passed four
actual-C gates with 16 Fixture pairs per gate. Locked/offline builds, both existing
no-Python/configuration checks, formatting, diff check and affected-crate Clippy
passed with the established shared warning allowances.

Both root cleanup passes and all ten retained diagnostic cleanup pairs exited
successfully. Root deduplication checked 96 snapshots; utility reclamation was
0 bytes. The supervisor retired its producer checkouts, reclaiming 1,935,173
checkout bytes and retaining 60,877,941 bytes of evidence. Measured disposable
build/helper/C-output retirement removed 6,206,532,996 logical bytes and reclaimed
5,777,862,656 allocated bytes. Observed remaining space was 186,937,880,576 bytes.
The stable CLI, exact retained reader, complete evidence closure and private
ROM/C/provenance inputs remain available; post-cleanup reports authenticate
`complete` for the proof and `ready` for the user command. No producer remains
running and no cleanup action remains for this ticket.

Read-only authority status matches checkpoint 06. Main's source and pre-existing
assignment edit, held production claims, both ledgers and progress were preserved.
This infrastructure proof grants no Live replay, original-ROM parity or
production gameplay stage.
