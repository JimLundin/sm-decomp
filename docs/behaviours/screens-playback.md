# Recorded controls in the intro

Samus fires, runs, jumps and fires again in the flashback to Zebes. She approaches the baby from the right,
stops to look up, and runs again while the picture fades back to text. The two scenes
supply their own pad to Samus and her weapons. The player's pad advances the intro's pages.

## Header

**Scope.** F2's two played scenes, through each scene's handoff back to the intro. This is
the first portable model for their recorded controls.

**Reads.** The intro supplies the start of each scene, its current crossfade count, and whether the
room actors step. The platform supplies opportunities to advance the room actors across display
cuts. Samus supplies her whole X at the start of her turn, measured in pixels from the left edge of
the discovery room. Content supplies the pad holds in our format.

**Hands on.** During a played turn, Samus and her weapons receive the recorded pad for this
turn and for the preceding turn. A finish also gives them the scene's handoff. The intro continues to read the movie pad.

**Leaves.** The intro owns opening and closing the room actor window, the cutscenes and dissolves,
and the lending and return of equipment and standard bindings. Samus owns position, motion, bodily
pose, animation and pose history. Weapons own ammo, muzzle, flight and impact. The encounter owns
the fight's actors and hits. The pad model asks for a pose on the battle finish and for the change
in Samus's updates at each finish.

## Pad data and memory

A **pad sample** has two button sets, held and fresh press. Here the buttons are Left, Jump,
Fire, and Aim Up. A fresh press belongs to the held set. A **hold** supplies a nonempty time
span in room actor turns with the same held set, plus the fresh press set for its first turn.
The holds in each of three timelines describe the battle, the approach to the baby, and the
watch by the baby. Each timeline is nonempty. The last hold in each discovery timeline holds
only Left. The discovery can keep walking after reaching the end of either of those timelines. The
battle instead comes to a standing finish. The importer validates these data and supplies named
button sets and positive durations. With standard intro bindings, Jump is A, Fire is X, and
Aim Up is R.

The caller can read, save and restore phase, progress and last pad together with the selected scene's content.

- **Scene phase** is a variant: Off, Battle, Approach, Watch, Battle Locked, or Released.
  Read by all entries.
- **Timeline progress** is the number of room actor turns already issued in the active phase,
  bounded by the length of that phase's supplied timeline, with zero before its first turn.
  It is absent outside Battle, Approach and Watch. Read by `DemoPlaysPad` and `BattleLocksOnLeft`.
- **Last pad sample** is the most recent held and fresh sets issued by the active phase, carried
  across Approach to Watch and reset to neutral on start and finish. Read by `DemoPlaysPad`,
  `BattleLocksOnLeft` and `DiscoveryReleasesSamus`. The prior sample is also output.
- **Whole X** is a pixel coordinate in the discovery room. Read by `SamusReachesEgg`.
- **Crossfade count** is the intro's remaining count in turns after its stage has advanced.
  Read by `DiscoveryReleasesSamus`.
- **Handoff** is a one-turn output to Samus and weapons: Start Battle, Start Discovery,
  Stand Left, or Release. The current sample is also output whenever a demo turn produces one.

## Order that matters

The intro may start a scene before its first room actor turn. Start is observable even if a cut
falls before that turn. It enables the ordinary controlled pose and weapon work and the intro's
movement and appearance work. Battle's scene entry places Samus standing left at (155, 115) px
and selects missiles. Discovery's entry places her standing left at (376, 147) px and leaves the
item selection empty. Both initialize the pose with coherent previous whole positions. These are
entry effects for the room, Samus and weapons owners to apply.

On a room actor turn, take the crossfade count from the updated intro stage and X from before
this turn's motion. Apply the pad and any handoff before new motion. Room actors and the display
can span cuts. A cut before the pad opportunity holds its progress. A cut after it carries the
step forward, including to the end of that same turn. The intro's later hide event controls
drawing immediately and closes future actor turns.

