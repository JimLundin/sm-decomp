---
name: sm-drift
description: Triage an sm-decomp divergence (lockstep, faithful or free-run failure, or a lock mismatch). Find the first tick, classify the cause, then fix it or record it. Use it when a row that should pass doesn't.
---

# Triage a divergence

## 1. Localise

- **Lock mismatch or free-run failure:** run the row with `--mode free-run`. It names the first
  divergent tick per domain. Then run lockstep with `--divergence-report R.toml`
  to name the field at that tick.
- **Lockstep failure:** the log's first failing frame plus the report give tick,
  domain and field.

Write down: the row, tick, domain, field, oracle value and engine value.

## 2. Find the writer

Find which routine writes that address on each side at that tick. Read the
original in the bank logs (https://patrickjohnston.org/bank/). Compare it with
the engine's code for the same routine.

## 3. Classify

| Finding | Action |
| --- | --- |
| The engine does something hardware doesn't | fix the engine: `(behaviour: …)` commit |
| The engine reproduces a snesrev patch | `sm-hardware` |
| Engine state that lockstep hid (it adopts the oracle each frame), e.g. a scratch word another routine later reads | fix the engine to keep the original's value |
| A checkpoint cut under an older oracle | re-seed the row from a hardware seed (`sm-scenario`) |
| The inputs no longer reach the event (0 failing frames, expectation not observed) | re-time the row's inputs |
| A byte only snesrev's oracle writes (e.g. `$C2`), never read by the game | mask it in the verifier, tagged `LEGACY(verifier)`, with a LEGACY row |

Hardware-visible differences are never masked. If the engine's scratch differs
for one tick and hardware would show it, it's a real divergence. Fix it, or list
it in LEGACY as a known gameplay-invisible divergence (routine, tick, rows).

## 4. Record

- **Fixed:** the row passes in the failing mode and in lockstep on both oracles. If it
  was unlocked, remove its `unlocked` note and record its lock
  (`python tools/scenario.py lock --record`, then `lock --check`).
- **Not fixed yet:** a `docs/LEGACY.md` row naming the cause, the routine, the rows
  and ticks, and an `unlocked = "..."` note on each affected scenario.
- **Want a permanent regression test:**
  `python tools/scenario.py cut ID --at TICK --window N --report R.toml`
  writes a small scenario. Send its bundle to the root for upload.

Done when every divergence you found is fixed or listed, and you have a report
line for each: cause, action, and rows affected.
