# Enemy / room / player detached composition

Install the Rust files and mutations.py over the maintained
sm/src/verification/player_detached in the complete integrated enemy-session
candidate. Existing player/room/animation/palette checks remain. DESIGN.md records
the independently approved cases and separates native-runtime work.

One SessionAccess implementation returns World.player for all three gameplay Hosts.
The new enemy wrapper imports actual candidate algorithms. Its content provider
uses actual generic EprojScript<RomImage> and actual LoadedRecord/NativeRecord;
verify_probe.rs is also the actual source. No Game, RAM codec, second inventory or
second enemy pool is included. Unsupported actor/backend operations panic.

New detached cases:

- 1050 actual EyeDoor room command, enemy setup, post-spawn sound observation and
  same-scheduler live selection plus lower-slot iteration.
- 1051 actual EyeDoorSmoke final RNG effect redirects selection inside enemy setup;
  same room scheduler resumes its captured script with live selected.
- 1052 actual player beam, enemy contact marks its real slot, actual player motion
  consumes the mark; plasma mask0008 sibling continues actual flight.
- 1053 actual drop body contact awards Inventory before sound and captured-slot
  dismissal, then actual missile consumes it; normal and capped ammunition cases.
- 1054 explicit unprepared DamagePlayer failure with actual contact's committed
  invincibility/knockback/hit-list prefix and distinct projectile protection.
  This does not simulate or validate the native damage routine.
- 1058 actual spawn_drop and forced missile selection consume shared spawn point
  and drop metadata; actual collection and missile firing consume the same owner.
  Native storage transfer itself is not tested by this detached check.

1055–1057 belong to torizo_review's real Runtime/native carrier. Labels are detached
case identifiers, not claims that corresponding DLL exports were allocated.

Mutation runner retains the four existing mutants and adds three specific-assertion
checks: restore into a private inventory copy, discard the shared shot-direction
mark, and use stale room selected after nested spawn. The unchanged baseline must execute successfully before any mutant. New
mutants must fail their named enemy check equality assertion, not an unrelated
panic. Survived mutants, compile errors and execution errors remain distinct.
Mutation execution retains the existing CI-only policy.

Integrator commands after complete production overlay (not executed by worker):

    rustc --edition=2024 -O sm/src/verification/player_detached/main.rs -o <isolated>/composed.exe
    <isolated>/composed.exe
    # Existing CI-only mutation run:
    python sm/src/verification/player_detached/mutations.py

Status: source binding complete against current A-D replacement files. Rust files
parsed/formatted with skip_children and Python runner syntax parsed. All three
Host method sets match, and each new mutation target occurs exactly once in its
actual source. No compiler, execution or mutation verdict. Integrator must install
all production changes, including shared slope helpers in projectile_geometry,
before compiling this harness. Recheck against the final assembled candidate.
