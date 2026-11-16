# Audio transport decode notes

The behaviour spec is audio-transport.md; this document is excluded
from the implementer export. Sources below are observation/source arithmetic, not model State.

## Evidence and scope

The native opening prefix supplies184 named CPU cuts,140CPU events and764SPC publications.
Reference control and an independently authorized repeat-expiry intervention use the same
CPU stimuli. Each owns a loaded seed; no native prestate is injected each turn. Both frozen
publication schedules match764/764 exactly. Both cut projections match736replies and5995
committed song fields;13 pending plans mask77 individually attributed source writers. The expiry capture's SMP execution
frontier differs at34 named CPU cuts; normalized CPU/PPU cut clocks agree184/184. CPU cut clocks and native SMP execution frontiers are distinct quantities; each publication
and semantic cut comparison retains its declared clock domain.

The retained sender proof covers552activities,184cuts,19offers and140CPU events, with its
independent input sequence. Earlier individually reseeded spans are localizers. Captured CPU
clock events are diagnostic stimuli; T1 must supply production workload clocks independently.

## Independent expiry continuation seed

`tools/audio_expiry_seed.py` imports `audio-expiry-post64-seed-v1` from the native expiry
capture's complete `music-after` snapshot. The capture dumps `cut-64` before the paused
repeat-count adjustment, increments its capture number to65, then dumps `music-before`
and `music-after` at the intervention barrier. Every `music-before` device object equals
its cut64 object. The post-adjustment CPU, PPU and DSP objects are unchanged; APURAM
changes only offsets128/130, and SMP_STATE changes only their embedded equivalents176/178.
The independent process-memory before/after snapshots equal the corresponding SMP objects.
The ordinary cut64 observation remains the pre-adjustment expectation.

The importer decodes every retained State field from the post-adjustment APURAM, CPU ports,
native timer prescaler, captured publications and validated source content. It loads no
reference or clean model and never advances one to construct the seed. Typed music positions
come from the observed next-byte pointer, repeated-phrase target and root return pointer,
with Duration/Articulation bundles mapped to their typed cursor. Active effect parts come
from their allocated voice, instruction-list cursor/countdown, pitch/fraction and active slide
registers. Their source span boundaries are checked against the independently imported clip.
Every admitted release-history entry is decoded, including dormant parts. All native effect
reservations belong to the decoded parts, so the external reservation set is empty.
Resident identity and full content stay bound to the independently imported arrangement.

The seed's global time is cut64's1109109 AudioCycles since the loaded native snapshot.
Native PC is at the DSP-update boundary after the resident music pair, with no work pending.
The preceding music publication is289 cycles earlier. The unchanged resident CPU0 request
establishes a stable pair without retries; the source return establishes the next polling
episode's first empty opportunity at publication+500.5, or211.5 cycles after the seed.
The prescaler recurrence agrees independently from loaded state through cuts63/64/65.
The first subsequent `soundEffectsClock` source store occurs406 cycles after the seed,
agreeing with the first nonempty read on that episode and its26.5-cycle distribution interval.
The importer checks both the recurrence and this source-store witness before emitting State.

`tools/audio_expiry_seed_pins.toml` freezes184 source inputs explicitly, including the
cut63/64/65 and intervention device objects, clock/write sidecars, capture code and decode
content. These post-adjustment objects are not assumed to belong to the earlier selected
roster. The private seed schema carries the full typed State, global start/next/last cuts,
publication offset, initial physical replies, content/fixture hashes and those source pins.
Its output remains private or in an encrypted evidence bundle.

Expiry acceptance is two carried cases: the initial seed compares cuts0..64; the independent
post-adjustment seed64 compares cuts65..183. There is no omitted cut, but these cases do not
prove an uninterrupted184-cut execution or a restored continuation that begins before the
intervention and crosses it. Restoration proof belongs separately to each carried case.

## Sources by behaviour

