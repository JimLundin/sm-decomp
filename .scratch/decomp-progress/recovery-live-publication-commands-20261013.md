# Commands for later passing live publication

Status: trail observation finalized and published; composed observations pending.

After the coordinator supplies the exact passing post-cleanup trail receipt,
replace the unresolved evidence path in `progress-live-trails-draft-20261013.toml`
with that absolute retained receipt path. Authenticate the same verdict first,
using the retained exact-source CLI/context selected by the coordinator:

```sh
target/release/sm-pipeline live-authenticate /home/jim/sm-decomp/out/recovery-session/trails-live/1791860457650339558-98848/verdict.toml
target/release/sm-pipeline progress update --root /home/jim/sm-decomp --update /home/jim/sm-decomp/.scratch/decomp-progress/progress-live-trails-draft-20261013.toml
```

This publishes only `HandleProjectileTrails`; the three movement opcodes have no
natural prefix hits. Ticket139 resolution additionally requires the independent
composed gate to preserve native137 admission.

For the full composed gate, first verify the actual successful final root and
its cleanup/authentication receipts. The current run path is provisional until
its verdict passes:

```sh
target/release/sm-pipeline live-authenticate /home/jim/sm-decomp/out/recovery-session/composed-live-release/1791861180321696848-129445/verdict.toml
```

Copy `progress-live-composed-draft-20261013.toml` into separate new observations
for camera (commit 7512b64), room (97afab2), Samus frame (c711fae), bounded attack
interpreter (e947e96), and native137 (692805ce), keeping only that scope's measured
roots and recording exact actual counters/remaining C domain. Replace all
unresolved evidence pointers with passing retained receipts and use full commit
hashes listed in `recovery-live-ticket-audit-20261013.md` / current stage evidence.
Then invoke the maintained command for each concrete finalized update:

```sh
target/release/sm-pipeline progress update --root /home/jim/sm-decomp --update /absolute/path/to/finalized-scope-live-update.toml
target/release/sm-pipeline progress status --root /home/jim/sm-decomp
```

Use the coordinator's retained namespace for authentication when the source
freeze differs from the current checkout. Progress hashes the provided evidence;
it does not replace gameplay acceptance authentication. Do not publish any draft
with pending/placeholder limitations or evidence. A failed replay preserves its
diagnostics and receives no live-exercised observation.
