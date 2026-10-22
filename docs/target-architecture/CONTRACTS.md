# Ownership, execution, and lifecycle contracts

These are implementation contracts for the agreed design. Rust sketches describe the intended interface; they are not an implemented SDK. Capacities and scheduling defaults below are starting policies to exercise in the vertical slice, not measured performance guarantees.

These contracts describe the target packages. Current Windows work may retain the emulated backend under [Migration](MIGRATION.md). A temporary adapter is a migration step with disclosed dependencies; it is not yet evidence that the target contract is complete.

## Public gameplay interface

Every gameplay operation executes our Rust implementation over owned domain state. Session has no emulator, recovered-C backend, foreign gameplay callback, or C/Rust ownership toggle. Original hardware register operations are expressed as the required gameplay or presentation behavior; synchronous collision queries call our own collision module. Platform libraries remain inside device adapters, while optional reference executables remain outside the game process and normal build.

```rust
pub struct OpenedSession {
    pub session: Session,
    pub initial: InitialOutput, // Committed initial Scene; no startup I/O.
}

pub struct TickInput {
    pub buttons: Buttons,
    pub host_results: Vec<HostResult>, // Bounded; e.g. a save acknowledgment.
}

pub struct TickOutput {
    pub tick: Tick,
    pub scene: Scene,           // Owned, complete observation.
    pub effects: TickEffects,   // Owned audio cues and optional save request.
    pub inspection: Option<Inspection>, // Development build/configuration.
}

impl Session {
    pub fn open(
        gameplay: Arc<ValidatedGameplay>,
        start: Start, // New game, Player save, Scenario, or compatible Checkpoint.
    ) -> Result<OpenedSession, OpenError>;

    pub fn step(&mut self, input: TickInput) -> Result<TickOutput, SessionFault>;
    pub fn checkpoint(&self) -> Result<Checkpoint, SessionFault>;
}
```

`step` advances exactly one logical Tick. Same compatible starting state, gameplay fingerprint, and ordered Tick inputs produce the same canonical state and semantic effects. Host results that can affect game behavior are recorded inputs. Rendering, inspection selection, worker completion timing, and OS device objects cannot independently mutate gameplay.

`open` validates the start against the current gameplay fingerprint and constructs a complete initial state before returning it. `checkpoint` observes a committed Tick and does not advance time or emit effects. No external caller obtains a mutable reference to the Actor store or live player.

Output construction and gameplay storage have explicit implementation limits, checked against compiled content. Distinguish an intentional original-game limit, such as a reference projectile cap, from an engine buffer limit. The former can be a defined gameplay result; exceeding the latter is a diagnostic fault, never silent removal of an Actor or cue. Measure packet allocation and copying before adding pooling.

The contract is commit-on-success, not transactional rollback of arbitrary memory changes: a failed Tick poisons the Session, produces no deliverable output, and cannot be stepped again. Drop it and restore a new Session from the last known committed Checkpoint. Fatal panics are not normal recovery flow. Delivery occurs only after `step` returns success.

## Owners and lifetimes

| Resource | Sole mutable owner | May cross a seam as | Ends when |
| --- | --- | --- | --- |
| Player, Actors, room state, RNG, game timers | `sm_game::Session` | Owned Scene/Inspection/Checkpoint | Session closes; room-local state follows transition policy |
| Validated gameplay definitions | Immutable allocation shared by Session/tools | `Arc<ValidatedGameplay>` | Last compatible consumer releases it |
| Tick admission, input journal, pending effects, timeline epoch | `sm_app::RunCoordinator` | Owned Tick input and identified requests | Run ends or coordinated epoch cutover |
| Compiled candidate content | Content worker, then revision coordinator | Owned validated candidate | Rejected, superseded, or promoted |
| GPU device, pipelines, textures, submissions | `sm_runtime::video` on event-loop thread | Revision lease and stable portable records | All submissions using the resource complete |
| Voice/sequencer state | Audio callback | Bounded commands and lightweight receipts | Voice ends or transport reset is acknowledged |
| Sound-bank allocations | Runtime asset manager | Prevalidated bank IDs/retained references | Callback reports no voices or pending commands use the bank |
| Save bytes and temporary file | Storage worker after acceptance | Owned immutable write snapshot | Atomic replacement completes or failure is reported |
| Workbench selections and edits | Development application | Read-only observations; validated transactions | Tool closes or transaction applies at its declared seam |

