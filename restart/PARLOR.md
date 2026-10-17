# Landing Site to Parlor checkpoint

This checkpoint extends the retained 16,991-pad Ceres route with ordinary
controller input through Landing Site's west door into Parlor. Success requires
normal area0 gameplay and a material input response after the transition.
It uses the existing mixed C/Rust Game loop; original-ROM agreement and complete
C/Rust differential agreement for the new tail are separate, unproved claims.

The authoring mode accepts only the exact retained Ceres prefix and appends a
bounded observation-based controller policy. The resulting full input log can
be replayed with the same scenario and no policy. Neither reaching a step limit
nor seeing the destination room during loading establishes success.

Initial bounded command from `restart/`, after integration/build:

```sh
./target/debug/sm-restart run --gameplay rust --scenario parlor \
  --inputs out/runs/ceres-06/inputs.txt --tail-policy parlor \
  --steps 21087 --deadline-seconds 90 \
  --library out/projectile-escape/native/0eee36456c66e6a447305e1a2097e96e3afd32365a9013c090f8faf5cea6720f/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc --out out/parlor/author-01
```

The prefix hash is
`39098a57ed3e51787b684da80c54561d81a46600b4931dc012d747c9e7c097f0`.
Ticket15 passes at source `c7d8e50`: the first authoring attempt reaches the
checkpoint at step18,072, adding1,081 pads. Landing departure occurs17,959;
Parlor is observed during loading17,977, then normal gameplay resumes18,064.
Eight consumed left/run polls move Samus from X1240 to1211 at Y139, health99.
Authoring takes16.825s; strict replay takes17.099s. Both have zero CPU fallback.

Replay the retained route without authoring:

```sh
./out/parlor/bin/sm-restart-01 run --gameplay rust --scenario parlor \
  --inputs out/parlor/author-01/inputs.txt --steps 21087 --deadline-seconds 90 \
  --library out/projectile-escape/native/0eee36456c66e6a447305e1a2097e96e3afd32365a9013c090f8faf5cea6720f/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc --out out/parlor/my-replay
```

`author-01` and `replay-01` have byte-identical complete pads, observation CSV,
final WRAM and final PNG. The exact Ceres prefix and its observed state rows
match the retained current ticket13 CLI. The new tail exercises57 Rust Fire,
365 Handle,1007 calls of each renderer,1041 scrolling,105 transition,64
directional and976 camera roots, adding no owned C entries/refusals. The
prefix's renderer exclusions remain, and other gameplay/foreign services remain
recovered C. This is mixed-game exercise and deterministic replay, not a new
per-step full-state C/Rust comparison or original-ROM parity verdict.

Source, executable, failed/passing test logs, input/observations and images are
retained under `out/parlor/`; independent review and final hashes are under
`evidence/`. Sixteen focused tests and Clippy pass. Both worker worktrees are
retired, required source snapshots remain, cleanup succeeds, and shared progress
records only the live-exercised stage for the observed existing Rust roots.
