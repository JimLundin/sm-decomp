# Recompilation readiness

Status: specified; Q1–Q15 agreed. Awaiting final confirmation of shared
understanding. This is a workflow specification, not a readiness result.

## Purpose, trigger and result

Run when the user explicitly requests readiness to start recompilation, or when
changes invalidate accepted feedback machinery. Implement and exercise the
contract needed by [recompilation iteration](recomp-iteration.md). The agent
owns setup, changes, checks and diagnosis, presenting a short brief at completion
or when a concrete unresolved decision prevents progress. Commit, publication
and merge require separate authorization.

The result is a readiness verdict tied to an exact source state and validated
domain. Historical passes, generation success and aggregate generated counts
cannot substitute for the mandatory gates below.

## Initial validated domain

Use the verified unheadered Super Metroid Japan/USA ROM at `rom/sm.sfc`, the
clean-start original any% recording `movies/super-metroid-any-3653.lsmv`, seed 0,
P1 input, and the explicit lsnes recording profile with pinned hardware source
and maintained patches. Verify ROM/movie digests against the movie container,
compiled table and `movies/manifest.json`.

Cold parity consumes exactly 18,000 input frames, indexed 0–17,999. Independent
synchronization compares actual-fetch checkpoints at `$82:8948`, with original
frame labels inclusively 272–18,000. Preserve multiple ordered visits per frame
and detect missing/extra visits. Compare the ten initialized semantic words
specified in `crates/snes/src/checkpoint.rs` exactly. Use the documented frame
label convention, zero additional offset, no tolerance or resynchronization.
Do not edit inputs or skip a failure to obtain a pass.

Readiness permits recompilation within this domain while later hardware accuracy
debt remains tracked. Historical evidence first differs at frame 18,195 versus
18,196, event 15,292; later first semantic mismatch is frame 35,203, X=44 versus
43. Keep links to those reproductions and their identities. They neither certify
nor invalidate a freshly passing 18,000-frame domain. Work requiring later
gameplay, low%, 100%, another ROM or initialization must independently validate
its new domain first. Full movie completion and zero interpreter execution
remain later milestones.

## Commands and immutable evidence

Provide repository-root commands for readiness, iteration and diagnostic replay,
surfaced through `just`. Extend existing generation/comparison machinery so the
user does not have to assemble a sequence of commands. Exact command names and
module boundaries are engineering choices within this contract. Print the run
directory and brief path; required failed checks return nonzero.

Allocate `out/runs/<UTC timestamp>-<unique suffix>/` before preflight or build.
Never overwrite completed evidence. Use the following layout:

| Artifact | Content |
| --- | --- |
| `manifest.json` | Versioned schema, run ID/type/scope, accepted parent ID, UTC times, commands, identities, frame conventions and reused evidence |
| `source/` | Restorable maintained source/config snapshot and ordered path/digest inventory, including relevant dirty/untracked files |
| `generated/` | Exact generated inputs and digests used by this run |
| `logs/` | Stage commands, stdout/stderr, exit codes/signals and elapsed times |
| `coverage.json` | Counts per address/MX and opcode/pattern, changed-key requirements, executed/unexecuted/unsupported inventory, generated/interpreted totals |
| `checks/` | Parity, instruction, timing, synchronization, cache and feedback-demonstration reports, expected/completed frames or cases |
| `diagnostics/` | Paired snapshots, bounded traces, first mismatch and control reports |
| `verdict.json` | Terminal status, failed stage/reason, mandatory-check outcomes, domain, acceptance eligibility and evidence links |
| `brief.md` | Outcome, exact scope, comparable progress, timing, limits, links and next action if needed |

Identities include ROM/movie/input SHA-256 and counts; seed/reset/input settings;
profile/features; pinned dependency revision, actual patched hardware source and
patch digests; source/config/generated/binary digests; lockfile and compiler;
baseline and cache digests with verified origins. Record identities as artifacts
become available: build failure cannot invent a binary digest.

Snapshot every maintained build input and relevant untracked source, excluding
ROM, downloaded material, build/run output, credentials and unrelated personal
files. Preserve ROM-derived generated artifacts locally in ignored directories.
Detect source/input changes during a check and invalidate it. Serialize shared
build/generated mutations or isolate them to prevent cross-run contamination.

Terminal statuses: `pass`, `divergence`, `uncovered`, `error`, `incomplete`.
Pass requires every mandatory check and expected scope complete. Divergence is
a comparison failure; uncovered is an unmet changed-key requirement; error names
the setup/build/infrastructure stage; incomplete covers interruption/timeout or
absent completion. Preserve all check outcomes when multiple failures occur.
Caught failures write terminal reports; uncatchable termination leaves a visibly
unfinished run that consumers treat as incomplete. No old or absent verdict
implies success. Command exit codes: 0 pass, 1 validation failure, 2 infrastructure
or incomplete failure; retain underlying codes in logs.

Update `out/accepted.json` only after all gates pass, binding the run ID, domain
and restorable source/config snapshot. Retain predecessors and rejected
candidates. Restore into an isolated checkout or explicitly selected candidate;
never overwrite unrelated user edits. Acceptance does not imply a Git commit.

## Provenance and reuse

Existing inputs include `config/functions.toml`, `config/jump_tables.toml`,
`crates/recomp/src/step.rs` and `movies/manifest.json`. The original-core capture
`out/sync/full-any-lsnes.csv` has completion metadata for 129,712 frames. Reuse
only after verifying CSV hash, completed scope, ROM/movie/core identity and
actual-fetch observer semantics. Bind missing old metadata through independently
verified provenance in a separate immutable attestation or recapture; do not
guess identities. Candidate metadata also binds hardware source/settings.

