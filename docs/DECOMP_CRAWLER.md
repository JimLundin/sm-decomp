# Bank A3 crawler movement

The pinned MIT snesrev/sm routines for Stone/Fire Zoomer, Big Eye Bugs and
Wrecked Ship Orange Zoomer are translated in `enemy_crawler.rs`. These own
initialization, script preinstruction selection, crawl orientation changes,
earthquake falling, fractional gravity, landing restoration, slope displacement
and the shared floor/ceiling alignment routine. Canonical WRAM preserves the
alias between EnemyData.ai_preinstr and crawler var_F at record offset 0x3a.
`FireZoomer_E8A5` deliberately uses current enemy index, even with a different
argument, matching actual C.

Shared enemy movement now processes horizontal nonsquare slopes, scaling the
mutable displacement on each terrain contact before applying it. BTS15 is the
sixteenth spike-table entry and executes D094's enemy-breakable PLM through the
explicit host; callback state and ordering remain observable.

Inputs require aligned enemy byte offsets below 0x800, speed table indexes below
32 (Stone/Fire's 255 sentinel preserves speed until landing), animation indexes
below six, supported preinstructions, bank A3 script words with an in-bank next
pointer, and bounded acyclic native terrain. Malformed accesses are rejected.
No hidden gameplay fallback is present. PLM host callbacks remain measured C
ownership in the offline harness.

Nine independent Rust examples cover gravity carry, corner contact, landing and saved state,
sentinel initialization, earthquake and fourth miss, repeated slope scaling,
script aliasing, all property orientations and invalid inputs. The actual-C manifest additionally covers
all property orientations, positive/negative velocities, air/solid terrain,
flipped slopes and ordered actual PLM callbacks.

Run `cargo run -p pipeline --locked --offline -- fixtures enemy-crawler tools/fixtures/enemy-crawler.smf` to regenerate the checked-in manifest,
then `cargo run -p pipeline --locked --offline -- family-verify --adapter callback --manifest tools/port/enemy-crawler.smf
--source deps/snesrev-sm --output out/port/crawler`. Translation and offline
verification are distinct from live exercise: the any% prefix may not encounter
these enemies, and no original-ROM parity or full Rust gameplay ownership is
claimed by routine fixtures.

The accepted offline run is `out/port/crawler/20261010T214436.664722Z/verdict.json`:
sixteen routines, 800 complete-state cases and 104 ordered actual PLM callbacks.
Later integration dependency changes require a fresh batch seal before live
admission; the recorded routine comparison remains immutable evidence.
