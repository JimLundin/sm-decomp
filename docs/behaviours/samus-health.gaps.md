# Current energy: missing capabilities

This bounded [spec](samus-health.md) does not complete H1.

1. Run Progress pickup exchange needs one live energy authority and proof of how a
   pickup interacts with fractional energy and pending loss. Its current Inventory
   API remains a pickup model; no implemented cross-owner exchange is claimed.
2. Actual pending-loss producers, accumulation, opportunity selection and freshness
   require their respective production contracts. Scoped independent observations do
   not prove those producers. Animation-before-loss and drawing-after-loss must use
   the same current health owner; previous drawing history stays separate.
3. Damage reduction for Varia or Gravity and behavior when time is stopped require
   separately qualified rules and witnesses. Unsupported contrasts are C-varia-missing,
   C-gravity-missing and C-stopped-missing.
4. Direct hits, invincibility and hurt protection need their own producer and freshness
   join. Escape countdown does not decrement Samus's invincibility. No protection
   algorithm is included in periodic loss.
5. Reserves, refill, capacity changes, death and game-over effects remain missing.
   Reaching zero here is only energy arithmetic, not proof of a death sequence.
6. Warning sound acceptance and resulting audible playback belong to audio. This
   model proves requests and warning state; it does not prove that a request is heard.
