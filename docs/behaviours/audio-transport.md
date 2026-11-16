# Audio transport: bounded behaviour spec

## Header

**Scope.** Sound requests, delayed acknowledgements and the musical progress needed to predict
when replies become visible. This round admits the opening's effect families and a continuously
repeating arrangement of two tracks. Inputs come from the request producers, imported typed
musical content and the platform's elapsed time. The platform supplies request presentation and
reply observation phases. Sequencer note output, samples and mixing belong to their own domains.
The selected arrangement is already resident; music selection/queue producers are outside
this round. No new selection request occurs while sound-group requests change. The resident
arrangement supplies an independently initialized `ResidentMusicReply` identity. Each music
publication carries that identity, whose original reply byte is preserved at the boundary.
It owns the sender groups and physical reply scheduling. The current engine has no sound queue
or modeled reply transport to replace.

**Notation.** A service turn is one opportunity to advance a sender group; a music turn is one
opportunity to progress the admitted song. Musical duration counts music turns. Loudness has
256 fractional units per whole unit. Timing quantities use exact `AudioCycles`: one audio instruction cycle spans 24 oscillator ticks.
Fractional instants use exact rational AudioCycles.
[Invented vectors](../../scenarios/vectors/audio-transport.toml) contain no game content.
Every checked label below uses the ByVectors convention: these are constructed, source-expected
cases exercised by the reference. Native paired capture evidence is described separately; a vector label does not claim that the native capture reached its case.

## Saved quantities

Every quantity below can be saved and restored. A saved value must satisfy its stated invariant;
restoration never advances time, repeats an input or publishes a reply.

| Quantity | Domain type and units | Read by |
| --- | --- | --- |
| sender queue | bounded sequence of `CueId`, oldest first | `RequestDelivery` |
| sender progress | `Ready`, `AwaitCue(CueId)`, `ClearDelay(Turns)`, `AwaitClear`, `ResetDelay(Turns)` | `RequestDelivery`, `RequestRelease` |
| resident music reply | `ResidentMusicReply`, supplied with the resident arrangement and unchanged throughout this boundary | `VisibleCompletion`, `ReplyCadence`; physical music output |
| last published reply | `Reply` per `SoundGroup` | output; `RequestDelivery`, `RequestRelease` |
| staged request | `Absent` or `Cue(CueId)`, with handled status | `StagedRequests` |
| current instant | exact rational `AudioInstant`, monotonically advanced | `VisibleCompletion`, `ClockObservation`, `ReplyCadence` |
| polling lattice | exact phase of eight-AudioCycle observation opportunities | `ClockObservation`, `ReplyCadence` |
| stable sample | pending first observation, paired-read phase and retained publication anchor | `StableRequests`, `ReplyCadence`, `VisibleCompletion` |
| accrued time | `AudioCyclePhase`, pending four-bit overflows and `CadencePhase` | `IndependentCadence`, `ClockObservation` |
| pending completion | `Completion` carrying remaining work and semantic result | `VisibleCompletion`, `SenderTiming` |
| song progress | `TrackPosition` in validated root or repeated phrase, with `RepeatCount` from one through 255 and root continuation | `FinitePhrases`, `WholeArrangement` |
| note duration and release | `MusicTurns`, positive duration and nonnegative remaining counts | `MusicalProgress`, `MusicTiming` |
| effect release history | `ReleaseHistory`, one entry for Action/Ambient and two for Contact; Inactive or Active `EffectTurns` from one through 255 | `EffectResources`, `EffectTiming` |
| loudness progression | `Loudness`, optional `LoudnessRamp` with one through 255 remaining turns, target and signed increment | `LoudnessProgress` |
| song controls | `SongMix`: `TimbreId`, `PitchOffset`, `NoteStrength`, `EchoGain`, `EchoDelay`, `EchoFilterId`, `PercussionSetId`, voice membership and enabled status | `SongControls`, `MusicTiming`; output to sequencer boundary |
| effect ownership | set of `VoiceId` and typed effect progress | `EffectResources`, `ConnectedPitchMotion`, `EffectTiming` |

The imported graph contains root phrases and nonnested finite repeated phrases. Every path
through consecutive controls ends in a timed musical event or root completion. Every repeated
phrase has a timed event, positive repeat count and a root continuation. Every root, including a root with no repeats, starts with a finite prefix of scalar or
Timing choices followed by Tone or Sustain before any repeated-phrase entry or root
completion. Tone or Sustain ends that introduction; Timing alone does not. A timed event
occurring only after earlier completion does not admit the root. This entry rule applies
at event zero after arrangement start or restart and prevents a zero-time repeating
arrangement [A-root-empty, A-root-scalar-only, A-root-unreachable-tone,
A-root-early-repeat, A-root-no-repeat, A-root-timed-before-repeat]. All timbre
references resolve to the admitted non-noise instruments; absence is not an instrument choice.
The importer checks these properties before content reaches the model. This round has one
fixed echo-delay setting in its admitted content, and the imported initial setting agrees with
it. Saved echo delay equals that setting; external echo edits are outside this input boundary.
Saved timbre choices belong to the resolved non-noise set for their track. Effect ownership
never includes either of the two music voices; the admitted effect-resource bound preserves
that separation. These invariants constrain every State chosen by a caller, not merely captures.

