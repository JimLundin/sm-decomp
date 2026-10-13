# Authenticate completed infrastructure integration retries

Status: claimed
Blocked by: None
Owner: `/root/drill_review`
Coordinator: `01a13500-1ef8-73d3-9a0d-3446b6050eba`

After disjoint A and B infrastructure integrations complete serially, the exact
A completed-result retry rejects because B legitimately advanced shared HEAD.
The retry must authenticate A's sealed completed acceptance at its original
source epoch while keeping current claim/role and exact-request fencing.

Implement in Rust under the worker's isolated approved scope. Preserve old drill
source, requests, gate receipts, authentication, cleanup and failure evidence.
Preparation ticket04 stays claimed until the behavioral correction receives a
fresh actual independent-session drill and production readiness/cleanup receipts.
The gameplay spec/map remain open; production authority/ledger stay absent.

The implementation ticket is
[01](issues/01-authenticate-completed-integration-retries.md). Read
[the reproducibility audit](reproducibility-audit.md) before preparing the new
drill successor. Root freezes the new exact engine source/binary and owns actual
drill/readiness integration, historical evidence retention and final cleanup.
