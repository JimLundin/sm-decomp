---
name: sm-hardware
description: Make the sm-decomp engine follow the real game at a snesrev patch site, and flip the patch's policy to follows. Use it when a row diverges only on the faithful oracle.
---

# Follow hardware at a patch site

The **patched** oracle runs snesrev's bug-fix patches; the **faithful** oracle is
the unpatched game. Each patch site has a policy in `tools/snesrev_patches.py`:
`native="patched"` (the engine copies snesrev) or `"follows"` (the engine follows
hardware). The real game is the authority, so sites move to `follows`.

## Steps

1. **Reproduce.** The row fails with `--oracle faithful` and passes without it.
   Note the first tick and address.
2. **Find the site.** Add `--oracle-keep 0xAAAAAA` (comma-separated, or `$80:95E1`
   form) to keep candidate patches in the faithful oracle. The row passes when the
   right site is kept. Bisect the parked and `patched` entries near the failing
   routine.
3. **Understand the original.** Read the routine in the bank logs
   (https://patrickjohnston.org/bank/<bank>) and the patch's note in
   `snesrev_patches.py`. Write down what hardware does that snesrev changed.
4. **Fix the engine** so it does what hardware does. Keep typed state; scratch
   words the original reuses (like `$12`/`$14`) live in the Session's spawn
   scratch or the owning state.
5. **Flip the policy** to `follows`, drop any "Parked:" note, and regenerate:
   `python tools/oracle_patch_policy.py`.
6. **Prove it** (`sm-check`):
   - the row passes on both oracles;
   - the local corpus rows and a chain sample still pass by default;
   - known faithful divergences in `docs/LEGACY.md` don't grow.
7. **Clean up.** If the row has a LEGACY entry, delete it.

Commit as `(behaviour: $BB:AAAA follows)`, one site per commit.

## When you can't fix it yet

Keep the row on lockstep. List it in `docs/LEGACY.md` with the site, the routine,
the row and tick, and why. Give the row an `unlocked = "..."` note in its scenario
file.

## Notes

- Open-bus reads: the faithful oracle models the CPU data bus. Reads of unmapped
  addresses return the last bus byte (often the operand's high byte).
- Stores to ROM are ignored by the cartridge. Writes the original makes to ROM
  addresses become no-ops in the engine.
- A fix that changes what the faithful oracle outputs (rare: only oracle-side
  code) must re-record the affected lock entries in the same commit
  (`python tools/scenario.py lock --record`).

Done when the site is `follows`, the row passes faithful, and nothing else regressed.