Allowed track positions are completed semantic event boundaries. A saved root position may
name terminal completion: completing there restarts once into both admitted timed
introductions, then performs the final volume pass. It does not invalidate the root's
content or consume its completion again during that restarted pass. A duration choice carries
its optional articulation together as one musical timing choice. A Timing choice directly
precedes a tone or sustain; scalar choices occur before that Timing choice. A position cannot enter that
articulation parameter independently; the importer bundles it with its duration choice. A repeated-phrase position carries a positive
repeat count and valid root continuation; a root position carries neither. Active notes carry
positive remaining duration. Loudness spans zero through 65535 fractional units, ramp targets
zero through 255 whole units, and stored increments minus 32768 through 32767 fractional units
per music turn. Admitted musical pitches span 19 through 51 semitones; the fractional pitch
offset is zero. Pending completion carries its completed state and a result awaiting publication;
it does not expose a half-applied result as committed state. Caller-defined State obey these
invariants; they are not restricted to states encountered in a captured row.

## Queue offers

### OfferAcceptance

- id: `OfferAcceptance`.
- kind: Mechanic.
- summary: admit a sound offer only while its producer permits sound and its queue allowance has space.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected Q-accept, Q-full, Q-disabled, Q-demo, Q-explosion).

**Rule.**

The caller supplies a typed queue allowance: one, three, six, nine or fifteen cues. The saved queue holds at most fifteen cues; an offer with a smaller allowance than its current length never removes already accepted cues. Existing requests retain their order.

- Sound enabled, outside demo playback and explosion suppression, with queue length below the offered allowance → append the cue to the queue [Q-accept].

- Sound disabled, demo playback, explosion suppression or queue length at least the offered allowance → leave the queue unchanged [Q-full, Q-disabled, Q-demo, Q-explosion].

## Request delivery

### RequestDelivery

- id: `RequestDelivery`.
- kind: Quirk.
- summary: deliver the oldest queued cue and wait for its acknowledgement.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected S-empty, S-send, S-retry, S-ack).

**Rule.**

- Ready with an empty queue → retain Ready and present nothing [S-empty].
- Ready with queued cues → remove and present the oldest cue, retaining AwaitCue for that cue [S-send].
- AwaitCue observes a different reply → retain AwaitCue and present the same cue again [S-retry].
- AwaitCue observes the matching reply → retain ClearDelay with two turns remaining; consume none of the new delay in this service [S-ack].

### RequestRelease

- id: `RequestRelease`.
- kind: Quirk.
- summary: clear a handled cue before sending the next one.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected S-delay, S-clear, S-clear-retry, S-clear-empty, S-clear-send, S-reset-wait, S-reset-expire).

**Rule.**

- ClearDelay has more than one turn → consume one turn and present nothing [S-delay].
- ClearDelay has one turn → present absence and retain AwaitClear [S-clear].
- AwaitClear observes a nonempty reply → present absence again and retain AwaitClear [S-clear-retry].
- AwaitClear observes absence with an empty queue → become Ready in this service [S-clear-empty].
- AwaitClear observes absence with queued cues → become Ready and deliver the oldest cue in this service [S-clear-send].
- ResetDelay has more than one turn → consume one turn and present nothing [S-reset-wait].
- ResetDelay has one turn → become Ready and leave queued cues for the next service [S-reset-expire].

### SenderTiming

- id: `SenderTiming`.
- kind: Timing.
- summary: a sound request and its acknowledgement become observable during one sender service.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected S-empty, S-send, S-retry, S-ack, S-delay, S-clear, S-clear-retry, S-clear-empty, S-clear-send, S-reset-wait, S-reset-expire).

**Rule.**

`CpuMasterTicks` count the platform's CPU master clock. They are separate from `AudioCycles`.
This boundary admits the platform's validated `FastBus` sender timing profile.
The platform supplies that profile after checking its execution width, memory access class and
queue layout invariants. It begins a sender service only while delivery is active. Caller scheduling and
platform stalls supply elapsed time outside the intrinsic sender work. Every offset below
starts at the beginning of this sender service. `none` means there is no such observation.

