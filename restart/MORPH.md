# Morph Ball progression checkpoint

The selected milestone preserves all18,072 pads of the proven Parlor route,
then follows Parlor → Climb → Pit Room → elevator → Morph Ball room. It uses
ordinary controller inputs through the existing mixed C/Rust Game loop.
The initial tail is bounded to8,192 additional host steps and90seconds total.

Success requires observed acquisition from initially clear Morph equipment,
collected-equipment and persistent item bits, followed by normal area1/room9E9F
gameplay and material controller response after pickup presentation. Merely
entering the room or observing the inventory mutation is insufficient.

Initial authoring command from `restart/`, after integration/build:

```sh
./target/debug/sm-restart run --gameplay rust --scenario morph \
  --inputs out/parlor/author-01/inputs.txt --tail-policy morph \
  --steps 26264 --deadline-seconds 90 \
  --library out/projectile-escape/native/0eee36456c66e6a447305e1a2097e96e3afd32365a9013c090f8faf5cea6720f/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc --out out/morph/author-01
```

The exact prefix SHA256 is
`125656a5c9d6bd040472d07ea23ac72f93c72ff6c54c092a61bb662b298b57bb`.
The successful complete pad log will replay with the same scenario and no
`--tail-policy`. Recorded-input EOF never authorizes generated input.

Item evidence supplements the existing observations: prefix and acquisition
WRAM snapshots and a bounded item-observation CSV retain room/state/input and
word09A2,word09A4,byteD873. Morph bit0004 in both words and persistent bit04
must change from clear to set. Source evidence identifies PLM EF23 at block
(69,41),argument001A, pickup message09. Source room/door topology provides
waypoints; it does not establish native route timing or feasibility.

Ticket16 passes at final source `c3b6a76`: author-03 acquires all three Morph
bits at step22,624, observes message09 and398 waits, then presentation clears
at23,022. Fourteen consumed polls (12 moving) produce16px leftward movement;
the checkpoint completes23,036 at(1108,651), health99. The route adds4,964 pads
and has zero CPU fallback. Normal room entries follow the pinned route at
18,072/19,189/20,943/21,645/21,883.

The first two bounded attempts exposed policy gaps: a raised Climb platform
needed an observed grounded jump; the elevator needed ordinary Up to restore
standing, then native contact before fresh Down. Both failures and focused red
regressions remain retained. The corrected source passes27 focused tests and
Clippy. No gameplay implementation or native library recipe changed.

Authoring takes23.426s; strict replay-02 takes23.326s. Both produce byte-identical
inputs, general/item CSVs, prefix/acquisition WRAM, final WRAM and final PNG.
The prefix also exactly matches the accepted Parlor inputs, observations and
final WRAM. Replay with a fresh output path and no policy:

```sh
./out/morph/bin/sm-restart-03 run --gameplay rust --scenario morph \
  --inputs out/morph/author-03/inputs.txt --steps 26264 --deadline-seconds 90 \
  --library out/projectile-escape/native/0eee36456c66e6a447305e1a2097e96e3afd32365a9013c090f8faf5cea6720f/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc --out out/morph/my-replay
```

This verifies acquisition and ordinary post-message control. Actual Morph-form
movement, downstream items, full-state differential and original-ROM parity
remain outside the checkpoint. Existing Rust roots are exercised with no added
owned C entries/refusals; prefix renderer exclusions remain. Samus, elevator,
item/PLM and message logic stay recovered C. Evidence, independent review,
cleanup and final authentication are retained under `out/morph/`; both worker
worktrees are retired and the shared inventory records live exercise only.
