# Morph movement and first missile checkpoint

This completed milestone preserves the23,036-pad Morph pickup route, demonstrates
ordinary directional movement in Morph form, then traverses Construction Zone
to First Missile. It must reveal the Chozo orb with ordinary uncharged fire,
touch the missile tank, observe persistence and ammo/capacity increases, finish
message2, and demonstrate resumed normal control. Missile firing is outside
this pickup checkpoint.

The authoring tail is bounded to8,192 steps (31,228 total), with the
existing90-second deadline. The accepted run uses1,796 additional inputs and
finishes at24,832 steps. The exact prefix hash is
`0a8ab6a50be742c1124aea5b802ae088a3b12cd785b7da54aea6cfc6143ae76a`.
From `restart/`, the retained successful candidate can author the route into a
fresh output directory:

```sh
./out/first-missile/bin/sm-restart-05 run --gameplay rust --scenario missile \
  --inputs out/morph/author-03/inputs.txt --tail-policy missile \
  --steps 31228 --deadline-seconds 90 \
  --library out/projectile-escape/native/0eee36456c66e6a447305e1a2097e96e3afd32365a9013c090f8faf5cea6720f/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc --out out/first-missile/new-author
```

Recorded replay uses the saved complete pad log and omits `--tail-policy`.
EOF cannot enable policy. Existing Game supervision, observations, source and
binary retention remain; item evidence supplements the general CSV with pose,
Morph movement, orb/PLM state, ammo, persistence and message state. Prefix,
reveal, acquisition and final WRAM snapshots retain the stages separately.

The route is Morph9E9F → Construction9F11 → FirstMissileA107, area1. The first
item starts with zero ammo/capacity and clear D874bit04 in the retained prefix.
Pickup must set that bit and add5 to09C6/09C8. Loading, shell destruction,
inventory or a step limit alone cannot satisfy success. Samus, item/PLM,
message and other unowned gameplay remain recovered C; the Rust policy adds
no gameplay translation. Implementation/results are tracked in ticket17.

Source `1b69b05` passes author-05 in24.906 seconds and strict replay-01 in25.134
seconds. Actual ground Morph movement is proved at23,424: nine consumed polls,
seven moving,18pixels right. FirstMissile normal gameplay begins24,372; shell
reveal occurs24,393 and acquisition24,420. Message2 has394 observed waits and
clears at24,814, followed by18 consumed polls and17pixels of rightward movement
through24,832 with99 health.

Complete inputs, general/missile CSVs, prefix/reveal/acquisition snapshots and
final WRAM/PNG are byte-identical on replay. Four failed bounded attempts and
their focused regressions remain retained. See
`out/first-missile/evidence/replay-equality.md`, `attempts.md` and `cleanup.md`
for exact evidence, ownership limits and retention. This establishes no new
gameplay translation or per-step full-state C/Rust/original-ROM equivalence.
