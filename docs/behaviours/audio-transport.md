# Audio transport: bounded behaviour spec

## Header

**Scope.** Sound requests, their acknowledgements and the musical and effect progress of the
audio side, one frame at a time. A request travels from the game's sound queues to the audio
side, which plays it and replies; the game reads that reply on a later frame and moves on to its
next request. This round admits the opening's effect families and a continuously repeating
arrangement of two tracks. Inputs come from the request producers, imported typed musical and
effect content, and each frame's audio cadence (`FrameCadence`). Sequencer note output, samples
and mixing belong to their own domains. The selected arrangement is already resident; music
selection producers are outside this round, and no new selection request occurs while
sound-group requests change. The resident arrangement supplies an independently initialized
`ResidentMusicReply` identity, whose original reply byte is preserved at the boundary. The
current engine has no sound queue or modeled reply transport to replace.

**What it leaves to others.** Whether a frame runs the game's sender service at all: a frame
that runs out of time before its end runs none, and the frame's caller supplies that. When
within a frame each audio turn falls: the audio side keeps its own clock, and until that clock
is modeled each frame's turn counts are a borrowed input (`FrameCadence`).

**Notation.** A frame runs from one frame cut to the next. A sender service is the game's one
pass over its three sound queues in a frame. An effect turn serves each sound group once; a
music turn advances the resident song once. Musical duration counts music turns and effect
duration counts effect turns. Loudness has 256 fractional units per whole unit.
[Invented vectors](../../scenarios/vectors/audio-transport.toml) contain no game content.
Every checked label below uses the ByVectors convention: these are constructed, source-expected
cases exercised by the reference, except the vectors whose `source` names a recorded frame.

## Saved quantities

Every quantity below can be saved and restored between frames. A saved value must satisfy its
stated invariant; restoration never repeats an input or publishes a reply.

| Quantity | Domain type and units | Read by |
| --- | --- | --- |
| sender queue | bounded sequence of `CueId`, oldest first | `RequestDelivery` |
| sender progress | `Ready`, `AwaitCue(CueId)`, `ClearDelay(Turns)`, `AwaitClear`, `ResetDelay(Turns)` | `RequestDelivery`, `RequestRelease` |
| request line | `Request` per `SoundGroup`: `Absent` or `Cue(CueId)`, the value the sender last presented | `StagedRequests` |
| staged request | `Request` per `SoundGroup`, taken from the request line at the group's last effect turn | `StagedRequests` |
| handled request | `Request` per `SoundGroup`, the staged request the group last handled | `StagedRequests`, `PublishedReplies` |
| last published reply | `Reply` per `SoundGroup` and for music | output; `RequestDelivery`, `RequestRelease` |
| resident music reply | `ResidentMusicReply`, supplied with the resident arrangement and unchanged throughout this boundary | `PublishedReplies`; output |
| song progress | `TrackPosition` in validated root or repeated phrase, with `RepeatCount` from one through 255 and root continuation | `FinitePhrases`, `WholeArrangement` |
| note duration and release | `MusicTurns`, positive duration and nonnegative remaining counts | `MusicalProgress` |
| effect release history | `ReleaseHistory`, one entry for Action/Ambient and two for Contact; Inactive or Active `EffectTurns` from one through 255 | `EffectResources` |
| loudness progression | `Loudness`, optional `LoudnessRamp` with one through 255 remaining turns, target and signed increment | `LoudnessProgress` |
| song controls | `SongMix`: `TimbreId`, `PitchOffset`, `NoteStrength`, `EchoGain`, `EchoDelay`, `EchoFilterId`, `PercussionSetId`, voice membership and enabled status | `SongControls`; output to sequencer boundary |
| effect ownership | set of `VoiceId` and typed effect progress | `EffectResources`, `ConnectedPitchMotion` |

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
offset is zero. Caller-defined State obey these invariants; they are not restricted to states
encountered in a captured row.

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

- Sound enabled, outside demo playback and explosion suppression, with queue length below the offered allowance → append the cue to the queue [Q-accept, F-offer].