| Entry | Source ownership and evidence |
| --- | --- |
| OfferAcceptance | CPU queue offers, producer allowances and suppression; native enqueue proof |
| RequestDelivery, RequestRelease | bank82 sender entries8A2C/8A55/8A6C/8A7C/8A90; native sender and retained-state proof |
| IndependentCadence, StableRequests | installed sound driver timer observation/distribution and stable port pair; native clock/port sidecars |
| MusicalProgress, FinitePhrases | typed imported two-root/five-phrase graph; music graph native field proof and source note/control arithmetic |
| WholeArrangement | first-root completion, outer arrangement and all-track reload; independent outer arrangement audit |
| LoudnessProgress | song's final volume pass; native scalar/ramp proof and constructed whole-song fault cases |
| EffectResources | sound library allocation and connected Door service; independent Door aggregate source audit |

Source work receipts are detailed in private observer reports. The admission is Forever,
not a general arrangement interpreter. All eight native track
slots are checked; two nonempty roots are imported and six slots are empty. Every event and
timbre reference closes inside the admitted graph/content. No native pointer leaves import.

## Sender intrinsic timing admission

`SenderTiming` uses CPU master-clock ticks, not AudioCycles. The `FastBus` profile is a checked
boundary contract: native mode is non-emulation with M8/X8; sender downtime is zero; group
queue accesses have no indexed page crossing; every opcode and long-indexed ROM table access
in the selected sender path has the six-master-clock fetch class. WRAM and stack accesses
cost eight master clocks, internal steps six and physical-port accesses six. The oracle read
callback occurs four master clocks before port-bus completion; the write callback occurs at
bus completion. These conditions are checked outside the model before this profile is supplied.
The profile is not inferred from a desired output timestamp.

The table's work starts at the selected group decision and ends at its return, including the
successful clear acknowledgement's same-activity queue delivery. Caller entry, dispatch,
group iteration and outer return remain T1 work. Refresh, DMA, HDMA and interrupt stalls are
also T1; they can move wall-clock milestones beyond this intrinsic table. Source timing
arithmetic alone does not establish whole CPU timing or the engine's CPU-event gain.
A pending sender asks for the physical reply at its named read callback, then schedules the
reply-dependent branch. It never uses the future reply to select a result at service start.
Invented vectors cover all eleven source paths, partial observations and restored continuations.

Audio time uses `AudioCycles`: one audio instruction cycle spans 24 oscillator ticks. The
2048-cycle timer period and all transport work use AudioCycles. CPU master-clock conversion
uses the independently admitted platform clock relation, with exact rational arithmetic.

## Mapping

APURAM offsets below are observations only. A track row has stride2 even for one-byte fields.
Raw source cursor addresses decode to validated typed event positions at the importer edge.
The comparison consumes declared semantic quantities, not driver PCs or native command words.

| Semantic quantity | Observed symbol | APURAM offset | Width | Track stride |
| --- | --- | ---: | ---: | ---: |
| note remaining | `trackNoteTimers` | 112 | 1 | 2 |
| release remaining | `trackNoteRingTimers` | 113 | 1 | 2 |
| duration | `trackNoteLengths` | 512 | 1 | 2 |
| articulation | `trackNoteRingLengths` | 513 | 1 | 2 |
| finite repeats | `trackRepeatedSubsectionCounters` | 128 | 1 | 2 |
| pitch | `trackNotes` | 865 | 1 | 2 |
| loudness | `trackVolumes` | 768 | 2 | 2 |
| ramp remaining | `trackDynamicVolumeTimers` | 144 | 1 | 2 |
| ramp target | `trackTargetVolumes` | 800 | 1 | 2 |
| signed ramp increment | `trackVolumeDeltas` | 784 | 2 | 2 |
| timbre | `trackInstrumentIndices` | 529 | 1 | 2 |
| fractional pitch offset | `trackSubtransposes` | 897 | 1 | 2 |
| note strength | `trackNoteVolume` | 528 | 1 | 2 |
| master loudness | `musicVolume` | 88 | 2 | 0 |
| echo left | `echoVolumeLeft` | 96 | 2 | 0 |
| echo right | `echoVolumeRight` | 98 | 2 | 0 |
| fixed echo delay | `echoDelay` | 77 | 1 | 0 |
| echo feedback | `echoFeedbackVolume` | 78 | 1 | 0 |
| echo filter | `i_echoFirFilterSet` | 1201 | 1 | 0 |
| percussion set | `percussionInstrumentsBaseIndex` | 95 | 1 | 0 |

