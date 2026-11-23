# Audio transport decode notes

The behaviour spec is audio-transport.md; this document is excluded
from the implementer export. Sources below are observation/source arithmetic, not model State.

## Evidence and scope

The spec states the domain one frame at a time (MODEL §1 decision 35): the rungs compare the
sender's State, the reply lines and the song at each NMI cut. Intra-frame instants (audio
instruction cycles, CPU master ticks, the driver's timer polling and its paired port reads)
belong to T1 and are not State.

The native opening prefix supplies 184 named CPU cuts, 140 CPU presentations and 764 SPC
publications, in a reference control arm and an independently authorized repeat-expiry arm
that share the CPU stimuli. Each owns a loaded seed; no native prestate is injected each turn.
Committed song fields are compared at every cut; 13 music turns under way at a cut mask 77
individually attributed source writers (see the harness note).

## The reply frame (FrameCadence)

**Question.** Without the cycle tables, which frame does a reply first reach the sender on?

**Rule chosen.** No frame count reproduces it, so the frame's audio cadence is a declared
borrowed input, decision 35's pattern for a lag cut: per frame, the effect turns before the
sender service, after it, and the music turns. The reply frame then follows from the turn rule
the spec states (a turn handles the request staged by the previous turn, publishes it, then
stages the request line), with no other timing.

**Evidence.**
- *A fixed frame count fails.* Over the 63 cached rows (lean CI cache, 5663de2c4b548c68), the
  sender acknowledges 4,852 presentations (a cue or a clear). "First service at least two
  frames after the presentation" holds for 4,475 (92.2%). In 312 (6.4%) the reply arrived
  earlier, in 65 (1.3%) later. Lag frames are accounted (the first service at or after the
  frame). One frame holds for 308 and three frames for 147. The misses are not rare cases of one kind: turns
  come every 16,384 audio cycles against about 16,950 per frame, so their place in the frame
  drifts across the service's, and the service's place moves with the frame's CPU load.
- *The turn rule is exact.* In both native arms, every one of 34 resolved presentations is
  first answered at the second effect turn after it, with no exception.
- *Borrowed cadence, cut by cut.* Deriving each frame's cadence from the capture's port order
  (turns before or after the frame's first presentation) and running the frame form on the
  native control and expire arms reproduces all three group replies at 184/184 cuts in each
  arm (552/552 replies). The two-frame cadence (every turn after the service) reproduces
  164/184 cuts (531/552); holding the previous cut's replies reproduces 151/184 (518/552).
- *Recorded cases.* F-recorded-two-turns is the control arm's Action group, frames 16-18: frame
  17 runs one turn before its service and one after, and the reply is read on frame 18.
  F-recorded-late is frames 29-32: the turn of frame 29 comes before the presenting service,
  frame 30's before its service and frame 31's after, so the reply misses frame 31's read by 142
  audio cycles and is read on frame 32. Frame 32 presents nothing, so where its turn falls is
  not observed; either order gives the same outcome. The cached opening row
  `chain-any-0003-DFD7` shows the same sender progress at those cuts (its queue is empty at cuts
  15 and 28, so each cue is offered and sent in the same frame).

**Where the cadence comes from.** A record whose APU port log carries the SPC's writes gives it
directly: a frame's effect turns are its writes of the three group lines, in order with the
CPU's sender reads and writes. The native captures carry these; the 63 lean rows log only the
CPU side of the ports (no SPC writes), so for them only the reply each service read is
available (the CPU read values), not the cadence.

**Sender rules over the rows.** Per frame, applying the frame's offers (the queue entries the
row shows appended), then the service when the frame ran one (the row's end-of-frame wait
marker), reading as each group's reply the value its CPU read that frame, the sender's
progress, delay, cue and queue agree at 122,239 of 122,446 group cuts (99.83%); holding the
previous cut agrees at 110,749 (90.45%). Of the 207 misses, 53 are in frames with sender
downtime (below) and the rest are frames where the wait marker misplaces the service: a
service whose wait falls after the cut, or a wait-for-NMI from a transition loop with no service.