- Sound disabled, demo playback, explosion suppression or queue length at least the offered allowance → leave the queue unchanged [Q-full, Q-disabled, Q-demo, Q-explosion].

## Request delivery

A sender service serves the Action, Ambient and Contact queues once each. Where it presents a
cue or absence, that value replaces the group's request line; where it reads a reply, it reads
the group's last published reply (`PublishedReplies`). Successful clear acceptance and the next
cue's presentation share one service.

### RequestDelivery

- id: `RequestDelivery`.
- kind: Quirk.
- summary: deliver the oldest queued cue and wait for its acknowledgement.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected S-empty, S-send, S-retry, S-ack).

**Rule.**

- Ready with an empty queue → retain Ready and present nothing [S-empty].
- Ready with queued cues → remove and present the oldest cue, retaining AwaitCue for that cue [S-send, F-two-frames].
- AwaitCue reads a different reply → retain AwaitCue and present the same cue again [S-retry, F-two-frames].
- AwaitCue reads the matching reply → retain ClearDelay with two turns remaining; consume none of the new delay in this service [S-ack, F-two-frames].

### RequestRelease

- id: `RequestRelease`.
- kind: Quirk.
- summary: clear a handled cue before sending the next one.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected S-delay, S-clear, S-clear-retry, S-clear-empty, S-clear-send, S-reset-wait, S-reset-expire).

**Rule.**

- ClearDelay has more than one turn → consume one turn and present nothing [S-delay, F-two-frames].
- ClearDelay has one turn → present absence and retain AwaitClear [S-clear, F-two-frames].
- AwaitClear reads a nonempty reply → present absence again and retain AwaitClear [S-clear-retry, F-two-frames].
- AwaitClear reads absence with an empty queue → become Ready in this service [S-clear-empty, F-two-frames].
- AwaitClear reads absence with queued cues → become Ready and deliver the oldest cue in this service [S-clear-send].
- ResetDelay has more than one turn → consume one turn and present nothing [S-reset-wait].
- ResetDelay has one turn → become Ready and leave queued cues for the next service [S-reset-expire].

## Frames and replies

### FrameCadence

- id: `FrameCadence`.
- kind: Timing.
- summary: a frame runs the effect turns its cadence names before and after its sender service, and its music turns.
- params: none.
- switch: Fixed.
- checked: oracle (recorded F-recorded-two-turns, F-recorded-late; constructed/source-expected F-two-frames, F-early, F-lag, F-no-turn, F-offer, F-two-music).

**Rule.**

A frame's audio cadence is three counts, each zero through two: the effect turns that complete
before the frame's sender service, those that complete after it, and the music turns the frame
completes. A frame without a service runs all its effect turns, and its count of turns before a
service is zero. The audio side keeps time on its own clock: its turns come slightly more often
than frames, and where they fall in a frame drifts. Until that clock is modeled, each frame's
cadence is a borrowed input, as a frame without a service is.

- A frame runs → apply its queue offers, complete its effect turns before the service, run the service if the frame has one, then complete its remaining effect turns [F-two-frames, F-early, F-lag, F-no-turn, F-offer, F-recorded-two-turns, F-recorded-late].
- A frame has music turns → advance the song once for each, independently of the frame's effect turns and service [F-two-music, R-resident-alpha, R-resident-beta].

So in the most common cadence, one effect turn after each service, a cue's reply first reaches
the sender two frames after the service that presented it. A turn that moves before the service
on the following frame shortens that to one frame [F-early]. When the presenting frame's turn
comes before its service, the next frame's turn stages the cue and the one after handles it; if
that third frame's turn comes after its service, the reply is read three frames after the
presentation [F-recorded-late]. A frame with no service still runs its turns, so the reply can
be read at the first service after it [F-lag].

### PublishedReplies

- id: `PublishedReplies`.
- kind: Output.
- summary: each effect turn publishes every group's handled request, and each music turn the resident reply.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected D-new, D-same, D-absence, D-protected, D-after-absence, F-two-frames, F-early, F-recorded-late, R-resident-alpha, R-resident-beta).

