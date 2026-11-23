# Audio transport integration gaps

The model is specified one frame at a time and compared at each NMI cut. Production integration
waits for the inputs below.

| Model input | Engine owner or missing prerequisite |
| --- | --- |
| Cue offers, allowance and suppression | The indexed sender owns FIFO admission. Bounded live callers deliver soft touchdown, the ordinary power beam and Ceres opening, and the first normal passage's silence pair. Footstep reaches the sender only in the scoped sender-offer cases, which offer their cues directly; production drops the appearance model's Footstep opportunity. Spin start/end, weapon impact, other actor cues and remaining timed caller joins still need producers. |
| Whether a frame runs a sender service | Borrowed per frame, like a lag cut, until T1 owns the CPU's frame timing. Sender downtime and queue reset are not modeled. |
| Audio cadence per frame (effect turns before and after the service, music turns) | Borrowed per frame until T1 models the audio processor's clock. A record supplies it only when its APU port log carries the SPC's writes; the lean cache rows carry only the CPU side. |
| Typed resident music and effects | A3 must import checked Forever roots, phrases and saved event positions, resolved dense non-noise timbres, sample-neutral instruments and fixed echo delay. No production importer supplies these assets. |
| Loaded transport, song and effect state | The session loader must supply the admitted initial State once, including every ordered release-history entry (Action one, Ambient one, Contact two), even dormant parts. A future bridge must retain canonical::model::Model with a shape pin. |
| Controlled repeat expiry | The expiry arm is compared only through cut 64, before the external repeat-count intervention: its post-intervention seed carries no CPU sender State, which the frame form needs. FinitePhrases' return arm (a repeat count's expiry) has no indexed witness until that seed is re-cut from the capture's CPU memory; its repeat arm is compared by the control case at cuts 57 and 149. Production has no such externally owned edit. |

The model emits, per frame, each line's reply and its sender, staged, handled and song State.
U1 must encode original cue bytes at the port boundary and retain replies; a later mixer
consumes musical/effect results separately. No production consumer currently holds these
outputs. No existing audio implementation is replaced.

Fresh construction validates the two-track state/content relation, saved phrase positions,
complete ordered effect histories, pitch domains and resolved timbres. A restored State resumes
with the same validated resident content identity; it is not reinitialized or supplied with
replacement content. The harness retains one immutable Content for every saved continuation.

Effect timbres name admitted instruments. Retained effect-instrument playback output is outside
this transport boundary; a future audio backend needs its own modeled owner. Music instrument
retention is compared separately.

Native evidence admits only the two frozen opening arms. Broader cached rows are unsupported
until their independent content, cadence and loaded transport states are decoded. Decode or
admission failures are defects, not successful skips. Constructed vectors establish no native
row coverage.

The native cases preserve the independently fixed CPU cuts. At every cut they compare the group
replies (except those of an effect turn under way at the cut), the music reply, the three
request lines and the committed song and release fields. The sender's State is not compared:
the captures record no CPU memory. The 77 source-write
exclusions belong to the 13 music turns under way at a cut, listed in
[decode notes](audio-transport.decode.md#harness-note); inactive ramp configuration is
supplementary observer state, rather than admitted retained model State.

Raw per-cut release-history probes are diagnostic evidence. Original effect scratch can
already contain an unfinished turn's writes at a fixed CPU cut, so raw equality there is not the
committed-history gate. The generator retains these probes separately and decodes every
ordered history at the seed. Future native turns and every restored continuation additionally
exercise dormant history when later playback consumes it.
