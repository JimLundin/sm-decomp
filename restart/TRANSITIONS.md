# Rust room transitions

Ticket09 replaces the complete scheduled door-transition coordinator: state9,
state10 and state11, its private dispatcher/fifteen phases and all eight
background commands. The native outer scheduler, NMI and directional IRQ
movement remain unchanged. Named loading, actor and hardware operations remain
C services and may synchronously call the existing Rust gameplay consumers.

The three scheduled entries return ownership separately from the original
coroutine byte. Pending results commit the continuation and return to the real
outer scheduler. On resume the outstanding NMI wait runs even if the IRQ has
already set the completion bit; PLM continuation cannot repeat room loading.
Private calls and the background stream stay within one gameplay module.

The existing `sm-pilot` runner gains `--scenario room-transition`. It uses the
same9,380 combined inputs and retains projectile, scrolling and Ceres-door
material requirements. The new evidence observes the two rightward crossings,
once-per-room loading, background decompression/two nonempty transfers,
pending/resumed waits, phase progression and return to input-controlled gameplay.
Exact53-service and fixed27-routine observations are part of the new native
recipe. They do not create a second scheduler or additional gameplay calls.

The integrated candidate at `9ee6419` matches all 9,380 observations against
one C reference captured with the new native recipe. The reference also has
identical per-step WRAM and passive external state to the previous door
reference. Only the added observations change its effects protocol.

All 215 scheduled coordinator entries complete in Rust, with zero C entries or
refusals. State9 executes twice; state11 executes 213 times. State10 executes
privately once per crossing and is never manufactured as a scheduled call.
There are 126 pending returns and 126 resumed entries. Twenty-one of the 27
observed routines are reached, including background commands 4/2/2/0.

| Transition event | DF45 → DF8D | DF8D → DFD7 |
| --- | ---: | ---: |
| Collision / state9 | 8,726 | 9,030 |
| First pending NMI wait | 8,750 | 9,059 |
| Outstanding wait completes after IRQ | 8,813 | 9,122 |
| State8 resumes | 8,831 | 9,140 |
| Input moves Samus | 8,832 | 9,141 |

The first Rust attempt stopped before comparison because it required exactly
3 MiB of ROM. The native loader expands the cartridge to a 4 MiB view. The
gameplay-only correction accepts that expansion and rejects truncated images;
all 15 focused tests pass with runtime-sized fixtures. The failed `dev-01`
is retained, and `dev-02` reused the same authenticated C reference unchanged.

The warm `dev` invocation took 15.356 seconds, including a 0.082-second cached
build; the preceding source rebuild took 0.61 seconds. The normal CLI installs
all four Rust consumers, finishes in 10.523 seconds with no CPU fallback and
produces identical final WRAM and PNG to both reference and candidate.
Focused native controls also verify raw coroutine returns, pending copyback,
child composition and fatal rejection without C retry.

Retained sources, binaries, native package, runs and control logs are under
`out/room-transition/`. From `restart/`, reuse the reference for candidate edits:

```sh
TRANSITION_LIBRARY="$PWD/out/room-transition/native/72faf44a8d34103f59312f2b76436e62926446fbe549237a0122a3b3585398d2/libsm_native.so"
./out/room-transition/bin-rom-fix/sm-pilot dev --scenario room-transition \
  --library "$TRANSITION_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/room-transition/runs/reference-01 --out out/room-transition/runs/my-edit
```

Use fresh output directories. A deliberate fault changes the first actual
pending NMI-service result to completed, so subsequent Rust control flow is
wrong; the reference and comparator remain unchanged. `fault-01` detects the
first divergence at step 8,750 in passive external state (13.866 seconds).
After a documentation-only change, `docs-01` reuses `dev-02` in 0.189 seconds
with zero game processes. Both worker worktrees have been retired; post-cleanup
native package, trace and result authentication also reuse the result without
game execution. See `evidence/cleanup.md` for retained dependencies.

Elevators, other directions, message-box suspension, alternate room states and
other background commands remain outside this live route. Focused branch tests
are separate evidence. No original-ROM or full-game Rust ownership follows from
the recovered-C comparison.
