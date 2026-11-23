# Current energy: missing capabilities

This bounded [spec](samus-health.md) does not complete H1.

1. Run Progress pickup exchange needs one live energy authority and proof of how a
   pickup interacts with fractional energy and pending loss. Its current Inventory
   API remains a pickup model; no implemented cross-owner exchange is claimed.
2. Actual pending-loss producers, accumulation, frame admission and freshness
   require their respective production contracts. Scoped independent observations do
   not prove those producers. Animation, reading State as the frame starts, and
   drawing, reading the energy the frame leaves, must use the same current health
   owner; previous drawing history stays separate.
3. Damage reduction for Varia or Gravity and behavior when time is stopped require
   separately qualified rules and witnesses. Unsupported contrasts are C-varia-missing,
   C-gravity-missing and C-stopped-missing.
4. Ridley's tail impact in Ceres and the protection it grants have rules.
   Integrated replay needs the actual Ridley eligibility and impact producer and the
   Drawing and status consumers of the frame's outputs. Native carry proves a
   protected actor check. Overlap during protection is constructed because the actor
   skips contact geometry on that check. Other attacks, alternate protection
   sources, stop-time protection behavior, knockback, and what she shows when hit
   remain outside the admitted scope. Escape countdown has its own clock.
5. Reserves, refill, capacity changes, death and game-over effects remain missing.
   Reaching zero here is only energy arithmetic, not proof of what follows her death.
6. Warning sound acceptance and resulting audible playback belong to audio. This
   model proves requests and warning state; it does not prove that a request is heard.