Reject incompatible identities, truncated captures, empty selected intervals,
missing/reordered events, and completion metadata not matching the CSV digest.
The unchanged original oracle need not be recaptured; initial readiness needs
a fresh current candidate replay.

CPU-only increments may reuse accepted independent candidate synchronization
and fallback evidence when hardware, reference execution, input/reset, observer,
comparator, build features and settings are identical. Record the reused run ID
and verified unchanged dependencies. Shared-path changes require fresh cold
fallback parity and independent candidate synchronization. When uncertain,
rerun. Record documentation-only edits as irrelevant only if executable/build
inputs are unchanged.

## Readiness procedure

1. Allocate a run and capture the current maintained state. Verify dependencies,
   tools, ROM/movie/profile and baseline provenance; run `just doctor`. Missing
   required inputs produce an error brief naming the exact user step. Find
   workspace facts rather than asking the user to look them up.
2. Implement per-key coverage, uniform verdicts, replay/controls, provenance and
   recovery. Build on the current frame comparator and resumable step runner.
   Report shared-backend parity separately from independent hardware/input
   synchronization. Preserve all pre-existing implementation changes.
3. Run `just check`, `just lint`, and complete available affected-opcode cases
   for currently enabled patterns through the real emitter/step path. Apply
   the pattern-test contract in the iteration spec. Current patterns: immediate
   LDA/LDX, REP/SEP, absolute STA, NOP, DEX. An inactive corpus is not evidence;
   a failure affecting an enabled pattern blocks readiness. Unsupported ADC
   overflow failures remain explicit debt.
4. Run the automated feedback demonstrations below and validate paired caches
   against cold execution. Isolate deliberately faulty fixtures from normal
   generated code and reviewed config.
5. Cold replay fallback-only subject against reference for all 18,000 frames;
   require CPU/clocks, complete WRAM and framebuffer parity. Regenerate/build
   the current compiled subject and cold replay the same domain, requiring
   parity and actual generated execution. Establish baseline per-key coverage;
   existing unexecuted variants remain unvalidated inventory. Later additions
   must pass the strict changed-key coverage gate.
6. Cold replay the current reference and strictly compare independent checkpoints
   through 18,000 with the verified original capture. A fresh exact pass is
   mandatory. Diagnose any failure before issuing readiness; do not shrink
   the agreed domain.
7. Record timings, supported patterns, executed keys, fallback share and debt.
   Finalize verdict and brief. Update the accepted pointer only if all mandatory
   checks pass.

## Required feedback demonstrations

- Correct execution produces a fresh identity-bound pass.
- An executed incorrect compiled instruction fails and names its altered entry
  at the first observable state/bus-timing disagreement.
- A transient incorrect effect restored later in the frame is caught by focused
  instruction/bus comparison even when frame-end bytes match.
- An added compiled key that never executes yields uncovered despite old
  compiled entries executing. Empty compilation cannot pass either.
- Preflight/build failure, interrupted/truncated captures, incompatible identity
  and stale prior verdicts cannot yield acceptance.
- A valid paired checkpoint reproduces the corresponding cold execution trace
  and outcome; incompatible/corrupt checkpoints are rejected and rebuilt.

## Diagnostic and checkpoint contract

Focused replay aligns both sides at corresponding instruction boundaries and
compares decoded CPU, interrupt/wait/stop state, CPU/master clocks, ordered bus
fetch/read/write/implied cycles and relevant mutable hardware state. Include
pending state affecting subsequent execution, DMA/HDMA, refresh and frame/input
boundaries. Observation is passive: do not consume interrupts or clock hardware
through diagnostic reads. Compare effects before later instructions cancel them.

Retain the first observable mismatch, preceding matching boundary, input digest/
index/pad, both PC/opcode/MX values, compiled key, decoded before/after differences,
clock/scanline position and differing bus event/state. Include up to 64 preceding
instructions per side with bus events, paired snapshots and exact rerun commands.
Use available symbols to decode WRAM; absent symbols cannot block diagnosis.

A checkpoint captures complete CPU/hardware and subject pending/dispatch state,
input position and relevant counters. Bind it to ROM/movie/source/profile/
serializer identities and its own digest. Do not interchange core save states.
Validate restoration against at least the next complete cold-executed frame,
including trace and ending state. Cross-candidate reuse must precede affected
execution and have verified equivalent replay history; otherwise build a new
candidate cache cold. Do not simply relabel old state as compatible.

Failure controls replay the accepted predecessor and the candidate with newly
enabled/changed compiled keys disabled over identical input/intervals. Passing
controls support increment attribution, not a specific root cause. Failing
controls route diagnosis to shared paths, prior assumptions, input or cache/
harness validity. Original-core failures need independent observations and
hardware probes; two shared-backend passes do not resolve them.

## Timing, stopping and brief

Target warm focused replay within 60 seconds and each 18,000-frame cold replay
within 20 minutes. Report generation/build, cache creation, instruction/test,
total-suite and diagnostic times separately. These are feedback targets;
completed passes with overruns report the overrun, whereas actual timeouts are
incomplete and block acceptance. Optimize without weakening checks or reducing
the accepted domain.

Continue routine fixes autonomously. Missing required input or a decision beyond
this design ends with a brief. After three successive diagnostic experiments
without new discriminating evidence, preserve the candidate/accepted state and
present concrete choices. This escalation never implies acceptance.

The brief states ready/not ready, exact source/domain, changes and evidence
links, coverage/fallback share, timing and debt. On failure give the earliest
disagreement, evidence-supported cause or next experiment and specific decision
needed. Link complete logs rather than placing them in the brief.

Relevant machinery/input changes expire affected readiness checks. This spec
session implements no checks and cannot itself issue a readiness verdict.
