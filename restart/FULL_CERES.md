# Mixed Rust gameplay through Ceres escape

The build with all five installed Rust consumers escapes Ceres from reset and
reaches normal gameplay at the Landing Site, area0 room91F8. Ticket11 uses the
unchanged16,991 saved inputs and the checked ticket10 binary/library; no
gameplay change, new native preparation or input discovery was needed.

`out/full-ceres-rust/runs/escape-01` passes in27.445 seconds, reaches the ordered
escape states20→21→22 (hex), then area0/state8, with16,068 input polls/updates
and zero CPU fallback calls/instructions. The complete observations CSV,
final128KiB WRAM and final PNG are byte-identical to the retained recovered-C
`out/runs/ceres-06` baseline. The CSV is sampled gameplay state; it is not a
per-step full-WRAM/external-state/effect trace. Original-ROM parity is unchanged.

| Consumer | Observed Rust execution | Remaining boundary |
| --- | --- | --- |
| Ordinary projectiles | 4,891 handling and5,343 calls to each renderer; no firing call | 706 handling and707 calls to each renderer refuse; C handles those and the existing prelude. |
| Room scrolling | 2 displays,645 scroll/map updates,5,802 layer2 updates | No C entries/refusals; generic hardware consumers remain C. |
| Ceres doors | All13 scheduled entry kinds;10 puff-spawn service calls | No C entries/refusals; private main-state coverage is not inferred from the main count. |
| Room transitions | 1,118 roots;630 pending/resumed waits;22 of27 routine kinds | No C entries/refusals; generic room/actor services remain dependencies. |
| Directional scrolling | 640 roots;5 right and5 left setups;320 moves each way;645 scrolling children | No C entries/refusals;8 of13 routines reached, with no upward/downward claim. |

The projectile pilot explicitly refuses the Ceres-status high bit, as documented
in `projectile/README.md`; Mode7 drawing was intentionally excluded. That bit
appears in706 sampled host steps14,372–15,077. The refusal counts align with this
existing boundary, but the sampled CSV does not identify each call's refusal
reason. The milestone therefore claims mixed gameplay, not full Rust ownership.
These inputs also contain no added pilot shot; the9,380-step comparison retains
the separate shot lifecycle evidence.

New reached behavior includes leftward return transitions, all scheduled Ceres
door instruction kinds, the BA48 solid-setup entry and background commandA.
Only live-exercised coverage is extended. The short route's C/Rust comparison
and bounded original Fire comparison retain their exact earlier limits.

From `restart/`, using a fresh output directory:

```sh
./out/door-scrolling/bin/sm-restart run --gameplay rust --scenario ceres \
  --library out/door-scrolling/native/311af8f3ea2746d0298cd202a1b1005ed8963c459bd71980b9d91421f85ed9fe/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc --inputs out/runs/ceres-06/inputs.txt \
  --steps 16991 --deadline-seconds 90 --out out/full-ceres-rust/runs/my-run
```

Evidence retains exact inputs/runtime hashes, the ordinary CLI result and
observations, a small read-only Rust baseline comparison and the independent
review. Existing ticket10 source/native/binary dependencies stay pinned at
gameplay4d6c541; cleanup does not remove that shared retained package.
