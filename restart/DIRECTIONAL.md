# Directional door scrolling

Ticket10 adds the complete 13-routine directional motion subsystem: upward
correction, the four setup paths, four movement paths, private bookkeeping and
the IRQ-follow root. Generic IRQ timing/register writes and the room-transition
coordinator remain separate. Three setup-math helpers are explicit C services;
the fourth service calls the existing installed Rust scrolling child.

The same child can execute inside coordinator setup or a later real IRQ.
Each parent publishes its current WRAM synchronously and resumes only after
the child commits. Right's terminal movement invokes the child separately at
counters63 and64. Vertical first calls expose temporary coordinates and restore
only the four scalar fields saved by C, retaining child block/queue changes.
The IRQ caller owns the completed-bit guard; setup may retain an earlier flag.

The existing `sm-pilot` gains `--scenario door-scrolling`, using the same
9,380 inputs and all four previous consumers' material checks. Its native
observers identify directional roots, private routines, services, scroll children
and before/after motion state. The first new C reference confirms the
predicted two setups,126 IRQ roots,128 right movement calls and130 children.
Candidate edits reuse that frozen trace. Left/down/up and selective-snap
edge cases remain focused evidence rather than claims from the rightward route.

Frozen integration `4d6c541` passes its first C reference and first Rust
candidate without a gameplay correction. All 9,380 observations match over
complete WRAM, passive external state and ordered effects. The new C reference
also matches the previous coordinator reference's WRAM/external hashes at every
step; the new effects have their own native identity.

All 128 scheduled directional entries complete in Rust with zero C entries or
refusals: two setup roots and126 IRQ-follow roots. Six of13 routines
are reached, including128 right movements, two calls to each bookkeeping
helper and130 actual scrolling children. Three-layer coordinator→directional→
scrolling composition and standalone IRQ composition both execute materially.

| Event | DF45 → DF8D | DF8D → DFD7 |
| --- | ---: | ---: |
| Setup and first movement | 8,748 | 9,057 |
| Terminal IRQ / completion bit | 8,812 | 9,121 |
| Outstanding NMI wait completes | 8,813 | 9,122 |
| State8 resumes | 8,831 | 9,140 |
| Input moves Samus | 8,832 | 9,141 |

The second setup actually enters with8000 still set and preserves it. Both
terminal movements call the child at63 then64. The route establishes final
layer1==destination and continuing layer2 motion; because normal motion already
reaches the destination, the selective snap assignment itself has separate
focused coverage with deliberately different coordinates.

The forced warm rebuild plus comparison costs15.689 seconds, including0.601
seconds for Cargo (which recompiles the touched, byte-identical gameplay crate).
The initial C capture costs30.956 seconds; it is reused across candidate edits.
The normal CLI takes9.797 seconds, has zero CPU fallback and matches both final
WRAM and PNG. There are36 passing affected integration tests and a passing
strict workspace/all-target Clippy check. The native worker separately passed
the prepared-package integrity test,132 C/refusal cases, seven positive modes
and twelve expected fatal modes. These controls complement13 authored gameplay
tests; they do not turn unreached branches into live parity.

Retained source, binaries, native package, runs and logs are under
`out/door-scrolling/`. From `restart/`, reuse the fixed reference:

```sh
DIRECTIONAL_LIBRARY="$PWD/out/door-scrolling/native/311af8f3ea2746d0298cd202a1b1005ed8963c459bd71980b9d91421f85ed9fe/libsm_native.so"
./out/door-scrolling/bin/sm-pilot dev --scenario door-scrolling \
  --library "$DIRECTIONAL_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/door-scrolling/runs/reference-01 --out out/door-scrolling/runs/my-edit
```

Use fresh output directories. The deliberate fault clears the completion bit
after one actual terminal IRQ movement; the comparator and C trace stay fixed.
`fault-01` rejects it at the first divergence, step8,812 in WRAM (14.417 s).
After this documentation changed, `docs-01` reused the passing result in0.191 s
with zero game processes. Both integrated worker worktrees are retired;
post-cleanup native/trace/result authentication also reuses the result without
game execution. Retention details are in `evidence/cleanup.md`.
The normal CLI installs all five consumers. No original-ROM or full-game Rust
ownership claim follows from this scoped recovered-C comparison.