Workers do not hold live Session borrows. Audio bank destruction and large reference-count drops happen outside the callback. GPU resource retirement is tied to actual submission completion, not “two frames later” or a pointer becoming unused on the CPU.

The first implementation uses one event-loop thread for Session steps, Scene extraction, input policy, and GPU encoding/submission. A bounded worker pool handles content compilation and file work; a dedicated serialized storage lane prevents save reordering. The audio callback is a separate real-time context. Gameplay parallelism is deferred until a measured need exists.

## A Tick, with explicit commit points

The following schedule is a concrete starting design. Exact phase order and immediate/queued interactions must be calibrated with the selected original game; this table is not a claim that its routines have already been verified.

| Stage | Owner and operation | Visibility rule |
| --- | --- | --- |
| 0. Admit | Coordinator resolves pending effects, latches Buttons and eligible host results | No new Tick while the previous effect outbox cannot be accepted |
| 1. Begin | Session increments the logical step, derives button edges, evaluates game mode | Game pause/menu modes determine which domains advance; Host pause admits no Tick |
| 2. Player | Update movement, pose, weapon decisions, and motion queries | Immediate player contacts resolve here when their rule requires it |
| 3. Commit early actions | Apply the defined early spawns/removals | A new shot is visible to its first eligible phase in this Tick |
| 4. Actors and shots | Evaluate Actor families, boss state, projectiles in total simulation order | Resolve handles through the store; no hash/archetype iteration dependence |
| 5. Contacts | Apply the defined ordered damage/contact interactions and resulting actions | Tie-breaking is explicit and reference-tested |
| 6. Rooms and progress | Advance room objects/sequences, progression, doors, camera, mode transitions | Persistent flags and transition carryover follow one policy |
| 7. Commit and extract | Apply late actions, validate invariants, advance committed Tick, create outputs | Scene and all Runtime effects refer to the same completed Tick |
| 8. Deliver | Coordinator assigns effect identities and offers batches to runtime adapters | Every sink accepts ownership once; later durable acknowledgments are distinct |
| 9. Present | Renderer uses the latest complete Scene; audio consumes its own timeline | Several Ticks may precede one display update; no Tick's effects are skipped |

Do not force all collision work into stage 5 if reference behavior requires an immediate resolution during movement. The schedule owns these distinctions and exposes named commit points. Pending actions are typed, bounded, and carry a target phase/order key; there is no unrestricted event bus.

No actor reference survives despawn without generation validation. Simulation order keys have a documented total ordering, including Actor creation during a Tick. Their rule may encode a reference slot priority when that is observable, but it is separate from the actual allocator. Checkpoints preserve the next generation, order counters, pending actions, and room-incarnation state needed for exact continuation.

## Clock, input, pause, and presentation

Store the Reference profile's cadence as an exact rational `p / q` Ticks per second. Use integer elapsed-time accumulation against this ratio and checked wide intermediates. A monitor refresh is never the simulation step. Bind the ratio to the captured reference and document intentional freezes/slowdown rules separately.

The native adapter timestamps physical input events on receipt. The coordinator latches the state at each scheduled Tick deadline; catch-up Ticks do not all retroactively receive the newest button edge. Replays bypass physical-device policy and supply the recorded Tick inputs directly. Gamepad deadzones apply to mapping physical controls into digital actions, not to invented analog movement.

