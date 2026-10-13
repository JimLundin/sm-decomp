# Recovery coordination consistency review

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research
Owner: `/root/recovery_consistency`
Date: 2026-10-13

Read-only review of coordinator strategy, completion, progress provenance and
cleanup records. Existing source-map and evidence-publication owners remain
active; their detailed mapping reviews are outside this scope. No gameplay
verification, registry publication, maintained-source edit or binary was made.

## Result

The final bounded integration report is supported by its retained receipts.
No unfinished gameplay run or cleanup writer was found during this review;
the progress server and active observation publishers remain intentional work.
The larger playable boot-to-credits objective and whole Ridley live admission
remain unfinished and are correctly separated from bounded C/Rust integration.

The revised plan follows `docs/DECOMP_PIVOT.md`: causal inputs and a named Ceres
checkpoint establish gameplay progress, while a shared synchronous game host
should reduce repeated per-family integration work. Diagnostic pairs remain
distinct from formal four-subject acceptance. Canonical memory, ordered effects,
visible C ownership and original-ROM comparison remain explicit. No ADR
directory exists and no existing ADR conflict was found.

## Verified record consistency

- `recovery-live-completion-20261013.md` identifies integration commit
  `3ce28ba58aba319b60bbc949153baafd4433132e`. The composed and trail verdicts
  record `mixed-C-Rust-integration`, matching state/effects, zero CPU fallback
  and original-ROM parity `not-tested`. Their recorded input extents are
  129,712 and 9,000 respectively.
- Direct SHA256 checks match both post-cleanup receipts: composed verdict
  `5e224bda08b3f597b07dd3a03c59bf8cf136b288993459c62e453ef3fac400c2`,
  trail verdict
  `312036af96f789c07af1ff370800a69fbaa28fcba2a26dc3d4c3507f2e240508`.
  Receipts report four subjects and 1,881 / 94 artifacts per subject. This audit
  inspected the existing authentication results; it did not rerun the reader.
- `recovery-integrated-final-authentication-summary.toml` records twelve
  affected Offline families totaling 20,943 fixture pairs. Its six root counts
  total 7,319 and six other counts total 13,624. Automatic dedup reclaimed
  4,522,901,504 bytes; both explicit final cleanup passes report zero bytes.
- The explicit live cleanup receipts agree with the final report: composed
  ordinary cleanup reclaimed 48,176,296 bytes and trail ordinary cleanup
  47,310,242 bytes; both dedup passes reclaimed zero bytes.
- `git worktree list` contains only the main checkout. Exact source-only
  bounded, trail, Samus and whole Ridley contexts still exist. Final reports
  preserve the shared `out/evidence` store, subject pointers, source snapshots,
  binaries and retained `sm-evidence` readers as dependencies.
- Tickets 137–140 are resolved. Whole lifecycle/hardware ticket 141 remains
  claimed; its isolated handoff explicitly says no formal whole 129,712-input
  gate started and no completed-run cleanup action remains.
- Ticket 78's final terminal-boundary evidence records 1,097 causal completed
  updates through source frame 1,600. The strategy's starting frontier therefore
  matches the ticket; its earlier 1,096-prefix entry is historical.

## Findings and coordinator actions

1. **Preserve frozen receipts and add a current checkpoint.** Several old
   snapshots still say integration or authentication is pending:
   `recovery-integration-progress-20261013.md`,
   `recovery-compiled-verification-20261013.md`,
   `recovery-bounded-source-retention-20261013.md`,
   `recovery-worktree-retirement-20261013.toml`, and the Offline summary's
   `live_stages` field. The port spec still ends at the older ticket 137/138
   frontier. These are historical snapshots rather than evidence of new
   required runs. The coordinator accepted the mitigation of a supplemental
   current recovery checkpoint linked from the strategy/spec. Do not edit
   hashed acceptance reports or receipts to make them current: published live
   observations already hash `recovery-live-completion-20261013.md`, and the
   legacy trails live observation became stale after a prior report edit. Its
   new context sibling preserves valid coverage without rewriting history.

2. **Use source correspondence for implementation extent.** Earlier strategy
   wording called the whole Ridley closure 220 translated native definitions.
   The reviewed source map instead identifies a 220-definition reachable scope
   and 26 closure candidates without an owned mapping, including explicit C
   services. `spec.md` and `revised-strategy.md` have now been corrected. Keep
   the older handoff as frozen provenance and carry this distinction into the
   final coverage report. The source map's 1,452 distinct mapped native bodies
   is an implementation correspondence inventory with bounded domains; it does
   not establish equivalent branches or complete engine ownership.

3. **Close reconciliation only after publisher handoffs.** The registry changed
   throughout this review. One coherent snapshot contained 141 observations,
   five stale records, 38 current-checkout verified/integrated definitions and
   15 current-checkout live-exercised definitions. Historical-frozen and legacy
   unspecified context counts remain separate and overlap by symbol; they must
   not be summed as unique completion. Earlier resolved progress-feature counts
   of 28 observations and zero live definitions describe that feature's prior
   handoff, not the final gameplay inventory. Wait for the existing source and
   historical-evidence publishers, record a final snapshot and reconcile the
   remaining gap list before resolving the strategy spec.

4. **Make the deferred whole gate explicit in the local task.** Ticket 141 still
   names `/root/integration_audit` as an owner and lists the original full-route
   verification gate, while the user steering and retained owner handoff hold
   that gate during strategy reconsideration. Record the replacement owner and
   hold/re-entry condition in the task's comments or current checkpoint. This
   prevents a future interrupted coordinator from relaunching the expensive
   gate merely because the older task is claimed. A diagnostic pass must retain
   its stated material/context-admission limits.

5. **Use the repository's triage vocabulary.** The independent source audit's
   `note.md` currently records `Triage: researcher`; this is not a default local
   role from `docs/agents/triage-labels.md`. Its owner should choose a canonical
   role before handoff. The finding does not change its evidence status.

## Completion evidence

Inspected project instructions and glossary before exploration, then reviewed
strategy/spec, the final live report, verdict headers/hashes, Offline summary,
cleanup/authentication receipts, progress schema/context summaries, tickets
78/137–141, source audit conclusion and whole candidate handoff. Read-only shell
checks included `jq`, `rg`, `sha256sum`, `git worktree list`, `ps`, `du` and `df`.

No run directory, executable, temporary source tree or worktree was created.
There is therefore no audit-generated finished run or disposable build output
to clean. No evidence dependency was relocated or removed. A read-only disk
measurement during review reported 231,258,730,496 available bytes; this is a
point-in-time measurement while unrelated observation publishers were active.
Only this Markdown review file was written. Coordinator cleanup and immutable
evidence remain governed by the retained final receipts described above.