Cells: reply observation offset, presentation offset and completion work, in CpuMasterTicks.

| Service outcome | Reply observation | Presentation | Completion |
| --- | ---: | ---: | ---: |
| Ready stays empty | none | none | 122 |
| Ready delivers a cue | none | 212 | 398 |
| AwaitCue retries | 58 | 104 | 162 |
| AwaitCue accepts its reply | 58 | none | 210 |
| ClearDelay continues | none | none | 116 |
| ClearDelay presents absence | none | 100 | 218 |
| AwaitClear retries absence | 58 | 104 | 162 |
| AwaitClear accepts absence, queue empty | 58 | none | 252 |
| AwaitClear accepts absence, delivers a cue | 58 | 342 | 528 |
| ResetDelay continues | none | none | 116 |
| ResetDelay expires | none | none | 142 |

- A sender service begins in its saved progress variant → retain a pending service, expose its named reply observation and presentation at the table's offsets, and commit its resulting progress at completion [S-empty, S-send, S-retry, S-ack, S-delay, S-clear, S-clear-retry, S-clear-empty, S-clear-send, S-reset-wait, S-reset-expire].

A reply-dependent result uses the reply supplied at the actual observation instant. It cannot
choose that result at service start. A reply update at that instant is visible to the observation.
Stopping before an observation does not request a reply. Stopping before a presentation does
not expose it; stopping before completion retains the prior committed sender state and its
pending result. Resuming or restoring the pending service observes, presents and completes
each milestone once. Queue offers occur between sender services. Successful clear acceptance
and the next cue's presentation share one service, with no extra caller dispatch.

## Staged sound requests

### StagedRequests

- id: `StagedRequests`.
- kind: Mechanic.
- summary: handle a stable sound request before publishing its reply, then sample the next request for a later turn.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected D-new, D-same, D-absence, D-protected, D-after-absence).

**Rule.**

A group retains the last handled request separately from the stable staged request. Absence releases the request line; silence is an admitted sound cue with its own voice lifetime. The admitted cues are footsteps, spinning, ending a spin, landing or wall jumping, firing the power beam, silencing Samus, silencing background sound and opening Ceres's door. Imported voice maxima are one, one and two for the three request groups. Door playback is protected against another ordinary cue in its group until it finishes.

- A changed present request is admitted by an unprotected group → release the group's previous voice ownership, replace its playback with a prepared cue for a later turn and mark the request handled [D-new, D-after-absence].

- A staged request equals the handled request → continue playback without restarting it [D-same].

- A changed request is absence → mark absence handled and continue the active sound [D-absence].

- A changed present cue targets protected door playback → retain that playback and mark the new request handled, dropping its start [D-protected].

## Musical progress

### MusicalProgress

- id: `MusicalProgress`.
- kind: Mechanic.
- summary: time the next musical event and its release.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected M-count, M-start).

**Rule.**

Duration is one through 255 music turns. Articulation ratio is zero through 255 units out of
256. Remaining release time is nonnegative. A sounding event uses the retained duration and
articulation; a sustain retains pitch rather than retriggering it.

- An active note has more than one turn remaining → subtract one note turn and reduce release time by one down to a minimum of zero [M-count].
- An active note has one turn remaining → advance through the next controls and start the next timed event with retained duration and release time `max(1, floor(duration*articulation/256))`; a tone replaces pitch [M-start].

### FinitePhrases

- id: `FinitePhrases`.
- kind: Mechanic.
- summary: a finite phrase repeat returns to its enclosing musical phrase.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected M-repeat, M-return).

**Rule.**

- A repeated phrase finishes with more than one repeat remaining → subtract one repeat and continue at that phrase's beginning [M-repeat].
- A repeated phrase finishes with one repeat remaining → finish that repeat and continue at the retained root continuation [M-return].

### WholeArrangement

- id: `WholeArrangement`.
- kind: Mechanic.
- summary: the first completed root restarts the repeating arrangement.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected M-restart).

**Rule.**

- Either track completes its root → restart both tracks at their roots, clear their finite-repeat and loudness-ramp progress, and consume the new root introductions in the same music turn; retain scalar controls until an introduction replaces them [M-restart].

### LoudnessProgress

- id: `LoudnessProgress`.
- kind: Mechanic.
- summary: interpolate loudness for a finite number of music turns.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected M-ramp-up, M-ramp-down, M-ramp-last, M-ramp-wrap-up, M-ramp-wrap-down, M-immediate, M-immediate-under-ramp).

**Rule.**