Start with at most four catch-up Ticks per event-loop turn so events and lifecycle work remain responsive. Retain remaining time debt rather than skipping simulation steps. If debt cannot recover, enter an explicit overload Host pause, diagnose it, and reanchor before resume; do not run an unbounded catch-up loop or quietly vary Tick duration. Reference/offline execution advances every requested Tick without wall-clock pacing.

Host pause releases stale physical input, stops admission, reanchors the host clock, and pauses/reanchors audio. On resume, discard the paused event backlog and compare the current physical Buttons with the Session's last latched Buttons. A press and release entirely during Host pause produces no game input; a button newly held at resume produces a normal pressed edge, and a released button produces its normal released edge on that first admitted Tick. Do not synthesize key repeat. In-game pause is a Game mode: menu and animation Ticks can continue while selected world domains are frozen. These two mechanisms are not interchangeable.

Present the latest native image without interpolating authoritative motion. Repeating or skipping a display presentation does not replay sounds. Display scaling, a high-refresh monitor, and compositor timing cannot alter gameplay or RNG.

## Bounded handoffs and overload

Capacities are configured and measured. These are initial defaults for a bounded desktop game; they are not throughput or latency claims. `max_cues_per_tick` and similar packet bounds are explicit engine/content limits checked by validation and stress scenarios.

| Handoff | Initial capacity | Full policy | Can older work be replaced? |
| --- | --- | --- | --- |
| Coordinator effect outbox | One completed Tick batch with per-sink acceptance bits | Keep the owned batch; stop admitting Ticks until acceptance/recovery | No |
| Scene awaiting presentation | One latest complete Scene | Replace the unsubmitted Scene | Yes; effects are elsewhere |
| GPU work | At most two submitted frames, plus latest unsubmitted Scene | Skip/defer new presentation until capacity returns | Unsubmitted only |
| Audio cue ring | 64 fixed-capacity Tick batches | Return batch unchanged; yield to event loop and retry; diagnose sustained blockage | No |
| Audio control/retirement lanes | Four reserved control records; retirement storage bounded by bank/voice limits | Process reset/stop while normal delivery is backpressured; gate a new reload until retirement drains | No |
| Save work | Two requests total: one active, one waiting | Retain request in outbox; keep event loop responsive | No; preserve slot order |
| Save acknowledgments | Two receipts | Worker retains receipt until coordinator drains it | No |
| Content preparation | One compiling request and one latest pending source revision | Replace the pending request; stale completed candidates cannot publish | Yes, before publication |
| Workbench inspection | One latest observation | Replace old read-only observation | Yes |
| Reference recording | Bounded chunks, initially 64 | Pause Tick admission until writer accepts; report failure | No |

Queue capacity is not scheduled audio lead. A starting target of roughly two Ticks of command lead is measured against actual device buffers; it must be tuned for underruns and control latency. The 64-batch ring provides bounded burst tolerance rather than one second of intended sound delay.

The coordinator moves one complete Tick's effects into its outbox. Each sink returns either acceptance of the owned request or the unchanged unaccepted request. Acceptance means enqueued, not “sound has played” or “save is durable.” Mark each sink accepted before retrying another sink, so a blocked save cannot duplicate an already accepted sound.

Identify each effect by `(timeline_epoch, tick, ordinal)`. Use ordered IDs for retry suppression and acknowledgments within the running process; do not claim distributed exactly-once delivery across crashes. Replaying a timeline uses a new epoch for live devices and either a fresh output file or a defined trace namespace.

If audio is lost, a reset/stop control path cannot be blocked behind ordinary cue traffic. Host-pause admission, recover or explicitly enter a diagnosed no-device sink, then reanchor. A no-device sink accepts the full semantic cue stream for recording even though no sound is physically produced. It is an explicit degraded state, not silent dropping of selected authored sounds. Offline audio rendering retains the full trace and applies no live queue truncation.

## Content revisions and fingerprints

