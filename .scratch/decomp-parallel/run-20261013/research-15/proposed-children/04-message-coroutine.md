# Own the message coroutine dependency and PLM resume contract

Status: open
Blocked by: proposed15-plm-frame-adapter/implementation; research13 hardware/NMI; 12 controller timing; 18 audio lifecycle; direct message-root/table-asset exercise
Triage: needs-triage
Type: task

Proposal only. Coordinator publishes separate gameplay and infrastructure scopes. The message lane owns `message_box.rs`, its test and manifest; integrator owns the separate adapter/ABI files specified in [children.toml](../children.toml), key `15-message-coroutine`.

## Work

Implement complete `DisplayMessageBox_Async` `$858080` and its22 bank85 functions listed in children. Use actual hardware/audio/controller/NMI dependencies, nested coroutine states0614/0615, save-selection/timers and the C global `message_box_das0l_value` outside WRAM. Seal selected message tables/tile/HDMA assets. A canned callback return is insufficient.

Extend reviewed PLM admission separately for state0613 resume: no duplicate preinstruction/timer/opcode before the await, preserve yielded uint8 result, clear queued message only after completion, restore the bank84 continuation and finish remaining slots. Future `plm_frame.rs` changes require their own exact ownership review.

## Acceptance

Gameplay submission: locked/offline `cargo test -p port --test message_box` at60s. Integration: `family-verify --manifest tools/port/message-box.toml --output out/port/message-box --case-jobs 4` at3600s, with exact argv in children. Require real nested coroutine/full-state/ordered MMIO controls, global-image mutations, unsupported-before-publication/no-retry controls, material direct-root live resumes and independent original-ROM delay/controller comparison.

## Completion evidence

Pending supported source/exercise/dependency scopes and full hardware/control/audio acceptance, actual receipts, cleanup/retention/worktree retirement and authentication. Twenty-two functions are an ownership proposal, not accepted provenance.
