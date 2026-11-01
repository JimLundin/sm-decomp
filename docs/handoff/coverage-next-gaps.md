# Next retail coverage batch

Source planning starts from integration `a64f9ad698db94c56706e6c03866ae92caf0c6c7`.
The inherited two scenarios, pins and original qualification are unchanged.
The next scenarios are not yet qualified or enabled: they require faithful
reachability, both lockstep oracles, free run, locks and actual mutant failures.

The planned pair uses the real 100% Tourian entry seed `100-input-phase:317`.
One scenario changes Samus's equipment to ice/plasma/charge, unequips screw
attack, unmorphs, freezes a Metroid and fires ordinary missiles. Its facts
require a fresh freeze and death of that same frozen slot, an ordinary missile
and a fresh explosion, with competing supers/bombs/power bombs excluded.
The other preserves the qualified latch seed and original 333-tick approach,
cancels the HUD selection, morphs and lays ordinary bombs. Its release fact
requires a fresh bomb-off event followed by the same live Metroid chasing
again. A loaded frozen or bombed-off actor does not establish either episode.
These are proposed inputs; they earn no reached behavior until replayed.

The post-tick facts follow the real update order: projectile contact runs
before the selected enemy's AI. Metroid shot sets frozen time to 400 or the
bombed-off timer to four; that same tick's AI consumes the first tick.
The snapshots therefore observe 399 or three, respectively. The earlier
private bomb recipe's timer-four assertion could not observe this boundary;
its retained trace also showed no bomb-off activity, so input reach still
needs repair. Its private recipe and receipts are preserved.

Botwoon's three-projectile fan cannot arise through the retail producers.
The [bank $B3 log](https://patrickjohnston.org/bank/B3) marks `$B3:9696` unused;
its `$96B1` store installs head function `$9E7D` without changing the boss
function. That is the producer that could select the three-shot branch at
`$9ED7`. The live producer at `$991C` installs `$9E7D` together with boss
function `$99E4`, so `$9ED2` instead selects the five-shot branch `$9EE0`.
The current engine has only that live producer. No scenario installs an
enemy function to manufacture the three-projectile fan.

Grapple spike contact/anchored placement and Etecoon ceiling contact remain
deferred, not proven unreachable. T3 found no suitable standing/crouching
grapple block or reachable spike geometry; T4's 4,000-tick release reached no
Etecoon ceiling contact. A fresh source-only scan of 198 retained 100% room
geometries found no grounded enemy-anchor candidate among 31 attach-capable
enemy observations either. That scan covers recorded positions, not all
possible enemy movement. These gaps retain no retail-unreachable annotation.

Complete any%, 100% and low% movie scope remains open as listed in
[the inherited inventory](coverage-movie-gaps.md). These two interactions
cannot substitute for boot, wait intervals, carried state or completion.