The gameplay fingerprint includes canonical gameplay data, behavior/build compatibility ID, arithmetic and input policy versions, and the Reference profile. It does not depend on file paths, serialization map order, or source timestamps. A release build records reproducible provenance; a changed rule cannot accidentally reuse an old fingerprint.

Media revision identifies visual/sound resources that do not feed back into the Session. Stable authored IDs survive pack reordering. Record-layout/schema versions are explicit. Frames and audio commands carry the appropriate revision lease; a raw integer asset index cannot accidentally resolve against a different pack.

Media-only classification is semantic: hitboxes, collision, animation timing, gameplay sequence timing, and RNG-affecting parameters change gameplay even when the source file is associated with art. Pixel/palette/sample changes can be media-only when their record contracts remain compatible. Sound timing is still independently checked for content fidelity.

Candidate promotion proceeds through `Requested → Compiled → Validated → DevicePrepared → Ready → Active → Retiring`. A failed candidate goes to `Rejected`, with source-located diagnostics. Workers can prepare future material; only the coordinator publishes a revision at a committed Tick seam.

## Lifecycle walkthroughs

### Start and normal play

1. Validate pack bytes, versions, authored references, and Reference profile. Prepare the complete required media set and devices.
2. Open the Session from new game, Scenario, Player save, or compatible Checkpoint. Retain the previous running Session until a replacement has opened successfully.
3. Establish timeline epoch and audio transport anchors. Publish the initial Scene.
4. Admit Ticks, accept each effect batch, and present the latest completed Scene. Record Tick inputs and external results when recording is enabled.

### Room transition

1. Session accepts the door/transition condition in its defined phase and enters a transition Game mode.
2. Build a destination Room instance from validated resident data. The source remains available for any scroll/transition image that needs both rooms.
3. Apply the explicit carryover/reset policy: player state, qualifying projectiles, collected items, room objects, and camera state are each accounted for. A source-room command cannot target a reused destination Actor handle.
4. Commit the destination at the declared Tick. Retire source temporary state when the transition no longer refers to it. The timeline epoch stays the same.

### Media-only reload

1. Compile/classify the change on a worker; retain the currently active revision if validation fails.
2. Prepare new GPU resources and sound banks before advertising readiness. Account for peak memory with both revisions alive.
3. At a committed Tick seam, ensure old Tick effects have been accepted and install the media revision for future outputs. For a bank swap, audio acknowledges its scheduled handoff; the coordinator can Host-pause briefly to synchronize that seam.
4. Preserve the Session and its Tick/epoch. A media change does not rewind history. Old submitted frames and voices retain old resources until completion/retirement; no new cues use the retired bank. Music-bank replacement uses an explicit resume/reset policy, not dangling voice references.

### Gameplay reload or Scenario reset

1. Prepare and validate the complete candidate revision. Compile-time Rust behavior changes rebuild/restart the process instead of loading new code into live objects.
2. Stop Tick admission at a committed seam. Drain accepted effects and settle in-flight save operations; epoch tags cannot cancel a disk write already in progress.
3. Open the named Scenario with the new gameplay. If opening fails, keep the old Session/revision paused and show diagnostics; resume it only with its compatible resources.
4. Start a new timeline epoch, reset/acknowledge the audio transport, swap Session and revision, then resume. Ignore stale old-epoch device/UI results.

There is no initial arbitrary-state migration. If a compatible Checkpoint replacement is added later, finish resource preparation first, then capture the **current** committed Checkpoint at cutover and open from that. An older Checkpoint captured while the old Session continued advancing would lose player actions.

### Save and load

