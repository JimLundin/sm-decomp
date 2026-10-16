# Persistence runtime installation: source handoff

Prepared source proposal; no implementation claim. Read issue24 and its frozen
`../persistence-interface.md`. Their accepted 30body/1IO/3chain/9admission/15fatal
evidence remains authoritative; this note does not repeat codec or ROM research.
Current Rust paths below are relative to `/home/jim/sm-decomp-restart/restart/`.
Recovered C caller citations use retained directory
`out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.

## Existing interception and smallest production module

`native-host/src/persistence_prepare.rs:11–46` already replaces the ordinary
`SaveToSram` and `LoadFromSram` definitions in `sm_81.c` with dispatch/original/
done wrappers. Load uses tagged native output10000|byte; callers still receive
the byte. Pack/Unpack wrappers bypass dispatch only as validated local native
Save/Load helpers. `prepare.rs:423,443` applies this recipe and compiles
`restart_persistence.c`. No new native seam or dispatcher patch is needed.

The missing production bridge already exists as an example implementation:
`native-host/examples/persistence-host-control.rs:65–82`. Its `Disk` adapter
forwards `Services::available` and `write_sram` to the issued
`PersistenceServices`; `install` registers a callback that maps native Entry
to codec Entry, invokes `sm_persistence::execute`, maps Done/Loaded exactly, and
maps None to Refused. Errors propagate with `?`, never C retry.

Assign a worker **only new** `src/persistence_runtime.rs` and optionally
`tests/persistence_runtime.rs`. Recommended public API:
`pub fn install(game: &mut sm_native_host::Game) -> Result<(), String>`.
Keep the small service adapter and typed mapping private. Explicit enum matches
are straightforward; the example's ordinal mapping is also backed by the frozen
identical four-entry order. Use the existing Game registration, no raw FFI,
unsafe Rust, copied SRAM owner, observer framework, or duplicate codec.

`native-host/src/lib.rs:549–580` already exports `sram`, `persistence_counters`,
`set_persistence_hook`, and `clear_persistence_hook`; the callback accepts
`(PersistenceEntry,u16,&mut[u8],&mut[u8],&[u8],&mut PersistenceServices)` and is
owned by Game until cleared/drop. `persistence_hooks.rs:24–54` enforces the
issued buffer pair. Types/counters already serialize (`persistence_types.rs`).

Root retains shared ownership: add `pub mod persistence_runtime` in `src/lib.rs`,
add ordinary `sm-persistence = { path = "persistence" }` dependency to the root
Cargo.toml (currently only a workspace member), and make CLI/snapshot changes.
Do not grant worker `gameplay.rs`, `runtime_cli.rs`, Cargo, native files, shared
indexes, or effect ids. Player25 owns new player files/effects56..59;
persistence stays on existing52..55. Adapt to concurrent edits.

## Root runtime changes

- Add `--persistence c|rust` to `Options::parse` whitelist
  (`src/runtime_cli.rs:80–100`) and preflight value validation beside other
  owner options (`:159–173`). Resolve the default from `--gameplay` as existing
  item ownership does; explicit c/rust overrides must work independently.
- Install once immediately after `Game::boot` (`:764–766`) and before the first
  `game.step`, when selected Rust. Existing item/scroll/camera installation follows
  at `:767–792`. This location covers ordinary startup codec calls as well as
  later menu/gameplay calls. C selection leaves the hook absent.
- Add `persistence_owner`, all existing `PersistenceCounters`, and the installed
  owner name to result metadata beside existing owner/counter serialization
  (`:1215–1239,1251–1258`). Preserve each entries/c_entries/rust_entries/
  rust_refused/rust_completed[4] plus disk requests/completions. Report observed
  counts; installation alone is not exercised ownership and zero Save calls
  does not establish saving. Safe native fallback is explicit evidence.
- Retain the new module and codec sources in the run source snapshot list
  (`:322–337`); existing CLI args are forwarded to run-worker (`:339–343`).
  No new route policy, save import option or filesystem semantics needed.

## Ordinary native call contexts and existing limits

| Context | Exact source call anchor and implication |
| --- | --- |
| Startup | `sm_80.c:217` calls VerifySRAM; `:90–98` short-circuits Load0/1/2 until valid. It may then call unrelated RtlWriteSram outside any Save root. Install before stepping so these Loads reach Rust. |
| File selection | `sm_81.c:1475–1476` LoadFromSram_ delegates; selection at1612 loads or NewSaveFile on invalid checksum. Data-clear completion at1256 and1276–1282 also loads slots. Caller-side SRAM writes/RtlWriteSram remain native. |
| Continue/game-over | `sm_81.c:544` loads selected slot; GameOverMenu_3_Main at443 saves then SoftReset. These are ordinary wrappers, not special fixture entrypoints. |
| Ceres transition | `sm_82.c:334–356` GameState_33_BlackoutFromCeres calls Save. Its area can be the unowned Ceres area6, making atomic Rust refusal/native fallback a relevant composition case. Do not require zero C Saves globally. |
| Save station | `sm_84.c:749–760` polls message23; only accepted confirmation spawns electricity, updates station fields and calls Save. This instruction is not among the current item-instruction callback selectors (`native-host/src/item_instruction_prepare.rs:8–45`), so no existing active item hook encloses this Save. |
| Debug game start | `sm_82.c:4637–4644` optionally Saves before starting. |

The runner creates a fresh `out/saves` directory and changes cwd before boot
(`runtime_cli.rs:764–765`). `native-host/src/shim.c:39–53` calls `RtlReset(0)`;
the retained `sm_rtl.c:155–159` cold reset zeros SRAM. This task therefore adds
runtime codec ownership, **not** saved-game import/reopen/continue capability.

Argument/admission semantics must remain those already accepted:

- Pure Rust masks slots with argument&3, accepts0..2/high-bit aliases; slot3
  returns None. The native dispatcher **fatally rejects slot3 before original
  C access or callback**, and nonzero direct-helper args likewise
  (`native-host/src/persistence.c:58–63`). Never turn slot3 into a C fallback.
- Save area>5 returns atomic None (`persistence/src/lib.rs:75–84`); safe unowned
  runtime areas may fall back to original C (`persistence.c:88–97`). This does
  not authorize arbitrary malformed areas/state as safe C fixtures. Load does
  not validate saved area/station contents beyond the accepted codec contract.
- Dirty refusal, refusal after service, wrong output, missing/extra Save disk
  service, callback errors and poisoned service are already fatal
  (`persistence.c:78–102`). The Rust trampoline catches panic and rejects changed
  generation/swallowed service failure (`persistence_hooks.rs:108–165`).
- Native sibling/reentry guards already reject concurrent callback/service/
  observer use (`persistence.c:20–24,58–60,108–145`); setters require idle state.
  No nested persistence call from another Rust owner is newly authorized here.
  Preserve these guards when root integrates player25 sibling fences.
- RtlWriteSram outside an active codec invocation is deliberately unobserved by
  persistence counters (`persistence.c:115–116`); do not count startup/file-menu
  raw writes as Rust Save ownership. Save's disk service retains its existing
  synchronous behavior and ignored native I/O failures.

## Additional composition checks only

Root chooses a bounded check schedule; no new runs were performed here.

1. CLI matrix: default C, gameplay-rust inheritance, explicit persistence-c
   override, independent persistence-rust with gameplay-c, invalid value refusal.
   Check run-worker option forwarding and result/snapshot fields.
2. One short ordinary startup in C and Rust selection: verify identical relevant
   full state/ordered effects and startup Load counters, proving installation
   precedes VerifySRAM. Fresh invalid SRAM may exercise three Loads; report actual
   observed counts rather than imposing a generic three-call requirement.
3. A bounded composition control through the **new production installer**:
   admitted Save then Load, plus the already-declared safe area6 refusal case,
   checks native disk forwarding/output/counters. Reuse existing finite fixtures
   and native invocation boundary; do not re-run the full accepted body/fatal
   matrix. One retained save-station caller exercise is useful if root already
   has bounded state, but navigation/new route work is not required by this seam.
4. Install alongside existing owners (and player25 once integrated) at idle;
   verify independent registrations survive. Existing fatal matrix remains
   evidence for unchanged guards; rerun only if shared guard integration changes
   their behavior. Do not introduce deliberate production faults or new effect ids.

## Completion evidence

Only this note was written. No builds, tests, games, claims, saved-game/reopen
evidence or processes created. Cleanup: no run directory, no command applicable,
**0 bytes reclaimed**; available disk observed **99049459712 bytes**. Existing
accepted persistence evidence and pinned sources remain untouched.