- An immediate loudness choice V is consumed → replace loudness with `256*V`, retaining any active ramp configuration and remaining count [M-immediate, M-immediate-under-ramp].
- A new ramp with positive length N and whole-unit target T starts at loudness V → set signed increment to the signed 16-bit wrapping of truncation toward zero of `256*(T-floor(V/256))/N`, with N turns remaining [M-ramp-up, M-ramp-down, M-ramp-wrap-up, M-ramp-wrap-down].
- A ramp has more than one turn → subtract one turn and add its signed increment with 16-bit wrapping [M-ramp-up, M-ramp-down].
- A ramp has one turn → finish the ramp and set loudness exactly to 256 times its target [M-ramp-last].

Volume advances after the final track pass of a music turn. A pass abandoned by an arrangement
restart does not advance volume. Introductory changes take effect before that final advance.

## Effect playback and resources

The three `SoundGroup` choices are `Action`, `Ambient` and `Contact`, with one, one and two
active parts respectively. Action admits spinning, ending a spin, the power beam and Samus
silence. Ambient admits background silence. Contact admits footsteps, landing or wall jumping
and the door. A typed cue belongs to its group, so a caller cannot offer it to another group.

An imported `EffectClip` has ordered parts. Each part plays a finite sequence of spans once,
or an introduction followed by a nonempty repeated span sequence. Every span carries a positive
`EffectTurns` duration, a pitch from zero through 71 semitones and an admitted `EffectTimbreId`.
The timbre resolves the ordinary instrument-work class admitted by this boundary. Each door part has one finite span. That span
also carries an upward connected pitch slide: a nonzero speed below 256 fractional semitone units
per effect turn and a target above the starting pitch, no higher than 71. Loop entry and loop
return have retained content positions; loop return resumes after loop entry. Imported content
admits at most one timing choice before a span.

A part retains one of prepared playback, sounding playback, release delay or finished playback.
Sounding playback carries the current span, positive instruction countdown and pitch motion.
A slide carries its current semitone, fraction from zero through 255, speed and target; it is
active only below its target. Prepared playback has consumed no first span and has no active
pitch motion. A release delay carries a positive release count up to 255 and an instruction
countdown from zero through 255. Inactive release memory is absence, not a stale count. Parts
retain their identity within their clip after another part finishes.

`VoiceId` identifies either music voice by its arrangement track or an ordered voice in the
six-voice effect pool. Those two music voices and six effect voices form the shared identity
universe. Constructed fixture labels zero and one name the first and second music voices;
labels two through seven name the six effect-pool voices in their order. Existing effect
fixture labels retain those identities. Effect ownership is
disjoint between active parts and external reservations; the combined set contains at most six
voices and leaves enough free voices for a pending clip's parts. A finished part owns no voice.
Each group retains release history for every ordered part it admits: one for Action, one
for Ambient and two for Contact. History is a semantic input and output independent of the
current clip and its active voices. Each entry is Inactive or Active with a positive release
count through 255. Dormant history owns no voice and has no autonomous progress. Its value
remains meaningful because it can determine a later part's first playback turn. A pending
completion contains the complete resulting history and echo membership, separately from
live state. Echo membership describes semantic selection, not a DSP write at the same instant.

### EffectResources

- id: `EffectResources`.
- kind: Mechanic.
- summary: prepared sounds acquire free voices and keep them until playback finishes.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected E-door-deferred, E-door-direct, E-door-wrap, E-door-prenote, E-second-part, E-loop, E-door-occupied, E-action-occupied, E-contact-occupied, E-ambient-occupied, H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation).

**Rule.**

- A group is initialized from admitted history → retain every ordered entry, including entries absent from current playback [H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count].
- A changed cue is prepared or replaces old playback → retain the entire group's history, including entries absent from the new cue; ownership release does not erase it [H-two-one-two-replacement, H-dormant-two-release-count].
- A prepared clip reaches its allocation turn → give its ordered parts the highest available voices, read inherited release history by part order and advance each allocated part once; remove exactly the acquired voices from completed echo membership, preserving unrelated members, echo gains and echo-write enable [E-door-direct, E-door-deferred, E-door-wrap, E-door-occupied, E-action-occupied, E-contact-occupied, E-ambient-occupied, H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation].
- A part is visited in an effect turn → update only its own history entry to the resulting release state; an unvisited or already finished part leaves its entry unchanged [H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count].
- A part or the last voice finishes → retain the complete resulting history and leave echo membership unchanged; releasing ownership does not restore echo selection [H-one-finished-two, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation].
- An allocation completion is unfinished or restored → retain live history and echo membership until publication, preserve the complete pending result and start no second allocation [H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation].
- A part's instruction countdown is nonzero after eight-bit subtraction by one → retain its span, and advance its pitch only if a connected slide is active [E-door-prenote, E-door-direct, E-door-nonterminal].
- An instruction countdown expires with active connected pitch motion → consume a one-turn release immediately and start the next span, or finish the part when its clip has ended [E-door-finish].
- An instruction countdown expires with no active slide and no release delay → begin release delay, retaining instruction countdown one and release count one after this turn's countdown [E-door-deferred, E-loop].
- An instruction countdown expires with an existing release count above one → subtract one release turn and retain instruction countdown zero [E-door-wrap].
- An instruction countdown expires with release count one → finish release and consume the next span, or finish the part if its finite clip has ended [E-door-deferred, E-loop, E-finish].
- A span starts → retain its positive duration, reset the pitch fraction, clear its own release-history entry, start the span's connected pitch motion and advance that motion once in the same turn [E-door-direct, E-door-deferred, H-two-one-two-replacement, H-dormant-two-release-count].
- A repeated sequence reaches its end → resume at its retained loop span without consuming loop entry again [E-loop].