1. A save request contains an owned Player save snapshot from its committed Tick and a stable request ID. Later gameplay changes cannot modify the bytes being written.
2. Serialize and validate the versioned payload. The storage worker writes a temporary file in the destination directory, flushes it, then uses the platform adapter's tested atomic replacement operation; durable success includes the applicable directory/metadata flush guarantees.
3. Report success only after acknowledgment. A failure preserves the previous valid save when replacement has not committed and retains diagnostics for retry. Crash/power-loss guarantees depend on the filesystem and tested platform primitive; do not equate a successful buffered write with durability.
4. If the in-game save flow depends on completion, admit its result through `TickInput::host_results` and record the Tick of delivery. The runtime never mutates the Session from a worker. The exact station/UI timing belongs in the Reference profile tests.
5. Loading is fenced behind earlier writes to the slot. Validate checksum, schema, authored IDs, and compatibility before creating a replacement Session. Switch epoch only after opening succeeds; corrupt or unsupported data leaves the current run intact. Do not silently reset a corrupt slot to a new game.

Writes to a slot are serialized. A reset or load waits for accepted writes to settle before its cutover; it cannot rely on dropping a receipt to prevent the actual file replacement. Backups and migrations are explicit versioned policies, never memory-layout serialization.

### GPU/surface recovery

1. Zero-size, occluded, or temporarily unavailable surface: skip acquisition/presentation while preserving the latest Scene.
2. Stale surface configuration: drop outstanding surface images, configure valid nonzero dimensions, and request redraw. Device resources survive if only the surface changed.
3. Device loss: Host-pause, retain Session and CPU content, recreate device/pipelines/media from the active revision, and invalidate old GPU-generation handles.
4. Resume from the latest committed Scene after preparation succeeds. If recreation fails, remain paused with a runtime diagnostic; do not reset progression or continue on invalid resources.

Map the pinned `wgpu` version's outcomes into stable project-level outcomes such as `Presented`, `Skipped`, `Reconfigure`, and `RecreateDevice`. This avoids encoding an outdated upstream error enum into the architecture.

### Audio recovery

1. Callback failure/underrun diagnostics reach the coordinator through a nonblocking status path. Host-pause and stop/reset the old stream through reserved control capacity.
2. The runtime retains banks and semantic music/ambience transport state outside device handles. Recreate the stream and negotiate its actual sample rate/buffer behavior.
3. Invalidate old device-generation acknowledgments; reset voices as required and reconstruct ongoing music/ambience at the new anchor from retained control state. Echo tails and transient voices are not promised to survive a device failure sample-for-sample.
4. Acknowledge the new transport before admitting further cues. Resume the same gameplay Session/epoch, or enter the explicit no-device state if recovery is unavailable. Device generation and timeline epoch solve different problems.

### Gameplay fault

Stop the poisoned Session, preserve diagnostic inputs and the last committed Checkpoint, and prevent its failed Tick from emitting effects. A new Session may be opened from that Checkpoint after the cause is addressed; restoring creates a new epoch. This is a development/recovery facility, not a loop that repeatedly ignores invariant failures.

## Three persistence formats

| Field | Player save | Checkpoint | Replay |
| --- | --- | --- | --- |
| Inventory, persistent flags, explored map, restart location | Yes, according to original save semantics | Yes | In starting state, then reproduced |
| Live room, Actors, poses, velocity, game mode | No | Yes | In starting Checkpoint or reproduced |
| RNG, input history, timers, sequence cursors, pending actions | No unless reference save semantics require a field | Yes | In starting state, then reproduced |
| Allocator generations, simulation order, room incarnations | No | Yes | In starting Checkpoint or reproduced |
| Tick inputs and admitted external results | No | No | Yes |
| Compatibility metadata | Versioned save compatibility and stable IDs | Exact gameplay fingerprint/schema | Exact gameplay fingerprint/input schema |
| GPU handles, device queues, sample-level mixer/echo state | No | No | No; audiovisual capture is separate |

Canonical state hashes use defined field order, integer encoding, and transient/persistent IDs deliberately. They exclude addresses, padding, GPU state, and unordered container iteration. No raw Rust memory dump is a portable save or Checkpoint.
