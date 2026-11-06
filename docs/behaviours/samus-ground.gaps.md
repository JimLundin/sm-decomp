# Samus on the ground: integration gaps

Where wiring [the spec](samus-ground.md)'s model into the engine found something that doesn't
fit (MODEL §5). Each gap names the frame, the field, the expected and actual values and the
suspected entry. The integrator files them; the observer answers them in the spec.

These come from reading the engine's existing ground motion, pose update and collision
(`movement.rs`, `pose.rs`, `pose/contact.rs`, `collision.rs`, `collision/solid_enemy.rs`), before
the model was wired into the Tick. No frame has been localised to them yet: the frame column says
"not run" until the free-run localiser or lockstep points at one.

| # | Frame | Field | Expected (today's engine) | Model | Suspected entry |
| --- | --- | --- | --- | --- | --- |
| G1 | not run | pose, x | The arm-pump check runs in the pose change, after the step, the elevator ride, the animation and the collision request; the request it reads (`new_running_pose`) is chosen from this frame's vertical result | `new_running_pose` is an input to the same `step` that produces the vertical result | `ArmPumpStep`, §6 order 6 |
| G2 | not run | x speeds, pose | A zero-distance horizontal move still runs the solid-enemy check in the direction she moves (facing, or against it while turning), so a touching enemy on her left stops a still Samus facing left | `move_horizontal` gets the signed distance only; at 0 it has no direction, and the bridge checks rightward | `HorizontalStep`, §5 `move_horizontal` |
| G3 | not run | y subpixel | The solid-enemy check zeroes her Y subpixel when she already touches an enemy, on every axis, including a horizontal move and the arm-pump's enemy test | `Moved` reports the moved axis only, and `enemy_blocks` takes `&self`; the Y write is lost | §5, `FloorFlushSubpixel` |
| G4 | not run | y, slope flag | The elevator's step down is a terrain-only move whose request says up (slope reactions read it) | `move_down` has no heading; the bridge infers it from `Enemies::Skip`, which only `ElevatorRide` uses | `ElevatorRide`, §9 |
| G5 | not run | x, speeds | Crouching, Transition, Moonwalking, the spring ball and RanIntoWall have no ground motion in the engine today (the handler returns), and Crouching, Moonwalking, the spring ball and RanIntoWall keep no speeds | Each has a row, and the speeds are one State whatever the movement | §3 rows; §1.1 |
| G6 | not run | base speed, mode | Pose commands (decelerate, stop, kill X speed, kill run speed) and the landing's speed stop change the speeds only for movements that keep them | The speeds are one State; those writers would apply in every movement | §1.1 `mode`, pose selection |
