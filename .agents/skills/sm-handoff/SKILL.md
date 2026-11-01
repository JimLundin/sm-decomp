---
name: sm-handoff
description: Write an sm-decomp lane report or end-of-task handoff the root can act on without asking. Use it for every batch report and when you finish or stop.
---

# Report and hand off

The root picks your commits from the hashes and evidence you send. Anything
missing costs a round trip.

## Batch report

Send it with SendMessage `to: "main"`. It opens with the result in one sentence,
then:

```
Branch <branch>, based on <base hash>. GATE OK <tip hash>.
<hash> <title>        (one line per commit, oldest first, copied from git log)
```

- **Evidence:** each row or room you ran, its mode and its result (`0 failing frames`
  or `lock matches`), plus any mutant result.
- **Ratchet:** `sm_game_ram_sites` and `sm_game_coupled_files`, before and after.
- **Needs from the root:** a bundle path and SHA-256 to upload, or a decision with
  the options laid out.
- **Found:** bugs, stale data, and anything a teammate's work hit.

Every number comes from output you saw, not one you expected. Keep anything
you couldn't run under "Not run", with the reason.

## Handoff

When the task is done, or your context is long, write `HANDOFF.md` in your lane's
folder under `$TEMP/collab/<lane>/`, at most 60 lines:

1. **State:** what landed or was picked, by hash; what's on your branch and not picked yet.
2. **Next:** the remaining work, ordered, each item with its first concrete step.
3. **How:** the scripts you wrote (paths) and the commands that worked.
4. **Traps:** each thing that cost you time, with what to do instead.
5. **Cleanup:** your worktrees and their size, with which are safe to delete.

Then stop your background jobs, and send the root the handoff path in your final message.

## Style

- Lead with the result. Use short sentences and the names the code uses.
- Give file paths, hashes and commands literally.
- Say "I couldn't" or "I didn't" plainly, with the reason.
