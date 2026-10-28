---
name: sm-decomp-build
description: Compile and qualify a pinned sm-decomp library or verifier DLL using supported host commands, existing results and precise compile evidence.
---

# Qualify a build

Read [PLAN](../../../docs/PLAN.md) first, then [REVIEW](../../../docs/REVIEW.md)'s
Milestone plan at each item's start. Direct user instructions win. REVIEW sets
priorities; PLAN owns required checks, resources, Git and completion.

Use the assigned checkout/source pin, stage scope, root-admitted budget, original
handles and next recipient. Read [iteration](../../../docs/ITERATION.md) and
[native dependencies](../../../docs/NATIVE_DEPENDENCIES.md) for reached commands.

1. Collect completed stages from original handles instead of rebuilding them.
   Preserve frozen checked source and loaded DLLs. Bind compiler/toolchain/profile,
   source/native inputs, output directory and final artifact bytes to the result.
2. Execute only on authorized hosts with the root's admitted workflow allowance.
   Use independent output directories for concurrent jobs. Linux remains stopped
   unless the user explicitly restarts it; keep existing source/native/cache guards
   and use the supported local Windows path.
3. If DLL growth blocks admission, use PLAN's admissible normal-library allowance
   only when no successful proof transfers to the unchanged pinned inputs.
   Otherwise reuse that result; final DLL/runtime qualification remains pending.
4. Run PLAN's required format, library/no-new-warning and DLL stages through the
   existing verifier. Select affected verifier/host compile checks from actual
   mod/#[path] declarations and public exports; library proof transfers only to
   code compiled by its checked crate/cfg. Existing player_detached supplies its
   affected detached reach. Metadata does not execute declared unit bodies.
5. Compare warning identities against the original result with the same compiler,
   options, native inputs and profile. Unchanged totals can hide new warnings.
   A warning increase, denied launch or missing artifact needs a correction or
   supported execution route; it is not a passing stage.
6. Send DLL/source/native identity, completed stages, executed units, gaps and
   original handles directly to replay and the primary. Record actual START/EXIT
   when known; notify the root at terminal compilation to release admission.

When reporting CI, select [CI](../../../.github/workflows/ci.yml) for the exact
pinned commit and verify run head_sha and required job conclusions. Other workflows
do not establish CI success. Continue independent implementation and direct READY
handoffs while CI executes. Compile/type, unit execution and runtime acceptance
are separate evidence; a build role handoff is not project completion.