CPU sender queue observations use WRAM base0643. Per group, read index is atbase+group,
write index atbase+3+group, source mode atbase+6+group, current cue atbase+10+group and delay
atbase+13+group; sixteen queue slots start atbase+19+16*group. Queue length is modular write
minus read with capacity15. Convert to oldest-first semantic FIFO. A cue word meaningful only
in AwaitCue and a delay meaningful only in its delay variant are carried by that variant;
inactive scratch retention is not a new semantic State quantity.

## Effect release history

`ReleaseHistory` covers the admitted ordered-part maxima (Action one, Ambient one,
Contact two), independently of current playback arity. Decode every admitted entry at the
seed boundary. `soundN_channelK_releaseFlag` determines whether its `releaseTimer` is
meaningful; a nonactive entry becomes Inactive and discards stale timer payload. Contact
flags are at 1164 and 1166, timers at 1165 and 1167. These histories remain inputs and
outputs when no part owns a voice. Do not derive their length from the current cue.

Source ownership: `sound library.asm` lines 67 and 75-83 skip disabled parts and update
only the visited part's release state. `ResetSoundChannel` lines 19-39 does not reset
release history. `sound library 3.asm` lines 29-41 (replacement), 84-93 (last-voice
completion) and 126-139 (allocation initialization) likewise preserve it. A prepared cue
consumes the retained history in its ordered allocation; it does not create absent entries
for parts unused by the preceding cue. The complete history travels in live and pending
semantic state and restored continuations. Invented H-two-one-two-replacement,
H-one-finished-two and H-dormant-two-release-count prove replacement, completion and a
varied retained count independently from native content.

## Effect allocation and echo membership

`SetVoice` (`sound library.asm` line 10) clears the allocated physical voice from
`echoEnableFlags`. All three sound-library allocation loops invoke it. `ResetSoundChannel`
(lines 19-39) does not restore the bit. The explicit music echo selection writer
(`music.asm` line 547) sets membership directly, independently of effect ownership.

Decode `echoEnableFlags` (APURAM offset 74) as shared VoiceId membership. Compare the
completed semantic membership at group publication, including in pending/restored State;
allocation-time source scratch writes are not separately published semantics. Selection
preserves ownership; allocation clears only voices it acquires; release preserves the result.
No membership mask is added. J-one-allocation-release, J-two-subset-reserved and
J-nonmember-allocation use invented content and test the pending boundary, external
reservations, release and explicit reselection. The mapping does not establish an identical
DSP update instant: `engine.asm` lines 80-93 separately publish DSP cache fields.

## Harness projection and masks

Compare every physical publication's group, byte and exact elapsed AudioCycle instant. At each
named CPU cut compare publication count and last physical reply of every group, reconstructing
from captured publications. Driver APURAM reply-cache words are not an additional physical-port
projection. All named cuts refer to actual cut objects. Native SMP scheduler frontier and CPU
cut master time are separate quantities; the frontier can differ while the CPU cut is the same.

