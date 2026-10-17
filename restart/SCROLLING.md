# Room scrolling: second consumer of the same comparison loop

The combined Rust replay reaches the bottom of the Ceres stairwell (DFD7) from
reset. All 9,380 completed host-step observations match the recovered-C reference:
full WRAM, declared external/hardware state, pixels/audio, and ordered effects.
The trace crate is unchanged. Source checkpoint: `6c095fd`.

The fixed input is the first 9,380 inputs of the retained `ceres-06` route with
one added firing edge at step 8,380. The runner checks its grounded running
context before firing. The shot moves, draws, hits terrain, draws its explosion
and clears. All projectile slots are empty in final WRAM.

Rust scrolling starts before the first host step. Completed entry counts are
`[1, 130, 936]` for initial display, transition scrolling and normal scrolling;
all C entry and refusal counts are zero. Ordinary projectiles activate at step
8,262, completing `[1, 802, 971, 971]` calls with no C entries or foreign gameplay
requests during that interval. The cinematic prefix remains C-owned.

The shaft camera advances vertically 0..512, corridor camera horizontally
0..256 and stairwell camera vertically 0..256. With Mode7 disabled and both
adjacent observations in normal gameplay in the same room, the run consumes
16 foreground columns in DF8D and 16 foreground rows in DFD7. Actual DMA during
those gameplay intervals writes 40,064 and 48,256 VRAM bytes respectively.
Those byte totals include other queued graphics in the same flushes.

The initial DF45 display makes 17 synchronous foreign queue flushes, but its
Mode7 flag disables ordinary tile preparation. This live route therefore does
not establish the nonempty 17-column display branch or background-layer uploads.
Separate native service controls prove full-RAM copy-back, ordered real DMA and
failure handling with nonempty rows/columns (9 DMA requests, 18 VRAM bytes);
unit tests check the gameplay module's 17 sequential flushes. Camera policy,
door scheduling, room loading and actual hardware queue execution remain C.
Original-ROM agreement remains unproved.

From this `restart/` directory, use the existing commands:

```sh
cargo build --locked --offline -j1
./target/debug/sm-restart prepare \
  --reference-archive /home/jim/sm-decomp/deps/downloads/snesrev-sm.tar.gz \
  --out out/scroll/native
```

Set `SCROLL_LIBRARY` to the prepared library path printed above. Keep its whole
directory, including `library.json` and source. The reference identity binds
both the C library and authenticated manifest containing the fixed Rust host
recipe. It excludes candidate gameplay and documentation.

```sh
./target/debug/sm-pilot reference --scenario scrolling \
  --library "$SCROLL_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --prefix out/runs/ceres-06/inputs.txt --out out/scroll/runs/my-reference

./target/debug/sm-pilot dev --scenario scrolling \
  --library "$SCROLL_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/scroll/runs/my-reference --out out/scroll/runs/my-candidate
```

Output directories must be new. `check` always executes; `dev` builds
incrementally and reuses an unchanged authenticated passing result. Add
`--fault yes` to corrupt the real vertical scroll register by one pixel during
shaft descent. `--capture-step N` retains full diagnostic state at a selected
host step. Native steps are not original-ROM frame numbers.

The normal executable uses the same two gameplay installers with
`sm-restart run ... --gameplay rust` or `play ... --gameplay rust`.
For exact replay, pass the reference's generated `inputs.txt` with
`--scenario none --steps 9380`.

Measured on this checkout: initial C reference 11.747 s, first Rust comparison
12.790 s. Binding the complete host recipe required reference-02 (17.513 s).
A forced warm gameplay rebuild plus all 9,380 comparisons took **16.622 s**,
including 0.580 s build and 0.081 s validation, within the 30-second target.
These are observed runs, not a bound on other machines or gameplay scenarios.
Primary `target/` is retained as the reusable incremental cache.

Adding this document reused the passing result in **0.208 s**, starting no game
process. The deliberate real scroll-register fault was rejected at **step 8,412,
WRAM**, in 14.618 s. Both successful and diagnostic-failure artifacts are retained.

Evidence: `out/scroll/runs/`, `out/scroll/evidence/`, and
`.scratch/restart-pipeline/issues/06-rust-room-scrolling.md` in the parent checkout.
