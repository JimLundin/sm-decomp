# Connect the verified SRAM codec to ordinary runtime calls

Status: prepared, source-only integration packet requested by integrator01a144ca.
Root selects a second active source assignment beside player25. No codec rewrite,
new native adapter, gameplay rerun or live save/reopen/continue claim is required.

## Existing implementation and missing wiring

Producer24 is resolved at7f0f6e5. Its acceptance covers frozen implementation
230fd9e and control-only correctionf00f206: 30 actual-C body pairs, one absent-
directory Save pair, three Save/Load chains, nine admission and fifteen fatal
controls, twelve semantic tests. Full record:
`/home/jim/sm-decomp-restart/.scratch/restart-pipeline/issues/24-rust-persistence-codec.md`.
Its 1104-file retained manifest SHA is
12c7ac0bd711c53242262ac3d6ff2e36136293d917560703a48640ab8b1c2ce9.
These are previously admitted receipts, not new checks performed by preparation.

`native-host/src/persistence_prepare.rs:11` already wraps actual SaveToSram and
LoadFromSram definitions; normal native callers cross those wrappers. Game owns
the SRAM/WRAM transaction, registration, disk service, counters and observer.
The missing production bridge is exactly `native-host/examples/persistence-host-control.rs:65–82`:
Disk forwards Services::available/write_sram; install maps native Entry to the
codec, and codec Done/Loaded/None to native Handled/Refused. It is currently only
installed by the verification example. No additional C call interception needed.

See [source mapping](persistence-runtime-source.md) for ordinary callers and
startup/save context. Use completed24's frozen persistence-interface.md as the
codec contract. Do not inspect SRAM tables/checksum bodies again to wire it.

## Disjoint source assignment

One worker owns only:

- New `restart/src/persistence_runtime.rs`: public `install(&mut Game) -> Result<(), String>`,
  private Disk trait implementation, typed Entry/Output mapping. Use
  `Entry::try_from(entry as u32)?`, retaining exact existing result/refusal behavior.
- New `restart/examples/persistence-runtime-control.rs`: small composition control
  that imports that public production install, never another copied bridge.
- New handoff README in the assigned local issue scope, with commands and limitations.

Root owns the existing files: restart/Cargo.toml/Cargo.lock, src/lib.rs export,
runtime_cli.rs, main.rs usage and its existing CLI tests. Root retains every
shared edit for player25. No worker edit to gameplay.rs/native-host or codec is
needed; no second native worker is required for this small integration scope.
A worker can deliver new files while root is still freezing player code.

Register `sm-persistence = { path = "persistence" }` as a normal sm-restart
dependency and export `pub mod persistence_runtime;`. The existing native-host
example remains a historical control source; it need not acquire a cyclic
sm-restart dev-dependency. The new composition example may use the already locked
libloading crate via a root-owned dev-dependency to invoke original exported
SaveToSram/LoadFromSram symbols. Do not expose unsafe diagnostic mutation as a
new production Game API merely to test installation.

## Root runtime changes

1. Add `--persistence c|rust` to allowed arguments, validation and help. Resolve
   its effective default from `--gameplay`, matching other selected consumers;
   explicit c can override Rust gameplay, and explicit rust can enable it with
   C gameplay. Test this small option matrix plus invalid value/prepare rejection.
2. Immediately after `Game::boot`, before the first host step, call the shared
   production install if effective owner is rust. Existing runtime creates the
   owned `out/saves` directory and changes child cwd before boot (`runtime_cli.rs`
   around761–766); retain that disk isolation unchanged. Do not add SRAM preload,
   file restore, directory migration or a new save UI.
3. Retain `persistence_runtime.rs` in the existing run source snapshot list near
   runtime_cli.rs:323. Keep the complete scoped source revision as the candidate
   identity; do not create another independent evidence format.
4. Report `persistence_owner`, hook-installed status, the existing typed
   `game.persistence_counters()` and native disk request/completion counts in
   result.toml. Add persistence to configured gameplay-owner description when
   enabled, but keep actual completion counts separate from configuration.
   Optional final SRAM hash is only a state observation, not durable disk proof.
5. Keep old observation CSV formats and milestone predicates unchanged. No new
   per-frame full-state dump, inventory refresh or effect IDs; persistence already
   uses52..55, while player25 has56..59. Tiny final counters suffice here.

The unchanged contract admits masked slots0..2 and Save area0..5. Ordinary Ceres
blackout can Save from area6, which the verified codec refuses unchanged and the
wrapper delegates to original C. Do not turn that legitimate mixed ownership
into a failure of Ceres or fake ownership. Masked slot3 remains unsafe native
input and existing native guard terminates before invocation; never defeat it.
A post-service error remains fatal. Loaded(1) is invalid saved data, not refusal.

No unconditional `all c_entries == 0` or all four routine completion requirement:
Pack/Unpack are local implementation calls inside Save/Load and not artificial
nested runtime callbacks. A run with zero saves/loads should truthfully report
an installed hook and zero exercised calls, not a new live-exercised stage.

## Needed checks and retained evidence reuse

Use focused root-owned tests and composition, not repeated acceptance matrices.
The production module is a small forwarding adapter; tests target installation
and owner selection, not the already verified checksum/map implementation.

- Build the normal sm-restart executable and run focused CLI-selection tests.
- One compact composition control boots the authenticated native Game in a fresh
  owned cwd, installs the **production** module and invokes actual exported
  SaveToSram then LoadFromSram with one admitted representative state. It checks
  typed return, Rust completion counters, canonical WRAM/SRAM and real disk
  service result/order against producer24's equivalent retained expectations.
  This establishes the new call path; it is not a gameplay route or reopen test.
- Include disabled selection/no install and safe unowned-area refusal accounting
  in that small control when needed to distinguish configuration from execution.
  Do not run an unsafe slot3 C body. Existing dirty transaction, poison, reentry
  and fatal-exit controls retain their evidence unless their source changed.
- Root chooses exact CLI arguments, one compiled helper and bounded child runs
  in the assignment handoff. No `persistence-host-control ... all`, full previous
  gameplay replay, original-ROM comparison, new emulator or whole-suite gate.

If only production Rust wiring changes, reuse the authenticated native library
and completed24 codec/adapter evidence unchanged. Concurrent player25 changes
native guards/recipe: bind the actually selected merged native identity and run
only the small composition check on that version. Do not label the old library
as the new one, and do not rerun30 codec pairs merely because an unrelated player
adapter was linked. A concrete failing interaction can expand its own check.

This scope grants source integration of ordinary persistence hook installation
and the demonstrated composition extent. Actual gameplay Save/Load, a real file
reopen/continue lifecycle, durability and original-ROM parity remain separate.
No new translated gameplay bodies: all four were already published under24.
Root publishes only the integration/coverage facts newly earned, completes normal
cleanup and retires this small source worker. Preparation produced only Markdown;
no runs/claims/worktrees, cleanup0bytes.