Compare committed Song fields from every cut object. During an explicit pending music
completion, mask only conditionally active source writers: note timers; positive release
countdowns or cue resets; active-ramp loudness/countdowns. The admitted pending expiring cue
has Loudness,Ramp,Duration,Sustain, so additionally mask its loudness/ramp target/increment
and duration stores. Verify that route from the preceding completed native prestate and
unchanged typed graph. Music175/196-197/510/517-520/785-795/913-914 own these writes;
engine184-197 returns for Sustain before the pitch store203. All other track and mix fields
remain compared during pending work. Across13 plans this masks77 fields and compares5995. Never mask
published reply, publication count, group or timestamp. The mask is an unfinished-source-write
projection; pending semantic work remains explicit saved State with restored continuation.

## Resident music reply ownership

Music29 initializes cpuIo0_write from the selected resident track. Music85?90 compares the
sampled CPU0 request with its previous value; equal retained selection does not load another
track. Music93 admits resident progress while the retained write is nonzero. Engine170
publishes cpuIo0_write before the stable pair; engine177 updates only the sampled read.
Arrangement restarts do not change that music reply latch.

Both paired loaded seeds independently have matching nonzero CPU0 incoming/read/previous/
write values, and neither arm has a CPU0 presentation. Each arm's191 music publications
preserves the loaded write byte. Fresh fixture initial_replies[0] independently decodes that
same byte. The resident reply identity is initialized from the loaded arrangement/seed, never
from expected publications or a captured constant. Effect absence retains its zero encoding.

The model boundary now explicitly accepts the resident reply identity and music outputs carry
it. This is an immutable input/output contract in this admitted scope, not a new music producer
or selection transition. Its Rust representation and placement remain the implementer's choice.
Two complete invented identities with distinct encodings are preserved through five music
publications and two arrangement restarts each. Generic selection-absence-to-zero encoding
fails at the first publication in both cases; correct boundary encoding restores all outputs.
The existing104 records retain their inputs/expectations; the new total is106. Existing
constructed cases use the invented default resident reply alias zero unless they specify one.
Native admission independently requires the loaded resident encoding, never that fixture default.

## Echo voice identities

EchoMix music547 copies the supplied eight-bit membership directly; it does not apply the
effect-reservation mask. The music track loop starts with physical voice0 and advances its
voice bit and two-byte track index together. The resident nonempty tracks occupy physical0/1;
effect allocation admits physical2..7. Echo identity and ownership use the same physical
mapping but different sets. Selecting echo membership changes neither reservations nor part
ownership. Subsequent effect processing retains its own source rules.

The resident initial echo membership is physical0/1/2/3, and a consumed root0 EchoMix names
all eight voices. This independently establishes admission of music-voice membership. The
complete invented M-echo-all-voices selects all0..7 while retaining external effect reservations
2/5. An actual effect-pool-only EchoMix fault drops0/1; the reference catches and restores it.
All prior103 constructed records retain their inputs/expectations; the total is104.

## Clock episode phase

The source's direct-read callback samples halfway through its final AudioCycle. The idle
music-start interval completes at anchor+874; its first resumed observation is anchor+873.5.
After music publication and stable-sample retries, the corresponding completion is
publication+retries+501 and observation phase is publication+retries+500.5. Each return
establishes a new eight-cycle episode; only empty observations within that episode preserve
its lattice. Timer prescaler phase survives observations and returns.

For P-idle, first nonzero observation1517.5 distributes at1544; music-start1573 gives
idle completion2447 and resumed phase2446.5. The next timer edge3565 is observed at3566.5,
matching7133/2. For P-effects-alone, publications1604/1690/1778 give music-start1835,
completion2709 and resumed phase2708.5; the same timer edge is observed at3572.5, matching7145/2.
The completion fixture field `next_observe_numerator` compares the first future nonempty
timer observation, not the next empty opportunity. After stops2447/2709, the respective next
empty opportunities are2454.5/2716.5. A reference may skip these zero-count reads because
they clear an already-zero count and preserve the prescaler phase; the compared nonempty
instants remain3566.5/3572.5. No internal next-event representation is required.
An actual global-initial-lattice fault instead observes3565.5 in both cases, and restoration
recovers their declared expectations. No vector or behavior expectation changes.