A retained zero instruction countdown wraps to 255 on the next turn. It delays the next release
observation until that countdown next expires [E-door-wrap]. A new span clears release delay. Finishing a part releases its voice; finishing the last part
also releases the door's protection [E-finish, E-door-finish].

### ConnectedPitchMotion

- id: `ConnectedPitchMotion`.
- kind: Mechanic.
- summary: a connected sound moves upward in pitch until its target and retains the final fraction.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected E-door-direct, E-door-nonterminal, E-door-prenote).

**Rule.**

- Active motion advances by speed S from pitch P and fraction F → set `sum=F+S`, retain `sum mod 256` and add `floor(sum/256)` semitones; reaching the target ends motion without clearing the final fraction [E-door-direct, E-door-nonterminal].
- Motion is inactive before the first span or after reaching its target → leave pitch and fraction unchanged [E-door-prenote, E-door-direct].

### EffectTiming

- id: `EffectTiming`.
- kind: Timing.
- summary: effect work determines when a group's reply and playback become visible.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected E-door-deferred, E-door-direct, E-door-nonterminal, E-door-wrap, E-door-prenote, E-second-part, E-loop, E-door-occupied, E-action-occupied, E-contact-occupied, E-ambient-occupied, H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation).

**Rule.**

All work uses AudioCycles. `A` counts parts visited alive in this turn, including a part that
finishes. A part keeps its physical identity when another part has finished; a second part's
start retains the second-part cost below. `O=floor(pitch/12)` is the resulting pitch's octave.
`S` counts occupied voices encountered while choosing highest free voices, including occupied
voices after the final assignment and before the next free voice. Choice stops at that next
free voice, so lower unvisited reservations contribute no work.

Cells: group base, preparation addition, replacement addition, playback addition and allocation addition.

| Group | Base | New preparation | Releases old active ownership | Playback | Allocation |
| --- | ---: | ---: | ---: | ---: | ---: |
| Action | 60 | 188 | 1433 | `59+19*A` | `494+300*A+16*S` |
| Ambient | 86 | 170 | 719 | `37+19*A` | `278+300*A+16*S` |
| Contact | 88 | 197 | 719 | `37+19*A` | `278+300*A+16*S` |

Samus silence subtracts eight from its preparation addition. A changed request handled without
preparation adds eight for absence or 36 for a present request. Empty groups add no playback
or allocation work. Active groups add playback work; an allocation also adds allocation work.

Cells: part activity work, instrument octave, part identity, consumed pitch motion and consumed loop boundary.

| Part outcome | Additional work |
| --- | ---: |
| Hold without active pitch motion | 0 |
| Start release delay | 64 |
| Continue release after an expired instruction countdown | 34 |
| Start a span | `1298-14*O`, minus six for the second part |
| Consume connected motion at span start | 102 plus that turn's pitch-motion work |
| Consume loop entry | 94 |
| Return to the loop span | 44 |
| Finish a part | 463, plus fifteen when it finishes the last part visited alive, minus 66 when connected pitch motion was active as its instruction countdown expired |
| Active slide, fractional step | `464-14*O` |
| Active slide, nonterminal carry | `479-14*O` |
| Active slide, terminal carry | `489-14*O` |

Ambient span starts and holds add twelve work units. Pitch-motion work occurs once, including
on the terminal step; an inactive hold adds none. Loop entry and return work occur only when
that boundary is consumed, never on a hold.

- A sound group has an effect turn → sum its applicable group and part work, retain the completed semantic result including complete release history and echo membership as pending, and expose that result and the group's physical reply only at its completion [E-door-deferred, E-door-direct, E-door-nonterminal, E-door-wrap, E-door-prenote, E-second-part, E-loop, E-door-occupied, E-action-occupied, E-contact-occupied, E-ambient-occupied, H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation].

## Song controls

### SongControls

