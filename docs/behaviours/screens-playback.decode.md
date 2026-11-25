# The intro's demo controls and their oracle records

Companion to the [behaviour spec](screens-playback.md) and its
[vectors](../../scenarios/vectors/screens-playback.toml). This file is observer-side.

## Sources and observations

The relevant bank logs are [bank 8B](https://patrickjohnston.org/bank/8B),
[bank 90](https://patrickjohnston.org/bank/90) and [bank 91](https://patrickjohnston.org/bank/91).

| Entry | Sources |
| --- | --- |
| DemoReceivesScene | The battle setup at `$8B:AECF..AF68` and discovery setup at `$8B:AF7B..B015` start Samus standing left, copy whole centre to previous whole centre, and set cannon item selection. They clear the old controller playback at `$91:8370`, enable it at `$91:834E` and select the scene through `$91:8395`. The stage in `$8B:A35B..A38B` advances before the actors. Its `$8E0D` tests the nonzero actor/display gate at `1A57`, runs the two Samus updates, then ticks hit timers. `$8E2D` tests the gate again after the scene sprites, for drawing. |
| DemoPlaysPad | `$90:E6C9..E712` saves movie-pad held, fresh, previous held and previous fresh at `0A14/0A16/0A90/0A92`, then uses `$E91D` to read the demo before pose choice. `$91:83C0..8426` hands the player code the current and previous demo samples. `$90:E833..E869` uses them until it restores the backed-up movie pad. `$8B:BC20..BC74` saves and loans the standard bindings. `$8B:B769/$BC75` restores the bindings at the intro's end. |
| SamusReachesEgg | `$91:864F` checks the pre-motion `0AF6` whole X against 178, then transfers to the watch phase on that turn. The signed comparison tests the sign of the 16-bit subtraction. All valid room coordinates here are within the non-wrapping range of that test. `$91:866A` first checks the newly selected phase on the next opportunity. |
| BattleLocksOnLeft | `$91:8739..8773` selects `$90:E713` for later current-state work and initializes pose 2 through `$91:F433/$FB08`. `0A20/0A22` move into `0A24/0A26`, and `0A1C/0A1E` move into `0A20/0A22`. `$91:835F` disables demo admission and the battle then selects the empty pose-input handler `$90:E90E`. `$91:8427` clears the pad and sequence. Because `$90:E91D` is already executing, it still calls the ordinary pose chooser after the last neutral sample. `$90:E833` still runs the intro's move, animate, hit, pose and palette update on this and later visible turns. The locked current update at `$90:E713` advances existing projectiles and resets transient movement and pose variables. |
| DiscoveryReleasesSamus | `$91:866A` checks the post-stage count at `1A4B` before another watch pad sample can be issued. At zero, `$91:8682..8693/$835F/$8427` disables playback and restores the ordinary pose selector `$90:E913`. Both Samus updates are set to `$90:E8CD`. The current update already in flight completes, with neutral current pad and the previous demo sample. The following new-state dispatch is already disabled. The dancing baby at `$8B:BB24..BB38` clears the actor/display gate later in the turn on the same zero. |

### Content conversion

The battle source begins at `$91:8694`, continues through `$86B8..86FA`, and finishes with
the standing-left handoff. The discovery source is the opening interval at `$91:860D..8618` and
the watch interval at `$91:8623..8640`. Each discovery interval has a repeated Left tail. The
importer converts the durations and the controls into the three named timelines. A first-turn
fresh sample and its following copies without fresh buttons may combine into one typed hold.
Keep independent fresh presses inside a longer held button period. Both discovery tails have
the same clean meaning, a continuing Left hold. Represent the threshold, fade and handoffs in
the model. The clean model consumes typed pad content and has no source addresses or executable
event data. The reference reads the localized input spans from the player's ROM and normalizes the
two discovery tails as permanent Left.

### State and cut mapping

- `0A88` admits the demo, `0A7A` identifies the discovery phase, and the combination of
  `0A7E` and countdown `0A7C` yields progress in the associated pad timeline. With a loaded
  hold, completed opportunities within that hold equal its authored duration minus the native
  countdown plus one. The pending setup has progress zero. The two repeated tails normalize to
  progress at the end of their finite timeline.
- `0A84/0A86` record the demo's issued held and fresh sets in standard intro bindings.
  `0A8C/0A8E` retain that same sample for the next opportunity. At entry to pose and weapons
  their previous held and fresh values come from the preceding retained sample. In-flight
  substitutions are observable at `008B/008F/0DFE/0E00` before they are restored or polled.
- The battle finish is disabled playback with `0A42/0A44/0A60 = E713/E833/E90E`. Discovery's
  finish is disabled playback with `E8CD/E8CD/E913`. The engine must carry the two
  outcomes as distinct states so they survive the remainder of each scene. Both endings
  clear the native hold and pad fields. The final publication has neutral current pad
  and the previous retained pair, because entry to the pad step was already admitted.
- Compare handoffs through their per-cut enabled state, Samus modes and the relevant pose or
  lack of a pose change. Compare no changes to motion, weapons or draw as demo-owned values.
  The State of those domains and their bytes remain with their owners. Skip the dormant
  native addresses, the unused input timer and initialization argument at `0A80/0A82`.
- `$80:9459` at `$80:95E1` polls the controller again in the NMI that publishes the frame.
  Completed-cut controller RAM is the machine's live stream, not an oracle for the temporary
  in-room substitution. The movie stream is a platform producer. Use the demo sample and output
  handoff as the scoped comparison, and let platform replay own the published pad bytes.
  The reference observed four cuts following scene setup where the live words differed from the
  in-room backup, with the demo's own retained sample unchanged.

## Replay

Start from the press at `any-intro-0568` cut 1,650 for the upstream page transition. The observer
comparison carries playback State across all of `any-intro-2219` and through cut 129 of
`any-intro-4323`. In production, the intro supplies scene start and the room actor gate.
The replay borrows these, pre-motion X from Samus, the post-stage crossfade count, and the platform's
work phase until those producers join. It carries the demo's State and uses the same pad source
in scoped and integrated play.

| Row and local cut | Event |
| --- | --- |
| `any-intro-2219` 0, 1 | Start is admitted before the first playable turn. The first turn issues a neutral pad. |
| 91, 135, 335, 376 | Fresh Fire, with the earlier hold still active at 135 and 376. These are also the ammo debit cuts in the weapons witness. |
| 101–102, 106–107 | Pad step is already visible on the unfinished cut. The next cut finishes the same work turn. |
| 131–132 and 520–521 | The unfinished cut precedes the pad opportunity. The following cut reaches the actors. The battle is already locked at 520. |
| 255, 263 | Start moving left, then press Jump while continuing to hold Left. |
| 395–397 | Last controlled Fire hold, neutral with the standing-left handoff, then locked current update. |
| 648–649 | The battle actors have their last turn before the encounter hides them at 648. The next turn is outside their window. |
| 1,298–1,299 | The page 2 press has installed discovery and reset the pad memory. The first opportunity follows. |
| 1,427–1,428 | Discovery's unfinished cut precedes the next actor turn. |
| 1,467–1,469 | Whole X is 180 at 1,467. The step to 177 keeps Left. That new whole X changes the pad to neutral on 1,469. |
| 1,769, 1,940 | Aim Up starts and later releases. |
| `any-intro-4323` 0 | The watch continues from the previous row under the return dissolve. Cut 0 is before the actor step. |
| 77–78 | Left begins again with a fresh press, then continues held. |
| 127–129 | Count 1 and walking, count 0 and discovery release, then the final colour step with the actor window closed. |

Use `05B6` for completion. The reference uses changes in `1A51` and the old actor gate to
classify the pad opportunities on these particular records, where the increment and pad step
are on the same side of each interesting cut. In general the cinematic count increment
precedes the pad opportunity, so production must carry its actual step phase. In the
two start cuts, room setup is visible before the first actor turn. In the two ending scenes,
the hide is later than the pad opportunity. The discovery stop must read the count the
fade stage just changed on that turn.

The current production intro reports `Intro(BattleBegins)` at the opening boundary. Closure
needs scene staging, room collision, Samus, weapons and cinematic actor joins. This pad source
feeds the real movement and weapons paths. Reference comparison does not certify their outputs.

## Reference evidence

The local observer scripts in `$TEMP/collab/demo-input-obs-notes/` are `reference.py` and
`vector_check.py`. They use Python 3.11 or newer and the lean cache reader in
`$TEMP/collab/fight-obs-notes/lean.py`.

- The 19 recorded and 10 constructed vectors agree.
- State matches all 2,234 cuts across the two rows, including eight unfinished cuts, versus zero
  for remaining Off. Current and previous pad agree on all 1,323 ordinary active turns.
  Nine of these publish fresh presses. Both final samples are neutral with the matching
  handoff. Both terminal update modes match. At the battle's ending the observed pose is
  standing left before and after the handoff. Discovery also retains its pose. These are
  terminal checks. A handoff that changes Samus's pose or history needs a join witness.
- Inferring presses from differences between held samples first fails on the second
  Fire press at `any-intro-2219` cut 135. Advancing only on completed cuts first fails on 101.
  Advancing again when that frame completes first fails on 102. Changing the discovery
  stopping line from 178 to 177 first fails on 1,469. Reading the stale fade count first
  fails discovery's release at `any-intro-4323` cut 128.

## Budget

Fixed base `b7dc44c8e5a3fe66434eaddcb0688be5e23a4db6`. The portable intro has no demo pad
engine to replace, so there is no attributable old code area.

k = 37.5 weighted tokens per spec unit.

- Area controls: DemoReceivesScene, DemoPlaysPad, SamusReachesEgg, BattleLocksOnLeft, DiscoveryReleasesSamus

The counter reports five entries and nine arms, 9.0 units and a budget of 337 weighted tokens.
