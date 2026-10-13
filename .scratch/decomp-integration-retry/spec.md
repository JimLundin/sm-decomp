# Authenticate completed infrastructure integration retries

Status: resolved
Blocked by: None
Owner: `/root/drill_review`
Coordinator: `01a13559-e517-72d1-88b5-e554fed61da4`

After disjoint A and B infrastructure integrations complete serially, the exact
A completed-result retry rejects because B legitimately advanced shared HEAD.
The retry must authenticate A's sealed completed acceptance at its original
source epoch while keeping current claim/role and exact-request fencing.

Implement in Rust under the worker's isolated approved scope. Preserve old drill
source, requests, gate receipts, authentication, cleanup and failure evidence.
The behavioral correction passed the fresh actual independent-session drill
and private readiness/cleanup; [preparation ticket04](../decomp-parallel/issues/04-concurrency-and-crash-recovery-drill.md)
retains its final evidence.
The gameplay spec/map remain open; production authority/ledger stay absent.

The implementation ticket is
[01](issues/01-authenticate-completed-integration-retries.md). Read
[the reproducibility audit](reproducibility-audit.md) before preparing the new
drill successor. Root freezes the new exact engine source/binary and owns actual
drill/readiness integration, historical evidence retention and final cleanup.

## Completion evidence

Rust source `57f93c5ce4e6adac2dfe181b852d4aad6e0776bd` is integrated as
`0d042e0`. [Ticket01](issues/01-authenticate-completed-integration-retries.md)
records retained red/green controls, exact corrected frozen build, both completed
replays after later merge/retirement, no gate reruns and final cleanup/authentication.
Final root checkpoint: `.scratch/decomp-parallel/preparation/final-checkpoint-20261013.md`.
