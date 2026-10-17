# Player vertical motion

`sm-player-motion` implements seven pinned recovered definitions over full
131072-byte canonical WRAM:

| Entry | Recovered definition |
| --- | --- |
| `MoveUp(i32)` | `Samus_MoveUp`, 90:93EC |
| `MoveDown(i32)` | `Samus_MoveDown`, 90:9440 |
| `CheckSolidEnemy(i32)` | `Samus_CheckSolidEnemyColl`, A0:A8F0 |
| `MoveUpNoColl(i32)` | `Samus_MoveUp_NoColl`, 90:988D |
| `MoveDownNoColl(i32)` | `Samus_MoveDown_NoColl`, 90:9871 |
| `UpFeedback` | `Samus_MoveUp_SetPoseCalcInput`, 90:E606 |
| `DownFeedback` | `Samus_MoveDown_SetPoseCalcInput`, addressless C definition |

`execute` returns `Some(Done)` or the checker's raw collision word and signed
amount. A refusal is `Ok(None)`, leaves all bytes unchanged and requests no
terrain execution. It validates exact WRAM extent, the entire enabled 32-word
terminated/aligned enemy list, reachable feedback indexes and both upward
negation domains before mutation. `MoveDownNoColl` accepts `i32::MIN`;
`MoveUp` and `MoveUpNoColl` refuse it. Disabled lists are not inspected.

The checker predicts its result immutably for root admission, then applies its
effects once. First eligible contact wins in list order; no sorting, nearest-hit
selection or enemy-pointer eligibility is added. Canonical byte0A1F remains a
byte; facing at0A1E is untouched. Left/up exact-contact equality preserves the
C integer promotion: wrapped separation zero can return amount0 without
clearing Y subposition. All other arithmetic uses the original word widths.

Roots call one synchronous `Terrain` continuation for the no-enemy-hit path.
The provider establishes immutable admission from the incoming root and state;
`available` must produce no effects or request. The continuation executes native
`Samus_MoveDown_NoSolidColl`94:9763 and returns its applied amount after full
WRAM copyback. Rust then publishes displacement and feedback in original order.
An accepted service error is an error, never a refusal or permission to replay
the root in C. The provider also owns pointer lifetime, serialization, native
geometry/reaction admission and fatal handling. This crate owns no bank94 solver.

Feature `fixture-support` exports `BodyFixture { name, entry, wram }` and
`body_fixtures()`: one deterministic 192-case valid-C routine matrix. Native
controls import this matrix and add separately identified terrain cases.
Invalid-list/size/index/negation and unavailable-service inputs are tested only
through the safe Rust Interface. Unit service models establish ordering and
copyback behavior; they cannot establish native terrain or gameplay acceptance.

The native bridge, registration, actual-C controls, live instrumentation and
verification/cleanup are owned by the integrator and native worker. See
[HANDOFF.md](HANDOFF.md) for source identity and pending evidence.