**Rule.**

- A group's effect turn has handled its staged request → publish that request as the group's reply, absence and a dropped protected cue included [D-new, D-same, D-absence, D-protected, D-after-absence, F-two-frames, F-early, F-recorded-late].
- A music turn completes → publish the resident reply identity on the music line [R-resident-alpha, R-resident-beta].

A reply stays published until the next turn of its line replaces it. A sender service reads the
last reply published before it, in this frame's earlier turns or an earlier frame. No new
music-selection request occurs throughout this resident-arrangement boundary. Its absence leaves
the supplied resident reply identity unchanged; it is distinct from an absent effect request. The
identity persists through music turns and whole-arrangement restarts. A music publication
resolves that same identity to its original physical byte; no zero reply is implied by selection
absence [R-resident-alpha, R-resident-beta].

## Staged sound requests

### StagedRequests

- id: `StagedRequests`.
- kind: Mechanic.
- summary: handle the staged sound request before publishing its reply, then stage the request line for the group's next turn.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected D-new, D-same, D-absence, D-protected, D-after-absence, F-two-frames, F-no-turn).

**Rule.**

A group retains the last handled request separately from the staged request. Absence releases the request line; silence is an admitted sound cue with its own voice lifetime. The admitted cues are footsteps, spinning, ending a spin, landing or wall jumping, firing the power beam, silencing Samus, silencing background sound and opening Ceres's door. Imported voice maxima are one, one and two for the three request groups. Door playback is protected against another ordinary cue in its group until it finishes.

- A changed present request is admitted by an unprotected group → release the group's previous voice ownership, replace its playback with a prepared cue for a later turn and mark the request handled [D-new, D-after-absence].

- A staged request equals the handled request → continue playback without restarting it [D-same].

- A changed request is absence → mark absence handled and continue the active sound [D-absence].

- A changed present cue targets protected door playback → retain that playback and mark the new request handled, dropping its start [D-protected].

- A group's effect turn has published its reply → stage the request line's current value for the group's next turn [F-two-frames, F-no-turn].

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
- checked: oracle (constructed/source-expected M-ramp-up, M-ramp-down, M-ramp-last, M-ramp-wrap-up, M-ramp-wrap-down, M-immediate, M-immediate-under-ramp, M-inactive-config).

**Rule.**

- An immediate loudness choice V is consumed → replace loudness with `256*V`, retaining any active ramp configuration and remaining count [M-immediate, M-immediate-under-ramp].
- A new ramp with positive length N and whole-unit target T starts at loudness V → set signed increment to the signed 16-bit wrapping of truncation toward zero of `256*(T-floor(V/256))/N`, with N turns remaining [M-ramp-up, M-ramp-down, M-ramp-wrap-up, M-ramp-wrap-down].
- A ramp has more than one turn → subtract one turn and add its signed increment with 16-bit wrapping [M-ramp-up, M-ramp-down].
- A ramp has one turn → finish the ramp and set loudness exactly to 256 times its target [M-ramp-last].

Volume advances after the final track pass of a music turn. A pass abandoned by an arrangement
restart does not advance volume. Introductory changes take effect before that final advance. A
ramp starts from the loudness at that point of the turn, so an immediate choice consumed earlier
in the same turn sets its V. An inactive ramp has no configuration: two songs that differ only
in a finished ramp's target and increment continue identically [M-inactive-config].

## Effect playback and resources

The three `SoundGroup` choices are `Action`, `Ambient` and `Contact`, with one, one and two
active parts respectively. Action admits spinning, ending a spin, the power beam and Samus
silence. Ambient admits background silence. Contact admits footsteps, landing or wall jumping
and the door. A typed cue belongs to its group, so a caller cannot offer it to another group.

An imported `EffectClip` has ordered parts. Each part plays a finite sequence of spans once,
or an introduction followed by a nonempty repeated span sequence. Every span carries a positive
`EffectTurns` duration, a pitch from zero through 71 semitones and an admitted `EffectTimbreId`.
Each door part has one finite span. That span
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
remains meaningful because it can determine a later part's first playback turn. Echo
membership describes semantic selection.

