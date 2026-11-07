# Samus moving through the world: decode notes

For the observer, the adversary and the domain harness's decoder only. Implementers never see this
file (MODEL §5). It holds every source the spec draws on: the original's routines, its RAM and its
codes. Addresses are the bank logs' (`$TEMP/c4a/logs*`, https://patrickjohnston.org/bank/).

## How the spec was made

Behaviour first: the 32 rows in `tools/oracle/samus_motion_rows.txt`, cached in
`$TEMP/collab/more-caches/cache`, were dumped per frame (`$TEMP/obs-ground/dump`, driven by
`$TEMP/respec-motion/one.ps1`) and read over time. The previous spec's arithmetic, which was pinned
from the bank logs, was reused only for exact values and edge cases. Vectors come from
`$TEMP/respec-motion/scripts/vec.pl` (frame selection) and `tag.pl` (entries); the constructed ones
were worked by hand from the rules.

**Frame selection for recorded vectors.** Record N is the State before, record N+1 after; the pad
is record N's. Taken when: the frame counter `05B6` advances by one (no lag); `0A66` is 0; the
movement type `0A1F` is one of the domain's; the pose `0A1C` and the radii `0AFE`/`0B00` are the
same at N and N+1 (or, for a landing, `0B36` goes from non-zero to 0 with y's subpixel FFFF); the
slope flag `0DBA` is 0 at both; extra run `0B42` is below 2.0 (no speed booster); the x move equals
± the frame's total speed `0DBC`.`0DBE`; in the air, `0DC6` is 0 at N+1 and the y move equals the
frame-start vertical velocity, or 0 at the apex; on the ground, y unchanged with y.sub FFFF and the
floor (y + y radius) on a 16 px line. Excluded rows: chain-0220-D017, chain-0224-D0B9,
chain-0232-D5A7, area-maridia (water), terrain-quicksand, samus-ceres-push (pushes). area-brinstar
and samus-square-slope-reserve give few or none (elevator, slopes). At most one vector per kind
(activity, vertical, drive, pad direction, run) per row, nine per row. The pad's bindings are taken
as the defaults (jump A `0080`, run B `8000`, shoot X `0040`); a row with other bindings would be
misread (not checked: `09B2`-`09B6` are not in the dump).

## State

| Quantity | RAM | Decode |
| --- | --- | --- |
| position | x `0AF6`.`0AF8`, y `0AFA`.`0AFC` | as is |
| run_speed | `0B46`.`0B48` | as is |
| dash | `0B42`.`0B44` | as is |
| dash_kept | `0B3C` | ≠ 0 |
| drive | `0B4A` | 0 Driven, 1 Coasting(Behind), 2 Coasting(Facing) |
| vertical | `0B36` | 0 Settled, 1 Rising, 2 Falling |
| vertical_velocity | `0B2E`.`0B2C` (a magnitude) | `0B36` = 2: + magnitude; else − magnitude (two's complement over 32 bits) |
| bounces | `0B20` low byte | 0, 1, 2 |
| pose | `0A1C`, `0A1E` | body and action from the movement type `0A1F` (below); facing `0A1E` low byte 4 Left, 8 Right, 0 Screen |

Pose decode by movement type: 0 Upright Still (poses 00, 9B face the screen); 1 Running; 2
Jumping (TakingOff for 4B, 4C, 55-5A); 3 Spinning; 4 Ball, Still for 1D, 41, else Rolling; 5
Crouched Still; 6 Falling; 8 Ball Falling; E Turning; F Changing; 10 Moonwalking; 11 SpringBall
(Still for 79, 7A); 12 SpringBall Rising; 13 SpringBall Falling; 14 WallJumping; 15 RanIntoWall;
17, 18 TurningInAir. Other types are outside the domain (0A knockback, 16 grapple, 19 damage boost,
1B shinespark).

**RAM shape** (fields decoded as one word copied as-is): none. Positions and speeds join two words;
dash_kept is a test; drive, vertical and bounces are mapped; vertical_velocity combines two words
and the direction.

**Inputs.** pad held `8B`, newly pressed `8F`; equipment `09A2` (0100 hi-jump, 0200 space jump);
`was_spinning`: `0A27` is 3 or 14; `spin_turned_far`: animation frame `0A96` ≥ 0B (≥ 1B in poses 81,
82); `elevator_moving`: `0E18` ≠ 0.

## Sources per entry

The single-routine share: 11 of 46 entries (24 %), marked *.

| Entry | Source |
| --- | --- |
| ActivitySpeeds* | `$90:9BD1` and the normal speed table it indexes by movement type |
| SpeedingUp | `$90:9A7E`, `$90:9A99`, `$90:8EA9` |
| Braking | `$90:9A7E`, `$90:9B1F` |
| StandingStill | the standing, crouching, transition, ran-into-wall and facing-forward handlers of `$90:A337`'s table; `$90:8EA9` with base 0; the stop at their end; `$91:DE53` |
| Dash | `$90:973E`, `$90:97D5` |
| DashCap* | `$90:97D5` |
| LosingTheDash | `$91:DE53`, `$90:E5CE`, `$90:A68F`, `$90:A5CD`, `$91:EC50` |
| TurnSkid | `$91:F543` (turn start), `$90:9B1F`, `$90:8EA9` (heading when turning around), `$90:A68F` |
| TurnCreep | `$90:A67C`, `$90:9A99` |
| TurnIgnoresFall* | `$90:A68F` |
| ArmPumpStep* | `$91:EADE` |
| StandingShotRestartsAnimation* | `$90:A3AD` |
| RidingTheElevator | `$90:A392`, `$90:A397` |
| BallRolls | `$90:A521`, `$90:9BD1` |
| JumpSpeeds | `$90:98BC`, `$91:FA0F`, `$91:F1FC`, `$90:90E2` (table values) |
| TakingOff | `$91:FBBB`, `$90:973E`, the take-off rows of `$90:A337`'s table |
| JumpStarts | `$91:F624`, `$91:F8D3`, `$90:98BC` |
| FollowingTheGround | `$90:923F`, `$90:E61B`, `$94:8F82` |
| StartingToFall | `$90:E61B`, `$91:EFC4` |
| MovesAsShePosed | `$90:A337`, `$91:EB88` |
| Gravity | `$90:90E2`, `$90:913A` |
| FallingSpeedLimit* | `$90:910C` |
| RiseEnds | `$90:8FDC`, `$90:90C4` |
| AirDrift | `$90:900D`, `$90:9174`, `$90:91A8`, `$90:A5CD` |
| CarriedDrift | `$90:9A7E` (no braking in the air), `$90:9174` |
| SpinDrift* | `$90:906C` |
| SpinReverse | `$91:F543`, `$91:DE53` |
| AirTurn | `$90:A68F`, `$90:90E2` |
| SpaceJump | `$90:A488`, `$90:98BC` |
| Moonfall | `$91:EBAF`, `$91:F36E`, `$90:913A` |
| WallJumpArming* | `$90:9D35` |
| WallJumpKick | `$90:9D35`, `$91:FA0F`, `$91:EFC4`, `$90:902D` |
| WallProbeSubpixel | `$94:967F`, `$94:8F49` |
| Landing | `$90:E61B`, `$91:EFC4`, `$91:F010`, `$90:A640` |
| BallBounce | `$91:F1FC`, `$91:F25E` |
| HitCeiling | `$90:E606`, `$91:EFC4` |
| PoseChangeEffects | `$91:EC50`, `$91:FB8E`, `$91:EB88`, `$91:F404`, `$91:F6E3`, `$91:EBAF` |
| FloorsStopFlush | `$94:8F49`, `$94:8F82` |
| WallStop* | `$90:E5CE` |
| CeilingsStop | `$90:E606`, `$94:8F82` |
| FastMovesTunnel | `$94:8F49`, `$94:8F82` |
| MoveLimit* | `$90:E4AD` |
| ZeroMovesStillTouch | `$90:8EA9`, `$A0:AAC8` |
| SolidEnemyTouch | `$A0:AAC8`, `$90:9D80`, `$90:9DF8` |
| SlopesCarry | `$90:923F`, `$90:9376`, `$94:87F4` |
| WordwiseTopSpeed* | `$90:9A99` |

`PoseChangeEffects`' four effects are the prospective pose-change commands `0A2E` 1, 2, 6 and 8
(`$91:EC50`). This is the spec's known one-to-one risk (open question 2): the next lane should key
them to the pose chart's records by what the player does, so the command word disappears.

## Harness notes kept from the previous spec

The domain harness (`verify::harness::samus_motion`) decodes the previous spec's types; its frame
skips (game state `0998`, lag, the elevator `0E18`, pose fit, handlers `0A44` and `0A58`, the
deferred behaviours) and the bridge's answers are in git history at `1dc3a12`
(`docs/behaviours/samus-motion.decode.md`). They carry over once the implementer's new types exist.
