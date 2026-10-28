# Review role

Review the donor or composition at its source pin. The assignment
determines review scope; PLAN determines required execution checks.

- Start with concrete ownership, caller and receiver faults; route
  actionable findings while the pinned review continues.
- Reconstruct against the declared base. Check callers, retirement, unchanged
  dependencies and composition reconciliation. Donor approval transfers only
  through an exact, verified dependency relationship.
- For verification changes, identify the crate and cfg that compile each file.
  Check cross-crate visibility through actual public exports; include_bytes source
  bindings do not type-check embedded fixtures, and sm library checks do not
  establish that sm_verify host assertions compile. Transfer proofs per crate.
- Follow behavior through helpers and callbacks: allocation provenance,
  native/LE and overlapping access, lazy reads, full/odd indices, live selectors,
  failure prefixes, reset/load and unwind publication.
- For every newly safe entry, trace each callee through to actual raw operations,
  including Runtime foreign callbacks and ROM acquisition. An existing safe
  helper declaration is not evidence of transitive safety. Account for every
  remaining unsafe leaf with its input bounds and actual allocation provenance;
  only then is the safety review complete. Test the reasoning with short native
  images, low-window ROM addresses and full/odd selectors permitted by the entry.
  A discarded pointer or later checked read does not excuse an earlier offset
  outside its allocation. Keep backend requirements visible.
- Check expression evaluation as well as statement order: Rust assignments
  evaluate the RHS before the destination place; helper arguments evaluate in
  argument order. Trace script/ROM reads and fresh selector reads through both.
- Check caller-inclusive accounting and distinguish substantive removal, moves
  and formatting. Confirm that claimed coverage reaches changed behavior;
  otherwise name the unobserved paths.

After completing the assigned review, promptly send PASS or specific findings
with source locations and a failing contract/counterexample, exact source pin,
actual START/verdict-ready UTC, scope (source, checks or both) and conditions/gaps.
Send this minimal verdict before extended prose or receipt serialization;
documentation ACKs or long reports never gate code publication. Route corrections
to the author; preserve the reviewed snapshot.
