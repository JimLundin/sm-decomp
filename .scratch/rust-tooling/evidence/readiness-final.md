# Final native readiness migration evidence

Status: migration verification complete; root integration pending

Branch: `decomp-pipeline-performance` in `/tmp/sm-decomp-pipeline`.
No ticket is resolved by this section document.

## Agreed completion scope

The user explicitly requested quicker closure of the Python-to-Rust and
internal-JSON migration. The completed scope is the native workspace regression
suite, meaningful protocol/admission/failure controls, final nine-case feedback
execution and authentication, and finished-run cleanup. A new commercial
18,000-frame readiness acceptance or optimization increment is not required for
this migration closure. No new optimization configuration is introduced.

The fresh full readiness attempt `out/runs/1791746216191267883-2` was deliberately
stopped and retained as incomplete. It is not restarted or counted as commercial
acceptance. No native `out/accepted.toml` is claimed by this migration evidence.
Historical commercial readiness and synchronization evidence retain their
original format and source/reader dependencies; they are not relabeled as native
TOML evidence.

## Passing migration verification

Root reports the complete workspace regression and migrated protocol controls
passing. Readiness lifecycle, interrupted-preflight and missing/forged selection
controls pass in `/tmp/sm-root-final-controls-retry.log`; lifecycle passes again
with integrated origin authentication in
`/tmp/sm-root-auth-integrated-controls.log`. Typed coverage controls pass 2/2,
diagnosis controls 5/5, and session policy/recovery/lineage controls pass. These
exercise the transferred selection/session contracts without requesting a new
optimization increment.

Feedback run `out/runs/1791745972678334997-2` completed all nine demonstrations.
Its native authenticator passed again after finished-run cleanup. This verifies
actual execution, exact executable/source/hardware/fixture and process origins,
all expected refusal outcomes, and private accepted-pointer preservation:

- Correct execution passes through eight instructions with frame-end parity.
- Wrong/transient stores fail at instruction 3, `$00:8004/MX=3`, writing `$99`
  instead of `$12`, despite matching frame-end state.
- Adding unexecuted `$00:8100/MX=3` refuses coverage despite four generated
  instructions executing; an empty compilation refuses after two matching
  fallback frames.
- Preflight/build, signal/timeout/truncated/unfinished, incompatible/stale and
  source/input controls refuse publication and preserve prior pointer bytes.
- Paired cache controls match warm/cold traces, validate one full frame and two
  rollovers, and reject/rebuild corrupt or incompatible caches.

The feedback verdict is a nonaccepting scoped pass. Root outputs are
`/tmp/sm-root-feedback-complete.log` and `/tmp/sm-root-feedback-complete.toml`;
the retained native owner and post-cleanup authentication remain authoritative.

Native iteration demonstrations `out/runs/1791744883357572384-2` passed with a
schema-2 verdict and schema-1 demonstration report. Production accepted count is
zero. They retain policy stopping, a real instruction-86 late mismatch with
passing predecessor/disabled controls, three failed candidate experiments and
an actual refused private publication.

## Final-source refusal fixes

Acceptance previously indexed an absent optional TOML `contract_version`, causing
preflight/build refusal subprocesses to panic with exit 101. Typed
`RunManifest`/`Verdict` admission now returns the required refusal code 2; missing
iteration predecessor, scope/check/seal and pattern metadata use fallible access.
Private-pointer preservation checks remain exact.

The earlier full feedback owner `1791744701480934086-2` passed actual correct
execution then panicked while inserting a new case through TOML indexing. Case
accumulation now uses `BTreeMap`; coverage inserts new fields through the table
API. Narrow unwind-to-error handling reaches existing terminalization and retains
case failure evidence. The earlier stranded owner remains incomplete and is not
counted as acceptance; the final nine-case run passes.

The final readiness origin/publication failure paths also attempted to index an
absent optional `reason`. They now share typed failure recording: durable
`status=error`, `acceptance_eligible=false`, original error reason, and a rejected
publication receipt when applicable. The meaningful private-run regression
starts from an owned finalized passing verdict with no reason, invokes actual
origin verification of missing material and actual publication refusal, then
checks complete retained error state and absence of an accepted pointer.
It passes 1/1 in `/tmp/sm-root-readiness-final-refusal.log` (0.21 seconds).

## Interrupted corpus scope and retention

Pattern owner `out/runs/1791745974023843680-2` was stopped cleanly after 70,000 of
70,011 declared cases executed with zero failures. The remaining 11 active
literal fixtures and complete timing suite are not claimed as finished. Its
incomplete terminal verdict, pinned corpus/declaration identities, binary ledgers,
executed binaries and diagnostic logs remain retained. This partial corpus run
does not certify readiness or pattern acceptance.

Root completed finished-run cleanup and post-cleanup feedback authentication.
Cargo clean reclaimed 372.3 MiB from disposable build output. Earlier scoped
cleanup receipts for interrupted/panicked owners record valid zero-byte results;
sealed source snapshots, logs, generated executables, ledgers, paired evidence,
acceptance pointers and historical reader dependencies remain preserved.
Integration-wide measured cleanup receipts and final available disk space are
recorded by root with the overall migration evidence. No further verification
is requested by this section.