- id: `SongControls`.
- kind: Mechanic.
- summary: retain musical choices until a later musical event replaces them.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected M-master, M-timbre, M-offset, M-strength, M-percussion, M-echo-mix, M-echo-all-voices, M-echo-space).

**Rule.**

Master loudness and note strength use zero through 255 whole units. A `TimbreId` resolves to
an admitted instrument for its track; a `PercussionSetId` names a retained percussion choice.
The current content contains no sounding percussion event. A `PitchOffset` uses fractional
semitone units and is zero in this boundary. Echo voice membership uses a set of the same shared `VoiceId` identities and admits
any subset of all eight voices, including both music voices. Membership may name a voice
that is silent or externally reserved; selecting echo membership does not allocate, release
or change effect ownership. Echo membership and effect reservations are separate quantities.
Admitted echo gains and feedback use nonnegative values zero through 127. An `EchoFilterId`
resolves to an admitted filter; echo delay remains the arrangement's fixed `EchoDelay`.

- A master loudness choice is consumed → retain 256 times its whole-unit value as master loudness [M-master].
- A timbre choice is consumed → replace that track's retained instrument choice [M-timbre].
- A fractional pitch offset is consumed → replace that track's retained offset [M-offset].
- An articulation choice is consumed with its duration → replace that track's retained release ratio and note strength together [M-strength].
- A percussion set choice is consumed → replace the retained percussion set [M-percussion].
- An echo mix choice is consumed → replace echo voice membership and both gains, each scaled by 256, and enable echo writes; it may select previously removed voices and preserves effect ownership and external reservations [M-echo-mix, M-echo-all-voices, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation].
- An echo space choice is consumed → retain its fixed delay and replace echo feedback and filter [M-echo-space].

Choices from a consumed introduction take effect in the same music turn. A later choice in
that turn replaces an earlier one. Track-specific choices affect only that track. Previewing
upcoming musical content replaces none of these quantities.

## Musical work

### MusicTiming

- id: `MusicTiming`.
- kind: Timing.
- summary: musical progress and upcoming pitch determine when the resident arrangement replies.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected W-count, W-released, W-preview, W-tone, W-sustain, W-timing, W-extra-three, W-extra-seven, W-ramp-equal-fraction, W-ramp-down, W-root-first, W-return-first, W-root-second, W-prior-tone, W-preview-timing, W-preview-articulation, W-preview-sustain, W-preview-timing-sustain, W-preview-articulation-sustain, W-preview-control, W-preview-repeat, W-preview-again, W-preview-return, W-preview-root).

**Rule.**

All work uses AudioCycles. One music turn has common work 651 plus its final tracks' progress
and volume work. It also includes progress work already completed before an arrangement
restart, and that restart's work. A restart discards a pass's volume work. Both root introductions
are consumed in the restarted pass; the first timed event ends each track's introduction.

Cells: track progress, note duration, release time and preview work.

| Track progress | Work |
| --- | ---: |
| More than one note turn, release already zero | 144 |
| More than one note turn, release remains positive and no preview is due | 159 |
| More than one note turn, release is positive and either one or note duration is three | 159 plus preview work |
| One note turn remains | 159 plus consumed-content work |

A consumed-content passage starts with 23 work units, then adds the work below for each
consumed semantic event. `O=floor(pitch/12)` is a tone's octave. A ramp is downward only when
its target is below `floor(current loudness/256)`; fractional loudness never changes that choice.
An immediate loudness choice changes the value used by a following ramp in the same passage.

Cells: consumed semantic event work, optional articulation, content handling and pitch octave.

| Consumed event | Work |
| --- | ---: |
| Timing | 38 plus its extra handling work |
| Timing's optional articulation | 71 |
| Tone | `467+14*(6-O)` |
| Sustain | 0 |
| Immediate loudness | 130 |
| Start a loudness ramp, target below retained whole loudness | 271 |
| Start a loudness ramp, target at or above retained whole loudness | 256 |
| Master loudness | 123 |
| Timbre | 334 |
| Fractional pitch offset | 122 |
| Percussion set | 120 |
| Echo mix | 198 |
| Echo space with the retained fixed delay | 599 |
| Enter a finite repeated phrase | 233 |
| Finish a repeat with more repeats remaining | 80 |
| Finish the final repeat and return to its root | 102 |

A Timing event owns a nonnegative whole count of AudioCycles named `ExtraHandlingWork`.
`AudioInstant` may be rational; this immutable event-work count is integral. The importer
assigns that immutable quantity; consuming the event adds it exactly once, including after a
repeat, return or arrangement restart. Preview and request retries consume no Timing event
and add none of its extra work. The optional articulation belongs to the same Timing event.

