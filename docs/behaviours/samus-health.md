# Samus's current energy: behaviour spec

Her energy and brief protection after Ridley's tail hits her in Ceres, her energy
after periodic loss, and the warning requested while that energy is low. Vectors:
[samus-health.toml](../../scenarios/vectors/samus-health.toml).

## Header

**Scope.** This is a bounded part of H1, Energy, reserves and death. It covers periodic
loss while time runs and neither damage-reducing suit is equipped, consumption of that
pending loss, the low-energy warning, and Ridley's tail hit in Ceres and its
protection. Health owns one current energy and one remaining protection count. The
existing pickup inventory's whole energy is a neighboring pickup value. Production
must reconcile that projection with this current authority before claiming integrated pickup behavior.

**Reads.** Equipped suits come from [run-progress](run-progress.md); whether gameplay
time runs comes from its gameplay controller. Periodic losses come from their owning
hazard or action domains. This scope accepts already accumulated pending loss as
settable State. Ridley supplies the frame's tail impact: whether his tail made an
eligible hit this frame, judged against Health's protection as the frame starts.

**Leaves to others.** Capacities, pickups and equipment belong to Run Progress. HUD
[EnergyGauge](interface-hud.md#energygauge) shows whole current energy and owns its
last shown value. [Appearance](samus-appearance.md) reads current energy for animation
and retains its own previous drawing history. Audio decides whether a warning request
is heard. Appearance owns the flash after damage and Movement owns knockback. Ridley owns tail
position, overlap and whether he is tangible or holding Samus.

**Notation.** An energy amount is a whole count plus an integer fraction measured in
`1/65536` energy. NeitherSuit means neither Varia nor Gravity is switched on.
Names in input enums have no numerical ordering. Whole and fractional components are
kept because loss smaller than one energy can eventually change the displayed count.

## Memory and boundary

Health plays one step per frame. It takes its State as the frame starts and the
frame's tail impact, and gives back its State as the frame leaves it with the
frame's warning sound and protection while drawn.

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| current energy | `EnergyAmount`: whole `0..32767`, fraction `0..65535`, in energy | `PeriodicLoss`, `LowEnergyWarning`, `TailHit`; Appearance's `Breathing`, after the frame's tail hit and before its periodic loss; HUD's `EnergyGauge` and Appearance's drawing, as the frame leaves it; output |
| pending periodic loss | `EnergyLoss`: whole `0..32767`, fraction `0..65535`, in energy | `PeriodicLoss` |
| warning requested | `Warning`: `Off` or `On` | `LowEnergyWarning`; output |
| remaining protection | `ProtectionFrames` in active gameplay frames, `0..96` | `TailHit`, `ProtectionTime`; Ridley's tail contact, as the frame starts; output |
| the frame's tail impact (input) | `TailImpact`: absent, or Ridley's eligible Ceres hit worth 15 whole energy | `TailHit` |
| the frame's warning sound (output) | `WarningSound`: `None`, `Start` or `Stop` | audio |
| protection while drawn (output) | `ProtectionFrames`: remaining protection after the frame's hit, before the frame spends one | Appearance's drawing (the flash after damage), that frame |

Current energy, pending loss, warning and protection comprise complete State. A caller can
set any allowed combination, read it back, carry it, save it and restore it without
losing either fraction, pending loss, warning state or remaining protection. A
complete restoration resumes with the same next frame. Reading State changes nothing.
External pending-loss production and current-energy replacement must be
explicit owner updates; neither oracle next-state feedback nor a HUD history value may
silently supply them. No receipt, opportunity count or sound queue is additional State.

Production admits a frame using neighboring gameplay time and equipped suits. The
vector context names Running/Stopped and NeitherSuit/VariaSuit/GravitySuit.
Only Running with NeitherSuit admits a frame to this model. A caller requiring another
context must report Unsupported before mutation or warning effects. Wider contexts are
integration admission boundaries, not model input variants. The clean model receives
complete State, its rules and the frame's tail impact for an admitted frame. It is total
over every allowed State combination.

The warning sound is a request read by audio; audible playback is a separate
responsibility. Every admitted frame applies every rule below, including zero loss and
no tail impact. Unchanged scalar values alone cannot attest that the frame was played.

**Neighbor exchange.** Run Progress owns the capacities and pickup outcomes. A future
integrated pickup uses a fresh whole-energy projection of this owner as the pickup
model's input and publishes the resulting energy through an explicit owner exchange.
That exchange's fractional and pending-loss rules are not established here. Existing
Inventory and adapters do not yet implement it. Until proved, integrated pickups
requiring this exchange are Unsupported; there must not be two writable live energies.

## Rules

### PeriodicLoss

- kind: mechanic
- summary: accumulated periodic loss reduces energy, including fractional loss, once.
- params: none
- switch: fixed
- Running with NeitherSuit, pending loss no greater than current energy → current energy
  becomes the difference, pending loss becomes zero [C-zero, C-whole-loss,
  C-fraction-loss, C-borrow, C-fraction-equal, C-borrow-once, R-maintenance-current].
- Running with NeitherSuit, pending loss greater than current energy → current energy
  becomes zero whole and zero fraction, pending loss becomes zero [C-empty-clamp,
  C-zero-fraction-clamp, C-maximum-loss].
- checked: oracle (R-maintenance-current); constructed arithmetic contrasts
  (C-whole-loss, C-fraction-loss, C-borrow, C-fraction-equal, C-empty-clamp).

The difference is calculated exactly: `borrow = 1 if energy_fraction < pending_fraction
else 0`, `fraction = (energy_fraction - pending_fraction) mod 65536`,
`whole = energy_whole - pending_whole - borrow`. If `whole < 0`, both components are
zero. Consuming pending loss applies even when it was zero. The next frame
therefore cannot spend the same loss again. Restoring a snapshot taken before loss
repeats the loss; restoring one taken after it does not [C-restored-before-borrow,
C-restored-after-borrow].

### TailHit

- kind: mechanic
- summary: Ridley's eligible tail hit in Ceres takes energy and briefly protects Samus.
- params: none
- switch: fixed
- The frame's eligible tail impact with protection zero as the frame starts → subtract its
  whole energy from current whole energy with a floor of zero, keep the energy fraction and
  pending loss, and grant 96 frames of protection [R-tail-hit, C-tail-fraction-and-loss,
  C-tail-clamp, C-tail-after-expiry].
- No eligible tail impact, or protection still positive as the frame starts → keep current
  energy, pending loss and protection [R-tail-protected, C-tail-repeat, C-tail-last-frame,
  C-tail-no-impact].
- checked: oracle (R-tail-hit, R-tail-protected); constructed overlap during protection
  (C-tail-repeat, C-tail-last-frame).

### ProtectionTime

- kind: timing
- summary: protection spends one frame in each admitted frame, after the frame's hit.
- params: none
- switch: fixed
- An admitted frame → protection while drawn is the remaining protection after the frame's
  hit, and the frame leaves one frame less, with a floor of zero [R-tail-hit,
  R-tail-protected, C-tail-last-frame, C-tail-no-impact].
- checked: oracle (R-tail-hit, R-tail-protected); constructed expiry
  (C-tail-last-frame, C-tail-after-expiry).

### LowEnergyWarning

- kind: output
- summary: entering low energy requests a warning, leaving it requests silence.
- params: `warning_threshold`, whole energy count, reference `30`, allowed `0..32767`.
- switch: parameter
- Whole energy after the frame's hit and loss at most the threshold, with warning Off →
  warning becomes On and requests Start [C-borrow, C-threshold-low, C-adjusted-threshold, R-tail-hit].
- Whole energy after the frame's hit and loss greater than the threshold, with warning On →
  warning becomes Off and requests Stop [C-threshold-high].
- Warning already matches that comparison → warning stays as it is and requests None
  [R-maintenance-current, R-tail-protected, C-zero, C-warning-repeat, C-borrow-once].
- checked: oracle (R-maintenance-current); constructed threshold contrasts
  (C-threshold-low, C-threshold-high, C-warning-repeat, C-adjusted-threshold).

The warning uses whole energy alone: a fraction just below the next whole count does
not move its threshold. Warning state records the request even if audio rejects it.

## Order that matters

Within a frame the tail hit comes first, then periodic loss, then the warning, and
protection is spent last. So the warning judges the energy both the hit and the loss
left [R-tail-hit, C-tail-fraction-and-loss], and a hit leaves 95 frames of protection
at the frame's end and 96 while drawn [R-tail-hit]. The hit is judged on protection as
the frame starts: a frame that starts with one frame left blocks the hit and leaves
none, and a hit on the next frame lands [C-tail-last-frame, C-tail-after-expiry].

Ridley's contact reads Health's State as the frame starts. Appearance's animation
reads current energy after the frame's tail hit and before its periodic loss, so on
the hit frame of R-tail-hit it sees 24, not 39. Health's outputs do not carry that
value yet (see the gap list).
Appearance's drawing and the HUD gauge read the energy the frame leaves; drawing also
reads protection while drawn. Health does not own Appearance's previous drawing energy,
the flash after damage or the gauge's last shown value.

## Missing capabilities

The [gap list](samus-health.gaps.md) distinguishes the bounded model from the rest of
H1. Whole energy above 32767 and negative or out-of-range pending loss are outside
admitted input ranges. The model is total for every admitted State and frame.
An unsupported integration admission leaves complete State untouched before the model
is called. Those admission contrasts earn no model-vector or null-gain credit.
No source crash is reproduced.
