# Samus at the start of the intro

The new file supplies Samus's inventory and controls. The intro's opening creates her
hidden actor for the played scenes and lends the equipment that those scenes use.
Movement, appearance, health and weapons retain their own State. The intro decides
when to enter a played room. The room start, the recorded controls and encounter rules determine
what happens once she enters it. This entry supplies the actor those rules take.

Positions are signed 16.16 pixels. The two halves of a position are its whole pixel
and its fraction. A concealed actor can be saved and restored at a cut inside the
opening's first frame or while text is on screen.

| Quantity | Type and units | Read by |
| --- | --- | --- |
| Samus | Movement State and figure. Current centre in pixels, pose, speed and movement memories in their ordinary types. | room start, samus-movement, samus-appearance |
| Her prior centre, prior pose, older pose and cached body radius | Position in pixels, two Poses and two whole-pixel radii | room start, output |
| Her appearance | AppearanceState, including the clip's step and hold in frames and the cannon's remaining eligible control frames | room start, samus-appearance |
| Her health | Health State, including energy and protection frames | weapons, fight, output |
| Her shots and selected item | Weapons State, including shot slots and cooldown frames. Selection in the item owner's type. | weapons, samus-appearance |
| Her supplies | Inventory counts and capacities, in whole units | weapons, run-progress, output |
| Her control map | Physical pad button per action. Keep the player's map for the equipment return. | demo controls, IntroEnds |
| Whether Samus is shown | Hidden or shown, with the intro's draw order when shown | actor turn, drawing |

Cells: quantities, domain types and readers.

### SamusEntersTheIntro

- kind: mechanic.
- summary: the intro readies Samus and her missile supply while the opening still hides her.
- params: none.
- switch: Fixed.
- New-file options hand on to the intro → carry the chosen file's supplies and controls into
  the opening. Keep the actor unstarted [R-new-file, R-hand-on].
- The intro begins its first frame → create hidden Samus at rest facing the screen, with zero
  current and previous centre, both poses in her history facing the screen and cached radii zero.
  Use the ordinary initial motion and appearance, including step 0 with no hold and two
  eligible control frames left before the cannon follows selection. Start with no selected
  item, no protection or pending loss, no knockback, and empty shot slots with zero cooldown.
  Carry the file's energy into her health. Set the missile count and capacity to 900 and keep
  the other inventory quantities. Keep the player's control map for the return and apply the
  intro's default map [R-opening-setup, C-opening-loan].
- A cut inside the first frame after setup, or a later finished frame with Samus hidden
  through the page-one wait → carry her State, supplies and controls as they stand. Her
  movement, clips, item controls and weapons take no turn [R-opening-finished, R-page-one-wait,
  C-hidden-actor].
- checked: oracle (R cases); constructed (C cases).

## Order that matters

The first intro frame can cross an interrupt after the actor is created and the loan applied.
Complete the same frame using that actor. The press at the end of page one starts the fight
room with the carried actor. The existing room-start rule selects missiles, puts her whole
current and previous centre at (155, 115) px and gives her the left-facing standing pose.
Her pose from the text remains in history until the first turn. The room start takes the first
hold from the standing clip. On the first completed room turn, her body acquires its ordinary
standing bounds and movement applies ground pull. The neutral demo pad leaves her selection
unchanged. The clip and the cannon delay each advance once.
