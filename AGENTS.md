# Agent instructions

Read `docs/PLAN.md` first. It replaces every earlier workflow, skill, ticket
and evidence rule. Ignore `.scratch/`, readiness gates, acceptance pointers,
translation scopes and anything else from before 2026-10-17; that project
is archived in the branch `archive/pre-transpile-2026-10-17`.

Then read `docs/REVIEW-2026-10-24.md`. Its directives (summarized at the top
of `docs/PLAN.md`) take priority over the existing work order.

Before changing subsystem boundaries, state ownership, or backend architecture,
read `docs/target-architecture/README.md` for the accepted destination and
Windows migration policy. Current work and checks still follow `docs/PLAN.md`.

## The loop

1. Take the earliest unfinished item in `docs/PLAN.md`.
2. Make one focused change in `sm/`.
3. Select and complete checks using `docs/PLAN.md`'s **Validation strategy**.
   No new failing frames.
4. `sh tools/progress.sh`; numbers must not go up without a reason.
5. Commit to `main` with a message that says what changed and the verify
   result. Update `docs/PLAN.md` when an item is done or the plan changes.

## Rules

- Never reserve or hold Git commit locks. Follow the Git concurrency rule
  at the top of `docs/PLAN.md` before any Git write.
- `sm/src` is the source of truth. There is no C to consult or regenerate
  from. When the meaning of code is unclear, use the oracle and the
  `ram_map.rs` names, and the bank logs at https://patrickjohnston.org/bank/.
- Preserve behaviour exactly, quirks included. The oracle decides, not taste.
- Keep `g_ram` byte-compatible with the SNES layout.
- Small commits. A refactor that touches many files is fine if it is one
  mechanical idea; mixing ideas in one commit is not.
- Do not build process tooling, evidence stores or dashboards. If the loop
  above is too slow or too weak, say so in `docs/PLAN.md` and fix the verify
  run itself.
- Never commit ROMs, save states or anything derived from the ROM.
- When stuck for more than an hour of work on one item, write down what you
  found in `docs/PLAN.md` under that item and stop.
