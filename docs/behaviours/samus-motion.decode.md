# Samus's motion: decode notes

For the observer and the domain harness's decoder only. Implementers don't read this file (MODEL §5): it maps the spec's quantities to the original's RAM.


ROM codes and RAM, for `verify`'s decode only. Record N gives the State and the inputs of frame
N+1; the harness compares the State with record N+1.

**State.**

| Quantity | RAM | Decode |
| --- | --- | --- |
| position | x `0AF6`.`0AF8`, y `0AFA`.`0AFC` | as is |
| base_speed | `0B46`.`0B48` | as is |
| extra_run | `0B42`.`0B44` | as is |
| momentum | `0B3C` | ≠ 0 |
| mode | `0B4A` | 0 Accelerating, 1 TurningAround, 2 Decelerating |
| vertical_speed | `0B2E`.`0B2C` | as is |
| vertical | `0B36` | 0 None, 1 Up, 2 Down |
| bounce | `0B20` low byte | 0 None, 1 First, 2 Second (the high byte, 6, marks the spring ball; nothing here reads it) |

**Inputs.**
- locomotion: movement type `0A1F` and pose `0A1C`. Ground: type 0 `FacingForward` (00, 9B),
  `Standing { plain }` (01, 02), else `Standing`; 1 Running; 4 Ball (at rest 1D, 41); 5 Crouching;
  E TurningOnGround; F Transition; 10 Moonwalking; 11 Ball spring (at rest 79, 7A); 15
  RanIntoWall. Air: 2 `Jumping` (`take_off` for 4B, 4C, 55-5A); 3 SpinJumping; 6 Falling; 8
  `BallInAir { spring: false, falling: true }`; 12 spring, not falling; 13 spring, falling; 14
  WallJumping; 17, 18 TurningInAir. Other types: the frame is not in the domain.
- facing: `0A1E` low byte: 4 Left, 8 Right, 0 Forward. radius: `0AFE`, `0B00` of record N+1 (the
  record lags the pose by a frame).
- pad: held `8B`, newly pressed `8F`; bindings: jump `09B4`, run `09B6`, shoot `09B2`.
- equipment `09A2`: 0100 hi-jump, 0200 space jump.
- was_spinning: `0A27` (two-updates-back movement type) is 3 or 14. spin_ready: animation frame
  `0A96` ≥ 0B (≥ 1B in the screw-attack poses 81, 82).
- pose_request, PoseStart, PoseCommand: derived from records N and N+1 (the pose change, and the
  command `0A2E`) until pose selection is its own domain.
- elevator_moving; on_slope (`0DBA` ≠ 0 or the square-slope flag); submerged is false.
- vertical result `0DC6`: 0 None, 1 Landed, 2 Falling, 4 HitCeiling, 5 WallJump; kind `0DC7`.

**Frames the harness skips or masks.**
- Frames where Samus didn't run: item-morph-ball:818-1216 repeat one record while the pad changes
  (the item's message box). Test the game state, not record identity (open question 5).
- A frame whose y radius changes between records N and N+1 has a pose-fit shift in y
  (item-morph-ball:739, 1259, 1304, 1311, 1327): don't compare y there until pose fit is an input.

**RAM shape** (copied as-is): position, base_speed, extra_run and vertical_speed are P.S
quantities (the cap quirks need both words); momentum is a bool. mode, vertical and bounce are
enums.

---

