# Own complete observed door transition orchestration

Status: open
Blocked by: proposed15-exercise-contract/PLM implementation; message dependency if observed; 13 NMI/IRQ/DMA/vblank; 12 controller timing; 18 audio; whole room/enemy/player/animated-tiles/palette-FX/HDMA dependencies; exact observed compressed assets; dedicated door adapter
Triage: needs-triage
Type: task

Proposal only. Coordinator publishes this successor after independent dependency scopes. One isolated owner receives `door_transition.rs`, its test and manifest; player modules and shared pipeline/state/ROM/registration remain external ownership.

## Work

Own the19 orchestrators listed under `15-door-transition-successor` in [children.toml](../children.toml), including game-state9/10/11 roots, addressless dispatch and15 selectors. Bind an actual observed complete route, coroutine0612 positions1/2, background command/table/setup/colored-door/PLM closure and exact assets. Split complete dependencies into successive1–32 scopes when helper closure exceeds the cap.

Preserve elevator48-frame/sign-bit semantics, old-room enemies/draw, fades/audio, room loads/decompression, immediate DMA byte order, four-direction NMI/IRQ completion flag, complete PLM invocation and final fade/state8. Preserve camera X-before-Y,11 accepted room-scrolling routines and separate17 ordered room-host flushes. Full door_direction4–7 closes and existing colored-door6-byte second scripts are distinct domains.

## Acceptance

Locked/offline port gate: `door_transition`, `plm_frame`, `camera_scroll`, `room_scroll`, at60s. Integration: `family-verify --manifest tools/port/door-transition.toml --output out/port/door-transition --case-jobs 4`, at3600s; exact argv is in children. Require actual-C complete state/returns/ordered hardware effects, independent unsupported/no-retry controls, four material live subjects and independent original-ROM timing/transition comparison. Dedicated adapters must seal actual supported argv first.

## Completion evidence

Pending direct-root route, owned dependencies, valid compressed-source consumption extents/digests, exact native scope and live/original adapters, verification, cleanup/retention/worktree retirement and authentication. No source extents are inferred from decompressed output sizes and no whole gameplay root is delegated to a host.
