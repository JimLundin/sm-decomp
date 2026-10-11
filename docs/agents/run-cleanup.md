# Run cleanup

Complete this process after verification runs, including failed or interrupted
runs, and before resolving a ticket or handing work back. The integration owner
handles cleanup across implementer worktrees.

1. Wait for processes writing to the selected run directories to finish or stop.
2. Clean the finished run or batch using the project utility:

   ```sh
   python3 tools/port_cleanup.py --root <run-or-batch-directory> --apply
   python3 tools/port_cleanup.py --root <run-or-batch-directory> --deduplicate --apply
   ```

   Scope cleanup to the work just completed. `port_batch.py` automatically
   deduplicates finished batches and `port_live.py` removes build intermediates;
   check their cleanup output and run the utility for any remaining directories.
   Packed live runs also depend on the shared store (default `out/evidence`),
   their subject `evidence.json` pointers and retained `sm-evidence` reader.
   Preserve these dependencies; never treat shared store objects as disposable
   run intermediates. Authenticate the finished live verdict with
   `tools/port_authenticate_live.py` after cleanup. See `docs/DECOMP_BATCH.md`
   for extraction and frame inspection commands.
3. Remove integrated implementer Git worktrees and disposable build outputs.
   Preserve exact source files referenced by evidence before removing their
   worktree. Retain only those small source snapshots at the referenced paths,
   without Git metadata or build trees. Reauthenticate retained evidence after
   relocation or worktree removal.
4. Record cleanup commands or automatic-cleanup output, measured bytes reclaimed,
   remaining disk space and any retained evidence dependencies under the ticket's
   `## Completion evidence`. A zero-byte result is valid. If cleanup cannot finish,
   record the reason and remaining action explicitly.

Completion requires finished-run cleanup to succeed and integrated implementer
worktrees to be removed, with required evidence still authenticatable. Preserve
sealed artifacts, verdicts, acceptance pointers and diagnostic failure evidence;
the utility shares identical immutable files without changing their hashes.
For run formats the utility does not support, remove only identified unsealed
intermediates and record what remains.

See [DECOMP_BATCH.md](../DECOMP_BATCH.md) for retention details and utility behavior.
