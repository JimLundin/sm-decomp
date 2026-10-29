---
name: sm-decomp-build
description: Compile and qualify a pinned sm-decomp library or verifier DLL using supported host commands, existing results and precise compile evidence.
---

# Qualify a build

Read [PLAN](../../../docs/PLAN.md) first, then [REVIEW](../../../docs/REVIEW.md)'s
Milestone plan at each item's start. Phase 0 verification-loop work lands first;
its exit is two consecutive green scheduled full-corpus runs. REVIEW sets the
active phase; PLAN owns checks, resources, Git and completion. User instructions win.

Use the assigned checkout/source pin, stages, original handles and next recipient.
Read [iteration](../../../docs/ITERATION.md) and
[native dependencies](../../../docs/NATIVE_DEPENDENCIES.md) for reached commands.

1. Collect completed stages from original handles instead of rebuilding them.
   Preserve frozen checked source and loaded DLLs. Bind compiler/toolchain/profile,
   source/native inputs, output directory and final artifact bytes to the result.
   Record persisted layout identity separately; matching it does not transfer
   compilation proof from changed source or native inputs.
2. Use only root-admitted stages on authorized hosts. Launch prepared conditional
   successors immediately after prior terminal release, exact guards and fresh fit
   under [resources](../sm-decomp-resources/SKILL.md), without another root ACK.
   At terminal observation, run the existing collector then the prepared
   successor launcher sequentially in one functions.exec call; retain both
   helpers' terminal, pin, qualification and fresh resource checks.
   Notify root at START; preserve ceilings, independent outputs and source/native/
   cache guards. Use supported Windows; Linux needs explicit user restart.
3. If DLL growth blocks admission, use PLAN's admissible normal-library allowance
   only when no successful proof transfers to the unchanged pinned inputs.
   Otherwise reuse that result; final DLL/runtime qualification remains pending.
4. Run PLAN's required format, warnings-as-errors library and DLL stages through the
   existing verifier. Select affected verifier/host compile checks from actual
   mod/#[path] declarations and public exports; library proof transfers only to
   code compiled by its checked crate/cfg. Existing player_detached supplies its
   affected detached reach. Metadata does not execute declared unit bodies.
   Keep normal compiled-input transfer separate from host visibility proof;
   unchanged normal inputs do not qualify a verifier import. For target-package
   migration, compile the actual headless package entry without compat dependencies.
5. Enforce sm/Cargo.toml warnings='deny' on every sm Cargo target; preserve
   dependency flags. Direct/detached rustc needs explicit -D warnings. Fix actual
   diagnostics; blanket allowances and historical warning-bearing builds do not
   pass. Bind reuse to the strict package/options; denied/missing artifacts fail.
6. Send minimal DLL/source/native pins, stages/units/gaps and original handles
   to the assigned replay owner and primary before extended receipts. Record actual
   START/EXIT; notify root on terminal compilation for release.

When reporting CI, select [CI](../../../.github/workflows/ci.yml) for the exact
pinned commit and verify run head_sha and required job conclusions. Other workflows
do not establish CI success. Continue independent implementation and direct READY
handoffs while CI executes. Compile/type, unit execution and runtime acceptance
are separate evidence; a build role handoff is not project completion.
