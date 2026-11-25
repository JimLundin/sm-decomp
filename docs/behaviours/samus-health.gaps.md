# Current energy: missing capabilities

This bounded [spec](samus-health.md) does not complete H1.

1. Run Progress pickup exchange needs one live energy authority and proof of how a
   pickup interacts with fractional energy and pending loss. Its current Inventory
   API remains a pickup model; no implemented cross-owner exchange is claimed.
2. Actual pending-loss producers, accumulation, frame admission and freshness
   require their respective production contracts. Scoped independent observations do
   not prove those producers. Animation, reading energy after the frame's hit
   and before its periodic loss, and drawing, reading the energy the frame leaves, must use the same current health
   owner; previous drawing history stays separate.
3. Damage reduction for Varia or Gravity and behavior when time is stopped require
   separately qualified rules and witnesses. Unsupported contrasts are C-varia-missing,
   C-gravity-missing and C-stopped-missing.
4. An eligible hit in Ceres, worth what its hitter gives, and the protection it grants
   have rules. Integrated replay needs the actual hitters' eligibility, impact and worth
   producers (Ridley's tail, worth 15, and the escape's steam vents, worth 0) and the
   Drawing and status consumers of the frame's outputs. Native carry proves a
   protected actor check. Overlap during protection is constructed because the actor
   skips contact geometry on that check. Other attacks, alternate protection
   sources, stop-time protection behavior, knockback, and what she shows when hit
   remain outside the admitted scope. Escape countdown has its own clock.
5. Reserves, refill, capacity changes, death and game-over effects remain missing.
   Reaching zero here is only energy arithmetic, not proof of what follows her death.
6. Warning sound acceptance and resulting audible playback belong to audio. This
   model proves requests and warning state; it does not prove that a request is heard.
7. Appearance's `Breathing` reads energy after the frame's hit and before its
   periodic loss. That differs from State as the frame starts on a hit frame (24, not
   39, on R-tail-hit) and from the energy the frame leaves when the frame also loses
   energy periodically. Health's one-frame step gives back neither. An integrated Health-to-Appearance read needs an "energy as animated" output
   carrying that value, or it is Unsupported. No integrated caller reads it today:
   the Appearance cases borrow energy.