Preview follows the next semantic path to a tone, sustain or root completion without changing
any musical quantity. It starts with fifteen work units when release expires in this turn, or 21 when note duration
is three and release remains positive after this turn. It adds the following traversal work.
A Timing choice replaces the ordinary terminal-read cost with its Timing and terminal costs.

Cells: previewed semantic event, optional articulation, phrase continuation and terminal kind.

| Preview path contribution | Work |
| --- | ---: |
| Scalar control | 45 |
| Enter a repeated phrase | 59 |
| Finish a repeat and begin another | 52 |
| Finish the final repeat and return | 50 |
| Tone reached without a preceding Timing choice | 70 |
| Sustain reached without a preceding Timing choice | 18 |
| Timing choice | 22, plus fourteen for optional articulation |
| Tone reached through that Timing choice | 58 |
| Sustain reached through that Timing choice | 6 |
| Root completion | 61 |

Cells: final-pass volume work and remaining ramp count.

| Final track's volume outcome | Additional work |
| --- | ---: |
| Active ramp retains at least one turn after its advance | 357 |
| Active ramp finishes on this advance | 344 |
| No ramp, a tone was consumed in this pass | 310 |
| Neither ramp nor new tone | 0 |

A root completion contributes 923 work units when reached directly, or 1025 when the final
repeat completes immediately before it. This replaces that ending track's ordinary progress
work; earlier completed tracks in the abandoned pass retain their full progress work.

- A music turn advances the resident arrangement → sum common, completed-pass, final-track and restart work, retain its completed result as pending, and expose the result and music reply only at completion [W-count, W-released, W-preview, W-tone, W-sustain, W-timing, W-extra-three, W-extra-seven, W-ramp-equal-fraction, W-ramp-down, W-root-first, W-return-first, W-root-second, W-prior-tone].

## Independent cadence and stable requests

### IndependentCadence

- id: `IndependentCadence`.
- kind: Mechanic.
- summary: effect and music turns accrue independently from elapsed audio instruction time.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected C-effect, C-music, C-both, C-discard-eight).

**Rule.**

One timer overflow spans 2048 AudioCycles; an effect turn consumes eight overflows. The admitted song accrues 32 of 256 music phase units per overflow. Saved effect phase is zero through seven, music phase zero through 255. The observed clock batch is a four-bit count; a zero observation keeps waiting without distributing cadence. Reading the count clears only the count, including an empty observation; it preserves the timer phase. While observations disagree, the count continues increasing modulo sixteen. A zero clock observation waits on an eight-AudioCycle polling lattice for another observation. External advance calls accept any nonnegative elapsed interval, including zero and intervals spanning many internal observations. Advance processes all completions at or before its stop; it does not combine a large caller interval into one bounded internal observation. Inputs are ordered by presentation instant, with observation-time updates visible before observation.

- Distributing an observed batch N → take `R=N modulo eight`; add R to effect phase and `32*R` to music phase, schedule one turn for each threshold crossed and retain each remainder modulo its threshold; full groups of eight in the batch contribute no turn [C-effect, C-music, C-both, C-discard-eight].

### StableRequests

- id: `StableRequests`.
- kind: Quirk.
- summary: accept a request after two consecutive equal observations.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected R-stable, R-changing).

**Rule.**

First and second observations occur 4.5 and 9.5 AudioCycles after reply publication.
A retry shifts both observations by fourteen AudioCycles. Input updates at an observation instant
are visible to that observation; between observations, the latest supplied request wins.

- Two observations agree → stage their request for a later turn [R-stable].
- Two observations differ → retain the earlier staged request and retry the observation pair fourteen AudioCycles later [R-changing].

## Completion and timing boundary

### VisibleCompletion

- id: `VisibleCompletion`.
- kind: Output.
- summary: completed work becomes visible once, while unfinished work can be saved and resumed.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected P-effect-before, P-effect-on, P-sample-between, P-sample-on, P-music-before, P-music-on, P-music-alone-before, P-music-alone-on, P-idle, P-effects-alone, T-fifteen, T-wrapped-empty, T-wrapped-one, R-resident-alpha, R-resident-beta).

**Rule.**

`AudioInstant` is an exact rational AudioCycle instant. Request updates are ordered by
presentation instant and remain environment inputs. A pending completion owns its completed
semantic result and remaining work; its prior committed state remains readable until completion.
Reply sampling and physical publication are distinct observable instants. A saved completion
contains every value needed to resume it, including samples already observed and pending results.

