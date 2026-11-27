# Samus's turns in the intro fight

Companion to [Samus's turns in the intro fight](samus-intro-turns.md). The
recorded-controls owner's BattleLocksOnLeft determines when the handoff occurs.
These notes cover what Samus does with it.

## Sources and mapping

| Entry | Original source and projection |
| --- | --- |
| SampleIntroBody | $90:E6FA calls $EC22 before $E700 pose input, $E70A weapons/item controls and $E833 motion and pose update. $EC25..EC39 stores x radius 5 and the entry pose's y radius from $91:B62F in $0AFE/$0B00. The locked current update $90:E713 has no radius writer. |
| RememberIntroPose | $90:E6CF..E6E1 clears prospective choices on a controlled turn. $91:8000 dispatches pose input, including the running chooser $8066. $91:EB88..EC15 resolves the animation-end, special or ordinary proposal. An admitted proposal reaches $EBF8..EC0D, which moves $0A20/$0A22 into $0A24/$0A26 and remembers the resulting pose and facing/action. A turn without a proposal exits at $EC10. The proposal words are $0A28/$0A2A/$0A2C. |
| RememberIntroPose, handoff | $91:8739..8773 explicitly remembers the standing-left placement. $91:FB5C..FB8D checks the reset hint, then compares the exact pose to the old pose before restarting its clip. A different pose facing left can restart. The movie already has standing left. The $90:8000 turn advances its existing clip. |
| FinishIntroControlTurn | The handoff installs $90:E713 for future current updates. The $E6C9 call already underway still returns from $E91D through the neutral sample's ordinary pose chooser and runs $E70A. Future $E713 turns call $AECE for existing projectiles and $EB02 for transient travel and aim reset. $EB02 leaves the three prospective pose words alone. $E833 remains the later call and continues motion, animation, reactions, pose and palette. $8B:8E0D still ticks hit timers. |
| Previous centre | Room entry at $8B:AEF5..AF04 writes whole coordinates. The actor path bypasses the camera mirror writer $90:956E..9583. In the witnessed fight no placement or reaction rewrites $0B10..0B16 after entry. |

[Bank 90](https://patrickjohnston.org/bank/90) · [Bank 91](https://patrickjohnston.org/bank/91) · [Bank 8B](https://patrickjohnston.org/bank/8B)

The movement model already uses Choice::chooses_anew before recording its coarse
spin history. The room needs that resolved decision to publish full pose history.
A same-pose choice can advance the history. Two NonSpin values cannot reveal
whether a choice happened. The current frame outcome omits this fact. The room needs the movement owner's
decision to carry its full history. The inherited standing
proposal on the locked movie path resolves to the already remembered standing pose.

## Witnesses

| Vector | Cut in any-intro-2219 | Observation |
| --- | --- | --- |
| R-recoil-to-fall | 181, movie 2,400 | The left recoil gives way to falling left. The cached y radius is still 21 px. |
| R-fall-reselected | 182, movie 2,401 | The radius becomes 19 px. Falling left is selected again and replaces recoil in the older pose. |
| R-run-carries | 256, movie 2,475 | Holding Left moves Samus and advances her clip. The three proposals are absent. Running left remains current and standing left remains older. |
| R-lock-handoff | 396, movie 2,615 | The handoff places the standing-left pose she already has. Step 0 has 14 frames left after it had 15. The last ordinary selection leaves the standing proposal. |
| R-locked-stands | 397, movie 2,616 | The clip's hold becomes 13. The proposal and radius carry, with standing left still older. |

The scoped reference imports pose radii and reads proposal presence and final pose
as movement observations. It uses the observed current-update authority at entry.
Across row cuts 1–648 the radius, pose history and previous centre match all 644
owner turns, including 393 controlled turns and 251 already locked turns. Four cuts
before actor work hold the mirrors. The previous centre remains (155, 115) px with
zero fractions. The reference plays the imported standing clip for all 252
handoff and locked turns. This checks the projections and owner opportunities. Movement's
choice and collision still need joined production proof.

Sampling the final pose first differs at movie 2,400. Always shifting history
first differs at 2,475. Shifting only on a different pose first differs at 2,220,
when standing is selected again. Holding the clip across locked turns differs at
2,616. The constructed body case gives the locked actor a carried radius different
from the pose's body. It distinguishes retention when the movie's radius holds 21 px.

The constructed handoff starts in left-facing recoil. The standing placement
records standing before the ordinary standing choice records it again. Both
history entries become standing. The imported standing clip starts with 10
frames left and the animation tick leaves 9. Skipping the placement's history
write leaves recoil as the older pose. A restart test that compares only facing
keeps the recoil playhead and leaves step 1 with 6 frames. The vector detects
both faults.