## Semantic ramp projection

An inactive ramp has no semantic configuration. The original retains its target and increment
bytes after its timer reaches zero, but the only admitted behavioral readers, music788 and
music792, are behind the active-timer guard at music785. A later ramp setter, music517/519/520,
overwrites target and increment before the timer can advance again. Root reload music141
clears the timer. Immediate loudness music510 preserves an active ramp and never reads an
inactive configuration.

The frozen paired reference's5995 comparisons include these retained inactive values as
supplementary observer-state evidence. Its predictions, comparisons and masks remain frozen.
Those extra observations do not make inactive configuration clean-model State. The clean
harness projects configuration only while a ramp is active; it still compares loudness, active
ramp progression, all admitted later behavior and every publication. This domain has no existing
engine floor changed by that projection. No new mask is added to the frozen paired experiment.
Invented M-inactive-config proves equal admitted continuations for two starting states that
differ only in inactive target and increment, including future ramp activation and root restarts.

## Admission and totality obligations

Valid repeated-phrase states retain a positive counter, valid phrase cursor and matching root
return position. Root positions include immediate root completion. Content admission separately scans each root
from event zero and requires its first Tone/Sustain/RepeatPhrase/CompletePhrase boundary to be
Tone or Sustain. This also applies to roots with no repeated phrase and rejects a timed event
that occurs only after earlier completion. The private importer applies this ordering check;
six invented cases exercise actual admission, including a presence-only validation fault caught
by two cases and restored. Both resident imported roots pass. All prior 97 vectors retain their
inputs and expectations; together with these six admission cases the reference passes 103. The independent structural
source audit covers5121valid semantic event-boundary/count states: only direct rootEnd or final repeatEnd
followed by rootEnd can end the arrangement; roots restart into timed sounds, at most once
per music turn. Exact source predispatch is923 or1025 respectively. Prior-slot full work
ownership is independently source-audited and exercised by constructed ending cases.

Admitted current timbres are resolved non-noise selections. No external edit may change the
fixed echo delay; the sole imported setting matches the seed, independently checked. Musical
writer guards and effect allocations preserve the low two music voices. Generalized Door
allocation, delayed starts, terminal motion and inactive holds are independently source-audited
and exercised by constructed cases. Ordinary allocation includes sixteen work units per
occupied visit for every group. The frozen model is unchanged; its separate successor carries
these completed laws.

The source audit closes signed ramp wrapping, fractional-target work, Timing adjacency and
extra handling, and all preview contributions. Preview starts with fifteen work units on release
expiry or21 when note duration is three with a still-positive release; repeated-phrase entry
adds59 and optional articulation adds14. These distinctions are verified with actual transition
faults and both frozen arms are re-proved after the corrections. No source page
walk or address-class interpreter belongs in the clean spec or imported musical event model.

## Budget

k = 37.5 weighted tokens per spec unit, frozen from samus-movement's old score
13,169.75 / 193.3 units times 0.55 (MODEL decision 7).

At `19242c525e03a1c981818dbba4498fb8ec0d307e`, this scope replaces no existing engine
implementation of sender queues, effect/music progress or physical reply transport. The old
source area is empty in each area below, so its size is zero; no surrogate source is counted.
The clean model adds the missing admitted behaviour.

- Area sender (old 0): OfferAcceptance, RequestDelivery, RequestRelease, SenderTiming
- Area effects (old 0): StagedRequests, EffectResources, ConnectedPitchMotion, EffectTiming
- Area music (old 0): MusicalProgress, FinitePhrases, WholeArrangement, LoudnessProgress, SongControls, MusicTiming
- Area transport (old 0): IndependentCadence, StableRequests, VisibleCompletion, ClockObservation, ReplyCadence

Budget: 37.5 × 72.3 units = 2,711 (sender 525, effects 862, music 787, transport 537)
