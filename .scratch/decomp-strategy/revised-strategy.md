# Revised development strategy

Status: proposed from repository and execution evidence
Owner: recovery coordinator

The next milestone is an independently running mixed port that reaches
the end of Ceres with correctly consumed controller inputs. Replaying the
original movie's number of video frames is not a gameplay milestone: the current
129,712-input run still ends in Ceres, game state8/roomDF8D. The existing causal
input adapter has matched 1,097 completed updates through original frame1600;
extend that working seam toward Ceres escape, preserving failures and explicit
unproved hardware/APU details. Follow with normal play, save/load and credits.

## Change the work loop

1. Reconcile all existing source and evidence before selecting more translations.
   Publish implemented, historically verified, current-source checked and live
   extent without converting declaration counts into acceptance. Show gameplay
   checkpoints, remaining C ownership and input-adapter reach beside code extent.
2. Build a reusable game host interface using the existing canonical WRAM and
   typed external effects. Prove it with the already translated Ridley and Samus
   modules rather than writing another bespoke adapter for a new subsystem.
   Keep hardware and temporary C ownership explicit. Preserve existing integer,
   alias, ordering and error semantics.
3. Give each translation agent one coherent module and a concrete gameplay
   outcome. The coordinator integrates disjoint changes into one runnable port.
   A module's external interface should hide its internal dispatch/callback
   details; expose dependencies once through the shared host.
4. During edits use focused actual-C fixtures and the shortest replay that
   materially exercises the changed behavior. Use two subjects for diagnosis;
   reserve four fresh subjects and full retained evidence for an integration
   milestone. Keep the current acceptance contract until a replacement is
   implemented and tested; diagnostic passes never silently grant acceptance.
5. Separate gameplay, scheduler/input, hardware adapter and evidence-reader
   identities. Reuse immutable oracle fixtures and baseline evidence when their
   actual behavioral inputs are unchanged. A progress HTML/CLI edit should not
   force every gameplay family to re-execute solely because main.rs changed.
6. Bound evidence costs: compact ordered event streams, material captures and
   frame summaries for development; complete exact evidence at milestones.
   Authenticate newly created objects and publication once, then reauthenticate
   after cleanup. Preserve old seals and reader contracts; optimize the new
   contract through a measured regression instead of rewriting old receipts.

## First bounded experiment

Use the existing Ceres/Samus host and input code with the translated modules.
Create one stable host interface with an actual-C adapter and a recording
adapter, and a focused diagnostic command. Success requires an existing module
to execute unchanged through the new interface, refusal/error/order controls to
pass, actual-C state
and ordered effects to match, and no new per-routine ABI template. Measure
edit-to-result time and retained bytes against the existing loop. A failed
experiment is retained and informs seam placement; it does not expand gameplay
scope or relax acceptance.

In parallel, advance the existing causal controller seam to a recognizable
Ceres completion checkpoint. Treat the original-reference comparison and C/Rust
translation comparison as separate checks. This addresses a playable outcome
while the integration interface improves translation throughput.

The concrete starting point is ticket 78's terminal-completion driver,
`docs/DECOMP_PREFIX.md`, and the maintained Rust `decomp-prefix` command. Its
first diagnostic extension should identify the earliest controller/scheduler
divergence beyond original frame 1,600, with at most 60 seconds for the initial
experiment. A named Ceres escape transition from ordinary reset is the milestone;
the matching semantic prefix alone retains its existing nonaccepting status.
The exact capture extent follows measured run cost and the observed transition,
without assuming raw input length carries the original movie route.

The host experiment starts with the complete `samus_projectile_frame` interface
and its existing PLM/HDMA requirements, then checks a second existing consumer
such as room scrolling. A shared integration owner controls host source and
registration while input work owns its independent scheduler files. Use the
assignment workflow to record concrete writable files and bounded native gates
before any implementation launches. This report defines the next work; those
new assignments and broad Ridley gates remain held under the user's steering.

## Basis and limits

The reconciled registry reports 1,460 translated, 924 verified, 38 integrated
and 121 live-exercised native definitions out of 6,053. Each stage is independent
and deduplicated by native symbol. Historical verification covers 911 roots;
38 have integrated checkout verification, with overlaps. The 121 live symbols
combine 106 historical-context symbols with 26 checkout-observed symbols, with
eleven measured internal opcodes/initializers shared between contexts. The initial
historical set contained 95 direct roots. These are bounded source/evidence counts,
not a gameplay completion percentage.

The reviewed source map covers all 44 Rust port files and 175 maintained
manifests. It maps 1,452 native definitions, including 618 without explicit
manifest roots, and records 309 uncertain or forwarding references. A separate
Ridley caller review identifies eight additional inlined native implementations
with explicit caller-domain limits. This is
implementation correspondence with explicit bounded domains and delegated
effects; it does not establish equivalent behavior for every branch. The source
contains 19,453 Rust lines, while the pipeline contains 38,937 Rust lines.
These raw counts describe repository shape, not completion or effort.

The recovered whole Ridley candidate has a 220-definition reachable scope.
The extractor initially identified 26 closure candidates without an owned
mapping. The caller review substantiates eight inlined implementations;
native services remain explicitly delegated to C and the other gaps retain
their documented uncertainty. Its
38,000-input diagnostic passes four subjects and material captures, but full
whole-subsystem live acceptance remains pending. User steering holds that new
full-route gate during strategy reconsideration. Recovery jobs have completed
their cleanup and authentication; the candidate remains available for next work.

Measured audit details: `architecture-audit.md`, `source-coverage-audit.md`,
`ridley-inlined-source-alias-review.md`, `coverage-audit.md`,
`../decomp-port/ridley-live-finish-handoff-20261013.md` and
`../decomp-port/recovery-integrated-final-authentication-summary.toml`.