### EffectResources

- id: `EffectResources`.
- kind: Mechanic.
- summary: prepared sounds acquire free voices and keep them until playback finishes.
- params: none.
- switch: Fixed.
- checked: oracle (constructed/source-expected E-door-deferred, E-door-direct, E-door-wrap, E-door-prenote, E-second-part, E-loop, E-door-occupied, E-action-occupied, E-contact-occupied, E-ambient-occupied, E-finish, E-door-finish, E-door-nonterminal, H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation).

**Rule.**

- A group is initialized from admitted history → retain every ordered entry, including entries absent from current playback [H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count].
- A changed cue is prepared or replaces old playback → retain the entire group's history, including entries absent from the new cue; ownership release does not erase it [H-two-one-two-replacement, H-dormant-two-release-count].
- A prepared clip reaches its allocation turn → give its ordered parts the highest available voices, read inherited release history by part order and advance each allocated part once; remove exactly the acquired voices from echo membership, preserving unrelated members, echo gains and echo-write enable [E-door-direct, E-door-deferred, E-door-wrap, E-door-occupied, E-action-occupied, E-contact-occupied, E-ambient-occupied, H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation].
- A part is visited in an effect turn → update only its own history entry to the resulting release state; an unvisited or already finished part leaves its entry unchanged [H-two-one-two-replacement, H-one-finished-two, H-dormant-two-release-count].
- A part or the last voice finishes → retain the complete resulting history and leave echo membership unchanged; releasing ownership does not restore echo selection [H-one-finished-two, J-one-allocation-release, J-two-subset-reserved, J-nonmember-allocation].
- A part's instruction countdown is nonzero after eight-bit subtraction by one → retain its span, and advance its pitch only if a connected slide is active [E-door-prenote, E-door-direct, E-door-nonterminal].
- An instruction countdown expires with active connected pitch motion → consume a one-turn release immediately and start the next span, or finish the part when its clip has ended [E-door-finish].
- An instruction countdown expires with no active slide and no release delay → begin release delay, retaining instruction countdown one and release count one after this turn's countdown [E-door-deferred, E-loop].
- An instruction countdown expires with an existing release count above one → subtract one release turn and retain instruction countdown zero [E-door-wrap].
- An instruction countdown expires with release count one → finish release and consume the next span, or finish the part if its finite clip has ended [E-door-deferred, E-loop, E-finish].
- A span starts → retain its positive duration, reset the pitch fraction, clear its own release-history entry, start the span's connected pitch motion and advance that motion once in the same turn [E-door-direct, E-door-deferred, E-second-part, H-two-one-two-replacement, H-dormant-two-release-count].
- A repeated sequence reaches its end → resume at its retained loop span without consuming loop entry again [E-loop].

A retained zero instruction countdown wraps to 255 on the next turn. It delays the next release
observation until that countdown next expires [E-door-wrap]. A new span clears release delay. Finishing a part releases its voice; finishing the last part
also releases the door's protection [E-finish, E-door-finish]. A part keeps its identity when
another part has finished, so a second part's first span starts as the second part [E-second-part].

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
that turn replaces an earlier one. Track-specific choices affect only that track.

## Order that matters

Within a frame: the frame's queue offers, then the effect turns before its sender service, then
the service, then the remaining effect turns. Within an effect turn, sound groups take their turn
in Action, Ambient, Contact order; each group handles its staged request, publishes its reply,
then stages its request line. Within a sender service, the groups are served in the same order.
The ordered parts of a clip keep their identities as other parts finish. Tracks progress in
their retained arrangement order; the first completed root restarts the whole arrangement
immediately. Already consumed scalar choices remain until a restarted introduction replaces
them. Final-pass volume advances happen after both tracks' progress, and an abandoned pass
advances no volume. These orders affect the state at a frame cut or a reply the sender reads;
internal work that cannot be observed is not prescribed.
