# Hardware lifecycle research worker handoff

Status: answered
Blocked by: None for research; gameplay admission requires reviewed contract extensions
Triage: ready-for-agent

Root session: `01a1356e-a8dd-7a80-88b0-1239a6a9b876`.
Worker session: `01a13571-7282-7660-99e1-227a72a2c4ff` (`/root/launch_owners`).
Claim: `(ticket-5b357039d4b20673c7d62a43260abada8f7f4f65d7c26999c08138037691dc12, 785e3c24a0e08cd0ab9815084aade2973c955bfcdbf96b48d00a1862d3771771, 1, decomp-launch-20261013)`.
Assignment receipt: `../receipts/next-1.stdout.toml`.
Source baseline: `40a8489856f36d21925188823e33b520f74d4cb4`.

## Result and next action

[answer.md](answer.md) identifies the complete accepted and missed
`Vector_NMI ($809583)` scheduled body and its ten hardware helpers. All eleven
lexical definitions are retained in [routines.json](routines.json). Native
DMA/PPU execution stays behind synchronous byte MMIO services; producer
gameplay, whole reset, IRQ dispatch and APU acknowledgment remain outside this
boundary. Original NMI scheduling/controller continuation requires separate
evidence.

[children.toml](children.toml) contains review-only proposals for
`13-exercise-contract`, `13-hardware-adapter`, and `13-nmi-implementation`.
The root should attach this Answer to ticket 13, create official child tickets
with precise blockers, and publish finished research through its owned tracker
and shared progress. The current Rust exercise reader accepts only bank 86
projectile diagnostics, and no full pre/post NMI hardware adapter exists.
Neither proposed extension nor gameplay scope is admitted by these files.
Implementation follows held integration 11 and both reviewed extensions.

The gameplay envelope is only three proposed new files: port implementation,
port tests and its family manifest. Shared source wiring, adapter work and
existing host claims remain under the sole integrator. No ticket, queue,
inventory, maintained source, shared registration or Git/index was modified.

## Verification and cleanup

The eleven selected registry definitions and identities were checked while
researching. Final input seals are in [inputs.sha256](inputs.sha256); all 40
inspected source, input and receipt hashes were checked from the project root.
`taplo lint --no-auto-config --no-schema children.toml` passes using installed
Rust Taplo. Proposed Cargo targets are future acceptance gates and were not run.
Existing evidence was read and hashed; its recursive admission readers were
not rerun, so historical gate status is not promoted.

No build, fixture, Live replay, emulator experiment, temporary worktree or
disposable run directory was created. No worker child process is running.
Under `docs/agents/run-cleanup.md`, this no-run research has no finished-run
cleanup target: reclaimed bytes **0**. All authored artifacts in this directory
are retained. Existing input/source/evidence dependencies in `inputs.sha256`
remain at their recorded paths; no shared store or historical run was changed.

Final pre-seal measurement: **221,117,214,720 bytes available** on the project
filesystem; research directory **38,836 bytes** before final handoff and seal
updates. These values are measured observations, not reservations; concurrent
sessions may change free space. [artifacts.sha256](artifacts.sha256) seals
`answer.md`, `children.toml`, `routines.json`, `inputs.sha256` and this handoff;
the manifest itself is excluded to avoid a recursive hash.
