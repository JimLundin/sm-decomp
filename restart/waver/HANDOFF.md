# Waver source handoff — ticket27

Owned only `restart/waver/`; baseline ce31946. Implemented all four selected
actor definitions, no scheduler/lifecycle/terrain rewrite. No dependency or
root workspace/lockfile edits. Public interface coordinated with native owner:

```rust
pub enum Entry { Init = 0, Main = 1, Instruction = 2, SelectAnimation = 3 }
// repr(u32), ALL, address(), TryFrom<u32>
pub enum Output { Done, Continue(u16) }
pub trait Services {
    fn available(&self) -> bool;
    fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String>;
    fn sine(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u16, String>;
    fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String>;
}
pub fn execute(entry: Entry, k: u16, jp: u16, ram: &mut [u8], rom: &[u8],
    services: &mut impl Services) -> Result<Option<Output>, String>;
```

`fixture-support` exposes identical root/module exports:
`sm_waver::{BodyFixture,body_fixtures}` and `sm_waver::fixtures::{...}`.
BodyFixture fields: name:String, entry:Entry, k:u16, jp:u16, wram:Vec<u8>.
Exactly24 fixture definitions:4Init,6SelectAnimation,2Instruction,12Main.
All instruction jps86C5/86D9 are intentionally odd and returned unchanged.

Finite geometry:16x16 room, default current40, x/y40,radii8. Right collision/
spike x38 crosses into col4, rows3/4. Left collision x48 crosses into col3.
Vertical-down fixture phaseC0/y38 targets row4; up phase40/y48 targets row3.
Spikes use blockA000, BTS15, index52 and free/full40-slot PLM pool. Native owner
supplies real finite terrain/Sine/SpawnPLM behavior and actual scheduler controls.
No terrain implementation or canned fake effects are included in fixture data.

Proposed inherited-state chains use existing `instruction-current`→Main and
`init-right`→Main fixture state. Init's sentinel D=BEEF remains intentional;
Main consumes its low byte for sine and updates its full word on no collision.
No private chain matrix or native executable was run by this worker.

Semantics retained:

- SignExtend8 checks bit80 without truncation; left Init A8000 stays8000.
- Left Init leaves sentinel D/timers because selector0 is unchanged; right Init
  uses runtime table and resets timers. Old F is overwritten before admission.
- Horizontal packed-byte reversal preserves fractional low-byte behavior and
  uses widened host arithmetic for signed byte80, avoiding debug overflow.
- Horizontal hit skips sine/vertical. Open path sends angle low8(D),amplitude4,
  then full signed16 result shifted16 to vertical. Native collision scalar must
  be0/1; other values are terminal errors after the request.
- Vertical hit truncates D+80 to low8; clear adds2 wrapping16. Special phase56
  selection occurs before event clearing and normal selection, including both
  changes in the same invocation.
- Full WRAM roundtrip effects survive; actor rereads fields after native calls.
- Main F>3 or unavailable native capability returns unchangedNone. Helper F==C
  returnsDone before ROM/index checks, evenFFFF; changed F>3 refuses unchanged.
- Complete128KiB WRAM, current/explicit instruction slot alignment/range and
  jp8000..FFFE plus ROM extent are validated before writes. Noninstruction k/jp
  are ignored. Table availability preflight covers all four runtime words;
  actual values are intentionally read at runtime, with identity owned by Game.

Fifteen semantic tests cover these distinctions, whole-state untouched Init/
helper/instruction cases, phase/event order, instruction→Main chain, native
scratch and field reloads, immutable refusal/preflight errors and post-request
fatal errors. They are source only until root runs the focused crate tests.

Source validation performed: rustfmt, staged diff whitespace check. No Cargo,
compiler, semantic/native test, game, capture or producer was run. No temporary
run artifact, build cache or save file was generated; worker cleanup cost0.
Source-only handoff is not actual-C verification, runtime/live ownership,
whole enemy lifecycle or original-ROM parity. Root owns integration, all checks,
publishing, evidence retention, worker retirement and final authentication.