On the battle's finish turn, Samus has already begun her usual controlled update. Apply
Stand Left before she makes her ordinary pose choice on the neutral sample. Complete that turn's
weapons work and the intro's movement and appearance. On later visible turns, stop accepting pose
choices and new weapon commands. Existing projectiles keep moving and the intro continues
movement, appearance and hit handling. The hit timers keep counting down.

On the discovery's finish turn, complete the controlled pose decision and weapons work already
underway with the neutral sample. Skip the subsequent movement and appearance update that
turn. Samus retains her pose. The next scene may choose her pose from the ordinary player
controls. The intro hides the scene later that turn, after counting down existing hit timers.

### DemoReceivesScene

- kind: state transition
- summary: Start the selected demo without spending its first room actor turn.
- params: none
- switch: Fixed
- The intro begins the battle or discovery → enter Battle or Approach respectively with zero
  timeline progress and a neutral previous pad. Publish the corresponding start handoff
  [R-battle-ready, R-discovery-ready, C-start-after-lock].
- No start, and no room actor opportunity or the demo is Off, Battle Locked or Released
  → retain all demo memory and publish no pad or demo handoff
  [R-battle-before-pad, R-battle-paused, R-battle-locked, R-battle-hidden, R-discovery-after, C-actor-closed].
- checked: oracle (R-battle-ready, R-discovery-ready, R-battle-paused, R-battle-locked, R-battle-hidden, R-discovery-after)

### DemoPlaysPad

- kind: input source
- summary: Supply the timed pad and the preceding sample to the room actors.
- params: three timelines of typed pad holds
- switch: Fixed
- An active phase has an actor turn and a timeline sample left after the scene cues
  → publish that sample at the current timeline progress, with fresh presses only on the first
  turn of its hold. Also publish the last sample from before this turn. Advance progress by one
  and make the new sample the last pad. A first Watch sample can follow a held Left from
  Approach [R-battle-first, R-battle-hold, R-battle-partial, R-battle-refire, R-battle-jump,
  R-approach-reaches-baby, R-watch-aim, C-two-turn-press, C-repress-while-held].
- Approach or Watch has an actor turn with no sample left and no scene change
  → issue Left with no fresh press and include the last pad from before this turn. Carry the
  new sample as last pad and hold progress at the end of the timeline
  [R-approach-keeps-left, R-watch-keeps-left, C-discovery-left-tail].
- checked: oracle (R-battle-first, R-battle-hold, R-battle-refire, R-battle-jump, R-watch-aim, R-approach-keeps-left, R-watch-keeps-left)

### SamusReachesEgg

- kind: scene cue
- summary: Trade the walk for the watch when Samus reaches the baby.
- params: stopping line 178 px
- switch: Fixed
- Approach has an actor turn and Samus's starting whole X is less than 178 px → start Watch and
  use its first sample immediately, even if Approach would have still had a hold left
  [R-approach-reaches-baby, C-approach-inside-hold].
- Approach has an actor turn and Samus's starting whole X is at least 178 px → keep the Approach
  timeline [R-approach-outside-line, C-approach-exactly-at-line].
- checked: oracle (R-approach-reaches-baby, R-approach-outside-line)

### BattleLocksOnLeft

- kind: handoff
- summary: Finish the battle's controls with Samus standing left.
- params: none
- switch: Fixed
- Battle has an actor turn after its last sample → publish neutral held and fresh sets with
  the previous pad and Stand Left. Enter Battle Locked with neutral pad and no
  timeline progress [R-battle-lock, C-lock-from-running].
- checked: oracle (R-battle-lock)

### DiscoveryReleasesSamus

- kind: handoff
- summary: End the discovery's control as the return crosses zero.
- params: none
- switch: Fixed
- An actor turn begins in Watch with crossfade count zero → publish neutral held and fresh sets
  with the previous pad and Release. Enter Released with neutral pad and no timeline progress
  [R-discovery-release, C-watch-release-during-hold].
- The count is nonzero or the actor turn begins in Approach → there is no discovery release
  this turn [R-watch-keeps-left, C-look-on-zero].
- checked: oracle (R-discovery-release, R-watch-keeps-left)