**Planted faults** (reference model): an acknowledgement delay of one instead of two fails 7
vectors (S-ack and every frame vector that reaches an acknowledgement); publishing before
handling fails 11; staging before handling fails 7; running every turn after the service (the
two-frame rule) fails F-early, F-recorded-two-turns and F-recorded-late; allocation that keeps
echo membership fails 2; a restart that keeps a ramp fails M-restart.

## Independent expiry continuation seed

The encrypted native bundle holds `audio-expiry-post64-seed-v1`, decoded from the expiry
capture's complete `music-after` snapshot at the intervention barrier, after the paused
repeat-count adjustment. The capture dumps `cut-64` before the adjustment, increments its capture
number to 65, then dumps `music-before` and `music-after`. Every `music-before` device object
equals its cut64 object. The post-adjustment CPU, PPU and DSP objects are unchanged; APURAM
changes only offsets 128/130, and SMP_STATE changes only their embedded equivalents 176/178.
The ordinary cut64 observation remains the pre-adjustment expectation.

The seed holds the audio side only: song positions, active effect parts, every release-history
entry and the driver's clock position. It holds none of the CPU senders' queues or progress,
which the frame-form State carries, and at cut 64 the Contact sender is mid-delivery. So no
indexed case starts from it: the expiry arm is compared through cut 64 only, and its cuts 65..183
wait for a seed that also decodes the senders from the capture's CPU memory.

## Sources by behaviour

| Entry | Source ownership and evidence |
| --- | --- |
| OfferAcceptance | CPU queue offers, producer allowances and suppression; native enqueue proof |
| RequestDelivery, RequestRelease | bank82 sender entries 8A2C/8A55/8A6C/8A7C/8A90, called once per main-loop frame from 8289EF; native sender and retained-state proof; the 63-row sender check above |
| FrameCadence | the driver's timer-paced main loop (effect and music turns every 16,384 audio cycles in this song); native publication order against CPU presentations |
| PublishedReplies | the driver's per-group reply writes after handling, and the resident music reply byte; native publications |
| StagedRequests | the driver's request sampling after its reply writes and its handling at the next turn; the turn rule above |
| MusicalProgress, FinitePhrases | typed imported two-root/five-phrase graph; music graph native field proof and source note/control arithmetic |
| WholeArrangement | first-root completion, outer arrangement and all-track reload; independent outer arrangement audit |
| LoudnessProgress | song's final volume pass; native scalar/ramp proof and constructed whole-song fault cases |
| EffectResources, ConnectedPitchMotion | sound library allocation and connected Door service; independent Door aggregate source audit |
| SongControls | music control events; native scalar and echo proof |

Source work receipts are detailed in private observer reports. The admission is Forever,
not a general arrangement interpreter. All eight native track
slots are checked; two nonempty roots are imported and six slots are empty. Every event and
timbre reference closes inside the admitted graph/content. No native pointer leaves import.

**Sender downtime** (WRAM 0686 above zero) makes a main-loop frame skip the sender states and
write zero to the three request ports and the three current cues. The opening never sets it;
item collection and boss rows do (3,428 group cuts in the 63 rows). It is outside this round:
such a frame is one without a service here, and the zeroed request lines and current cues are
not modeled. **Queue reset** (8A9A) empties the queues and states outside a service; also not
modeled (4 group cuts in the rows).

## Mapping

APURAM offsets below are observations only. A track row has stride 2 even for one-byte fields.
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

