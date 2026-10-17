# Ordinary Rio

Rust translation of all ten ordinary Rio definitions: `EnemyInstr_Rio_Instr_1`,
`Rio_Init`, `Rio_Main`, `CallRioFunc`, and `Rio_1` through `Rio_6`.
Norfair variants are outside this actor scope.

Primary recovered C is `snesrev/sm` revision
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`, authenticated archive SHA256
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.
Bodies are `src/sm_a2.c:2247-2363`; owned dependency behavior comes from
`sm_a0.c:2997` (screen) and `:3236` (distance), `sm_80.c:22` (RNG),
`:763-793` (SFX2 queue), and `sm_94.c:2106` (literal signed sine table).
The speed words are read from patched runtime ROM at file offsets `0x113bbb`
and `0x113bbf` (SNES A2:BBBB/A2:BBBF). Runtime ROM is caller supplied; no ROM
is included.

`execute` accepts full 128 KiB canonical WRAM. Typed entries use numeric
discriminants 0 through 9. `arg` is the bank-A2 continuation for `Instruction`,
the u16 animation list for `SelectAnimation`, or the exact bank-A2 address for
`CallRioFunc`. `Entry::address()` deliberately returns `None` for that dynamic
wrapper. State1 through State5 use explicit `k`. All other roots and the animation,
event and screen helpers use WRAM `cur_enemy_index` exactly as recovered C does.

`Ok(None)` refuses unchanged for unsupported dispatch states or unavailable
required services. Bad WRAM/slot/continuation/speed-table/sound-ring inputs error
before owned effects when their branches need those inputs. Main advances RNG
only after preflight. Native failures after admission return errors; a caller
must not fall back to C. State bodies cache their actor base, reread fields after
each synchronous full-WRAM service copyback, and leave current-slot helper
lookups dynamic. A native angle outside 0..255 is a terminal service violation.

Horizontal movement, vertical movement and angle calculation use the `Services`
trait. A production adapter must provide side-effect-free admission for the
complete incoming invocation, synchronous full-WRAM copyback, and terminal
failure propagation. Geometry, terrain/PLM effects, scheduler, generic
instructions, contact, damage, death, frozen handling and rendering remain
native dependencies.

The `fixture-support` feature exposes `body_fixtures()` and scripted services.
The shared corpus covers every root/state, strict distance and screen boundaries,
ring/gating behavior, random carry, signed sine extrema, event/deceleration
priority, collision ordering, explicit/current index differences, actor pointer
caching and full-WRAM rereads. Scripts contain response values and byte patches;
they neither implement terrain nor establish natural live encounters. Unit
controls assert semantics and atomic admission/failure behavior; independent
compiled-C comparison is performed by the integration owner.

`service_wram_matches` supplies exact full-WRAM validation for Rust writes before
a native service: Main RNG, State4 horizontal bounce, State5 pursuit vector, or
unchanged state. Its selector must come from the adapter's admitted root and
observed service sequence. The adapter must separately validate terrain, service
order, arguments, generation, allocation and final commit. This helper is not yet
wired into native production; it grants no terrain or integration acceptance.

`ServiceProtocol` binds the incoming root and actor slot to ordered native
requests, exact signed movement arguments and those pre-service WRAM checks.
The native adapter owns this object and supplies actual service results plus
full copyback. A rejected request or completion permanently poisons it. It
handles State3 collision short circuit, State4 bounce and State5 angle/pursuit;
it also captures Main RNG before the first service. `services_complete()` proves
only service completion. Terrain admission, pointer/generation binding and final
actor commit validation remain separate native host obligations. Production
wiring is pending. `validate_commit` replays only linked Rust actor writes using
retained native results and full copybacks; it never repeats a native service.
It compares final WRAM and output and rejects missing services, unrelated final
writes and duplicate commit. The host must still bind immutable ROM, prevent
reentry and publish only after validation. This checks the linked actor contract;
independent actual-C verification remains required.

The native host registers `execute_rio` and `RioTransport`. The execution wrapper
binds private scratch, runs the actor through protocol-checked native requests,
and validates final state before transport commit. Unchanged refusal has no
service/commit; errors remain terminal. `Game::install_rio()` now installs the
concrete native transport at ordinary AI and instruction scheduler roots.
`rio_counters()` exposes roots, native/Rust/refused/completed execution, terrain
refusal and service counters. Other gameplay remains owned by existing consumers.

Native preparation installs Rio after Ripper, extends sibling fences, and seals
the new C/Rust recipe files. Terrain admission bounds both movements to a finite
rectangle with supported block extensions and spike setup, before Main RNG or
actor writes. States outside it retain unchanged C fallback. This admission and
the new scheduler/FFI wiring have only compile/syntax evidence: complete native
build, actual-C/admission/fatal/composition tests and live encounter remain pending.
The runtime CLI selects this installation explicitly with `--rio rust`; its
default `--rio c` is independent of `--gameplay`. Results retain source snapshots
and distinct ownership/service counters. CLI compilation and live execution remain
unverified because dependency build-script execution is blocked on this host.
The native fixture admission/invocation exports now cover all ten typed roots
for focused controls; they have syntax evidence only.

The current Windows candidate passes all 22 semantic, service-write, protocol
and commit tests, all-target `cargo check` with fixture support, and focused
formatting checks. See ticket 29 for retained commands and source hashes.
Actual-C comparison, Clippy, native production integration and live execution
remain pending; the enforced host policy still blocks verifier dependency build
scripts. The passing test binary does not prove those other executables can run.
