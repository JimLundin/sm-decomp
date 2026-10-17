# Ordinary Chozo missile instructions

The `sm-item-instructions` crate owns thirteen instruction and trigger bodies
for the ordinary Chozo missile PLM, header EF2F and argument0022. Admission checks
the full WRAM, even slot, sealed script sites and ROM bytes before mutation.
The native scheduler, allocation/collision, timed drawing, HUD/music and message
presentation remain dependencies. Safe foreign items refuse unchanged and use C.

`--gameplay rust` installs this consumer with the six existing Rust consumers.
`--item-instructions c|rust` overrides only item ownership for comparison.
Use a newly prepared library with these exports; older sealed runs keep their
original binaries and libraries.

Ticket18 candidate1bba1a8 passes25actual-C body pairs,14atomic refusals, real
service52 and projectile trigger controls, and11fatal transport controls.
Six semantic tests and13runtime tests pass. The fixed24832-input C/Rust pair
finishes in25.603/24.861seconds. All eight existing artifacts also match accepted
ticket17. All114instruction/service callback records match, including full-WRAM
hashes and typed outputs. All13bodies complete in Rust:51standalone calls plus
three real service52 calls, zero in-scope C/refusals. Three named native services
complete once each. Message2 waits394steps, followed by resumed ordinary input.
Outside this item domain1184entries remain C.

From `restart/`, the retained Rust run is reproducible with a fresh output path:

```sh
./out/item-instructions/bin/sm-restart-02 run \
  --gameplay rust --item-instructions rust --scenario missile \
  --inputs out/first-missile/author-05/inputs.txt --steps 24832 \
  --deadline-seconds 90 --capture-every 500 \
  --library out/item-instructions/native/74a6e54f740f4b9af2fc226932984074bbae3a3ca66c8b683751da9b24f9196c/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc --out out/item-instructions/replay-new
```

Evidence lives in `out/item-instructions/`: compact control logs, both live runs,
source archives, binaries, a retained compiler failure, and authenticated cleanup
and comparison reports. All four achieved stages are published separately.
Both implementer worktrees are retired with required source snapshots retained.
This establishes the bounded native C/Rust comparison, not original-ROM parity.

## Ordinary Morph Ball

Ticket 19 extends the same module to EF23/argument001A. Eight existing bodies
are reused; graphics loading, both animation frames, equipment pickup and the
private frame Common body are now Rust. Common calls three explicit native
dependencies: ProcessPlmDrawInstruction, CalculatePlmBlockCoords and DrawPlm.

`--morph-instructions c|rust` changes only Morph ownership and defaults to the
item owner. Separate `morph-callbacks.csv` and `item-callbacks.csv` preserve
Morph and missile observations. The missile scenario also retains the original
Morph item CSV and acquisition snapshots through its 23,036-step prefix.

Frozen candidate `64f135c` passes 31 Morph actual-C pairs, 20 refusal cases,
real room-load and drawing transaction controls, and four new fatal controls.
The changed ABI also passes the existing compact controls. Ten semantic tests,
14 runtime tests and all-target Clippy pass.

The unchanged 24,832-input pair finishes in 26.589/26.534 seconds. All 13 artifact
pairs match, including 3,176 Morph callbacks and the 114 missile callbacks.
The prior Morph and missile artifacts remain identical. All 12 scheduled Morph
roots complete in Rust: 1,028 calls, five inside real room-load service 52,
zero in-scope C entries or refusals. Both frame roots complete 93 times, executing
local Common 186 times. Message 9 waits 398 steps, then control resumes.

For this candidate, use `out/morph-instructions/bin/sm-restart-01` and library
`out/morph-instructions/native/27db63a152057ceb64f8b59ffd29aacaa3c9d8b58cafa9089e2dd9b0303c7740/libsm_native.so`
with the preceding fixed-input command and `--morph-instructions rust`.
Evidence and retained tools are in `out/morph-instructions/`. Both source workers
are retired, all achieved stages are published, and retained dependencies
authenticate after cleanup. Native rendering, setup/contact, scheduler and
presentation remain dependencies; original-ROM gameplay parity is untested.

## Exposed resource tanks

Ticket 20 owns the complete exposed energy, missile, super-missile, power-bomb
and reserve instruction family: headers EED7/EEDB/EEDF/EEE3/EF27 and permanent
item IDs 0000..01FF. Its 17 functions comprise 16 scheduled roots and local
Common, with four new collectors and two explicit native HUD services.
`--tank-instructions c|rust` selects this family and defaults to the item owner.
Tank callbacks and standalone/service52 counters are reported separately.

Frozen candidate `e5d7498` passes 36 actual-C body pairs, 21 atomic refusals,
and ten fresh-process scheduler pairs. All five normal chains reach the real
pending pickup message with equal WRAM, effects and service observer images;
collected paths skip pickup, with reserve graphics loaded before the branch.
Message completion is outside these chains. Six new fatal controls and fifteen
affected legacy fatal controls return the expected native exit 2. The affected
missile/Morph compact families, 13 semantic tests, 15 runtime tests and Clippy
also pass.

One unchanged 24,832-input replay takes 25.373 seconds and matches all 13 accepted
ticket 19 artifacts. It reaches no tank collectors, so this is compatibility
evidence; no new-collector live-exercised stage is granted. Source translation,
actual-C verification and integration are published independently. Native
scheduler, setup/contact, drawing, HUD/music/message services remain dependencies.
Evidence lives in `out/tank-instructions/`; source workers are retired and
retained dependencies authenticate after cleanup. Original-ROM parity remains
untested.