CPU sender queue observations use WRAM base 0643. Per group, read index is at base+group,
write index at base+3+group, sender progress at base+6+group (0 Ready, 1 AwaitCue, 2 ClearDelay,
3 AwaitClear, 4 ResetDelay), current cue at base+10+group and delay at base+13+group; sixteen
queue slots start at base+19+16*group. Queue length is modular write minus read with capacity
15. Convert to oldest-first semantic FIFO. A cue word meaningful only in AwaitCue and a delay
meaningful only in its delay variant are carried by that variant; inactive scratch retention is
not a new semantic State quantity. A frame's offers are the slots between the previous cut's
write index and this cut's. The request line of group g is APU port g+1 as the CPU last wrote
it; the reply line is the same port as the SPC last wrote it, and the CPU's read of it in the
frame's service is the reply the sender reads.

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
for parts unused by the preceding cue. The complete history is State at every cut. Invented
H-two-one-two-replacement, H-one-finished-two and H-dormant-two-release-count prove
replacement, completion and a varied retained count independently from native content.

## Effect allocation and echo membership

`SetVoice` (`sound library.asm` line 10) clears the allocated physical voice from
`echoEnableFlags`. All three sound-library allocation loops invoke it. `ResetSoundChannel`
(lines 19-39) does not restore the bit. The explicit music echo selection writer
(`music.asm` line 547) sets membership directly, independently of effect ownership.

Decode `echoEnableFlags` (APURAM offset 74) as shared VoiceId membership and compare it at each
cut. Selection preserves ownership; allocation clears only voices it acquires; release
preserves the result. No membership mask is added. J-one-allocation-release,
J-two-subset-reserved and J-nonmember-allocation use invented content and test external
reservations, release and explicit reselection. The mapping does not establish a DSP update
instant: `engine.asm` lines 80-93 separately publish DSP cache fields.

## Harness note

**Compared at each cut:** per group, the sender's progress, delay, cue and queue (WRAM, above);
the three group replies and the music reply (the last SPC write of each port before the cut);
the committed song fields from every cut object; effect release history and echo membership.
A port reply that changes gameplay is compared on the frame it is read: the reply each
service reads is the CPU's read of its port in that frame.

**Borrowed per frame:** whether the frame runs a sender service, and its audio cadence (effect
turns before and after the service, music turns), until T1 models the CPU's frame timing and
the audio processor's clock. A harness derives the cadence from the record's port log; a record
without SPC port writes cannot supply it.

**Skipped:** publication instants, CPU master ticks, the driver's timer phase and polling
lattice, the paired port reads and their retry spacing. These are T1's.

**Masks.** A music turn can be under way at a cut: its turn has started but not completed, so
the cut's APURAM holds some of its writes. The borrowed music count counts only completed turns,
and the fields that turn's source writers have reached are masked at that cut: note timers;
positive release countdowns or cue resets; active-ramp loudness/countdowns. The admitted
expiring cue has Loudness,Ramp,Duration,Sustain, so its loudness/ramp target/increment and
duration stores are also masked. Music 175/196-197/510/517-520/785-795/913-914 own these writes;
engine 184-197 returns for Sustain before the pitch store 203. All other track and mix fields
remain compared. Across 13 such cuts this masks 77 fields and compares 5,995. Never mask a reply
or the sender's state.