- An elapsed advance stops before a pending work completion → retain that completion and prior committed state, publish nothing early and continue accruing the timer [P-effect-before, P-music-before, P-music-alone-before].
- An elapsed advance reaches a pending work completion → commit its result and publish the corresponding physical reply at that instant exactly once; effects publish their staged reply and music carries the supplied resident reply identity [P-effect-on, P-music-on, P-music-alone-on, R-resident-alpha, R-resident-beta].
- An elapsed advance stops between a request's two observations → retain its first observation and the previously staged request [P-sample-between].
- An elapsed advance reaches the equal second observation → stage the observed request for a later turn, without replacing an already published reply [P-sample-on].
- A saved state is restored or an advance has zero elapsed time → resume the same remaining work and observations without repeating a completed milestone [P-effect-before, P-effect-on, P-sample-between, P-sample-on, P-music-before, P-music-on].

An advance accepts any nonnegative elapsed interval, including one spanning many observations.
It processes every due observable milestone through its stop, including milestones exactly at
that stop. The latest request update at an observation instant is visible to that observation.
No new music-selection request occurs throughout this resident-arrangement boundary. Its
absence leaves the supplied resident reply identity unchanged; it is distinct from an absent
effect request. The identity persists through music turns, stable sampling and whole-arrangement
restarts. A music publication resolves that same identity to its original physical byte; no
zero reply is implied by selection absence [R-resident-alpha, R-resident-beta].

### ClockObservation

- id: `ClockObservation`.
- kind: Timing.
- summary: work and unstable requests continue accumulating a four-bit clock count.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected T-fifteen, T-wrapped-empty, T-wrapped-one, P-idle, P-effects-alone).

**Rule.**

Timer phase spans zero up to, but excluding, 2048 AudioCycles. A complete period increments the
pending count modulo sixteen. Reading that count clears it, including after a full wrap, and
preserves timer phase. Pending work and sampling retries both accrue time under the same law.

- A clock observation has a nonempty count → clear that count, retain it as the batch to distribute and complete cadence distribution 26.5 AudioCycles after the observation [T-fifteen, T-wrapped-one].
- A clock observation is empty → clear the elapsed count and keep waiting on the same eight-AudioCycle polling lattice until an observation becomes nonempty [T-wrapped-empty].

### ReplyCadence

- id: `ReplyCadence`.
- kind: Timing.
- summary: due effect and music turns determine the next public reply sequence.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected P-effect-before, P-effect-on, P-music-before, P-music-on, P-music-alone-before, P-music-alone-on, P-idle, P-effects-alone, R-resident-alpha, R-resident-beta).

**Rule.**

All offsets below use AudioCycles. A successful stable sample pair retains its reply publication
as the scheduling anchor, delayed by fourteen for each failed pair. Work completing after that
pair uses the retained anchor; it never publishes before the pair has actually been observed.

- Cadence distribution makes effects due → complete the ordered groups' effect work, with the first publication anchored at distribution and each later publication anchored at the preceding group's publication and retries [P-effect-before, P-effect-on, P-effects-alone].
- Music is due after an effect turn → start its work 57 after the final effect publication and retries, then publish at the music work's completion [P-music-before, P-music-on].
- Music is due without an effect turn → start its work 29 after cadence distribution, then publish at completion [P-music-alone-before, P-music-alone-on].
- Music is not due → publish no music reply; the idle interval completes 874 after the corresponding music-start anchor, with the new clock episode's first observation half an AudioCycle before that completion [P-idle, P-effects-alone].

Each return to clock observations starts a new eight-AudioCycle polling episode. The initial
State supplies the first episode's observation phase. After a music publication, the next
interval completes 501 after that publication plus its sampling retries, and the new episode's
first observation is half an AudioCycle before that completion. Further empty observations
lie on the same episode phase, eight AudioCycles apart; a new episode does not preserve
the preceding episode's phase. Neither a new episode nor an observation resets the timer
phase. An observation clears only the observed four-bit count. An empty clock batch never
distributes cadence or serves a sound group.

In completion fixtures, a positive `next_observe_numerator` names the first future nonempty
timer observation after the case's stop, at that numerator divided by two AudioCycles.
Immediate empty polling opportunities precede that milestone on the same episode's lattice.
The field compares the observable nonempty batch instant; implementations may retain or skip
empty opportunities while preserving their timer-count clearing and phase behavior. A zero
field makes no assertion about a future nonempty observation. P-idle observes that milestone
at 7133/2 and P-effects-alone at 7145/2; their immediate next empty opportunities after the
case stops are 4909/2 and 5433/2 respectively.

## Order that matters

Sound groups publish in Action, Ambient, Contact order. The ordered parts of a clip keep their
identities as other parts finish. Tracks progress in their retained arrangement order; the first
completed root restarts the whole arrangement immediately. Already consumed scalar choices
remain until a restarted introduction replaces them. Final-pass volume advances happen after
both tracks' progress, and an abandoned pass advances no volume. These orders affect publication
instants or the committed result; internal work that cannot be observed is not prescribed.
