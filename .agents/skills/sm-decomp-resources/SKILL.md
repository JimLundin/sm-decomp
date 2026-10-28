---
name: sm-decomp-resources
description: Admit, monitor, release or clean up bounded sm-decomp source/build/replay workflows in the primary root resource ledger.
---

# Manage one resource ledger

Read [PLAN](../../../docs/PLAN.md) first, then [REVIEW](../../../docs/REVIEW.md)'s
Milestone plan at each item's start. Direct user instructions win. REVIEW sets
priorities; PLAN owns required checks, resources, Git and completion.

The primary root owns one ledger. Grant ready sequences and conditional successors
upfront with exact source/native/output pins, per-stage RAM/commitment/disk ceilings,
terminal checks and fresh-fit conditions. Workers use only granted stage allowances;
successors require prior terminal release and fresh fit against all reservations.
Use current PLAN allowances, including the uncapped direct-DLL growth.

1. Sample actual RAM, CPU, paging and disk before admission and during artifact
   jobs. Budget launchers and Cargo/rustc descendants as one owned workflow;
   count every admitted workflow's remaining growth once, plus copies, captures,
   caches and retained output. Leave PLAN's RAM/disk reserves after all budgets.
   Track physical remaining growth and family private commitment separately.
   Choose caps for the actual workload from comparable original command,
   toolchain, profile, cache and source evidence instead of reusing a native
   helper's thresholds. Label host RAM drawdown separately from compiler RSS.
2. Preserve live jobs, frozen sources and loaded DLLs. Isolate concurrent outputs.
   Release reservations promptly only after authoritative terminal evidence.
   A quiet parent, monitor error or observation timeout does not prove its child
   stopped; retain uncertainty and inspect owned descendants.
3. On a reserve breach, stop the newest producer through its owned executor and
   retain lightweight readers. Preserve actual handles and confirm termination
   before releasing its allowance; cleanup/monitor failures do not imply success.
   Admit useful overlap only when measured growth and the shared ledger fit.
   Record every stop's exact cause, measured value/limit, UTC time and reserve
   state. A cap-stopped curve leaves the uncapped peak unproved. Compare corrected
   throughput only after the qualifier's actual result.
4. If DLL admission is blocked, keep core source/review work supplied. An admissible
   normal-library checkpoint is useful only if its unchanged inputs lack an
   accepted result; otherwise transfer the existing proof.
5. Resolve reparse/junction and absolute target paths before sizing, budgeting or
   cleanup. Count physical targets once and exclude shared caches from per-worker
   reclaim. Verify that deletions stay in the intended owned workspace and that
   each artifact is unneeded. Prefer exact nonrecursive compiler-file allowlists.
6. Preserve other workers' source, results, checkpoints, loaded DLLs, caches,
   locks and directories. Follow PLAN's disk runway/copy limits and cleanup
   ownership. Compression needs unchanged hashes. Report a policy rejection;
   do not retry through another tool or use tiny gains to bypass the reserve.

Apply current user host/cache authorization over older plan wording; a stopped
host needs explicit user restart. Use authorized caches bound to toolchain/profile
and source/native/output identities. Send root the measured budgets, actual
START/EXIT, live/terminal handles, retained bytes and remaining uncertainty.