**Indexed replay.** The native cases play production's audio owner one frame at a time. Each
frame's offers are the cues the CPU newly presents in it, and its cadence comes from the port
log: an effect turn counts in the frame of its Contact reply, the turn's last write, and splits
at the frame's first CPU write; a frame with no CPU write runs its service after its turns. The
model completes a turn whole, so where a cut falls inside an effect turn (after its Action and
Ambient replies, before Contact's) those two replies are not compared at that cut: 12 in the
control arm (6 cuts) and 2 in the expiry prefix (cut 17). Every other reply, every request line
and the song fields above are compared.

## Resident music reply ownership

Music29 initializes cpuIo0_write from the selected resident track. Music85-90 compares the
sampled CPU0 request with its previous value; equal retained selection does not load another
track. Music93 admits resident progress while the retained write is nonzero. Engine170
publishes cpuIo0_write before the stable pair; engine177 updates only the sampled read.
Arrangement restarts do not change that music reply latch.

Both paired loaded seeds independently have matching nonzero CPU0 incoming/read/previous/
write values, and neither arm has a CPU0 presentation. Each arm's 191 music publications
preserves the loaded write byte. Fresh fixture initial_replies[0] independently decodes that
same byte. The resident reply identity is initialized from the loaded arrangement/seed, never
from expected publications or a captured constant. Effect absence retains its zero encoding.

The model boundary accepts the resident reply identity and music outputs carry it. This is an
immutable input/output contract in this admitted scope, not a new music producer or selection
transition. Its Rust representation and placement remain the implementer's choice. Two
complete invented identities with distinct encodings are preserved through five music turns
and two arrangement restarts each. Generic selection-absence-to-zero encoding fails at the
first music turn in both cases. Existing constructed cases use the invented default resident
reply alias zero unless they specify one. Native admission independently requires the loaded
resident encoding, never that fixture default.

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

## Semantic ramp projection

An inactive ramp has no semantic configuration. The original retains its target and increment
bytes after its timer reaches zero, but the only admitted behavioral readers, music788 and
music792, are behind the active-timer guard at music785. A later ramp setter, music517/519/520,
overwrites target and increment before the timer can advance again. Root reload music141
clears the timer. Immediate loudness music510 preserves an active ramp and never reads an
inactive configuration.

The paired reference's 5,995 comparisons include these retained inactive values as
supplementary observer-state evidence. Those extra observations do not make inactive
configuration clean-model State. The clean harness projects configuration only while a ramp is
active; it still compares loudness, active ramp progression, all admitted later behavior and
every reply. Invented M-inactive-config proves equal admitted continuations over 320 music
turns for two starting states that differ only in inactive target and increment, including
future ramp activation and root restarts.

## Admission and totality obligations

Valid repeated-phrase states retain a positive counter, valid phrase cursor and matching root
return position. Root positions include immediate root completion. Content admission separately scans each root
from event zero and requires its first Tone/Sustain/RepeatPhrase/CompletePhrase boundary to be
Tone or Sustain. This also applies to roots with no repeated phrase and rejects a timed event
that occurs only after earlier completion. The private importer applies this ordering check;
six invented cases exercise actual admission, including a presence-only validation fault caught
by two cases and restored. Both resident imported roots pass. The independent structural
source audit covers 5,121 valid semantic event-boundary/count states: only direct rootEnd or
final repeatEnd followed by rootEnd can end the arrangement; roots restart into timed sounds,
at most once per music turn.

Admitted current timbres are resolved non-noise selections. No external edit may change the
fixed echo delay; the sole imported setting matches the seed, independently checked. Musical
writer guards and effect allocations preserve the low two music voices. Generalized Door
allocation, delayed starts, terminal motion and inactive holds are independently source-audited
and exercised by constructed cases. The source audit closes signed ramp wrapping and Timing
adjacency with actual transition faults. No source page walk or address-class interpreter
belongs in the clean spec or imported musical event model. The importer's per-Timing extra
handling count is T1 work and is not part of the frame model's content.

## Reference model

The observer's reference model implements the frame-form spec and runs every vector, 77/77;
its per-frame checks over the native arms and the cached rows give the agreement above,
against a do-nothing step.

## Budget

k = 37.5 weighted tokens per spec unit, frozen from samus-movement's old score
13,169.75 / 193.3 units times 0.55 (MODEL decision 7).

At `19242c525e03a1c981818dbba4498fb8ec0d307e`, this scope replaces no existing engine
implementation of sender queues, effect/music progress or physical reply transport. The old
source area is empty in each area below, so its size is zero; no surrogate source is counted.
The clean model adds the missing admitted behaviour.

- Area sender (old 0): OfferAcceptance, RequestDelivery, RequestRelease
- Area effects (old 0): StagedRequests, EffectResources, ConnectedPitchMotion
- Area music (old 0): MusicalProgress, FinitePhrases, WholeArrangement, LoudnessProgress, SongControls
- Area transport (old 0): FrameCadence, PublishedReplies

Budget: 37.5 × 52.3 units = 1,962 (sender 450, effects 725, music 637, transport 150)
