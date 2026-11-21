# Shared random sequence

Each completed random draw advances one shared sequence and returns its new
number. [Vectors](../../scenarios/vectors/game-random.toml).

## Header

**Scope.** The numeric advance of the game's shared random sequence,
including consecutive ordinary gameplay draws in Ceres. It replaces the
borrowed terminal number for these draws.

**Reads.** The caller supplies the current shared sequence and an admitted
draw. The room, boss and enemy systems may request or read a number from
the same sequence through their respective integrations.

**Leaves to others.** The game schedules draws and publishes their results
to consumers in gameplay order. The integration with physical timing owns
ordered requests, samples, interrupted work and completion. Consumers
own what a random number means for their own behaviour. The spec grants
no new draw and establishes no schedule for another scene.

## Memory and boundary

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| number | whole sequence value, 0..65535, `RandomNumber` | AdvanceRandom, next consumer |
| drawn number | whole sequence value, 0..65535, `RandomNumber` | current consumer |

The current number is the complete sequence state. Every value can start
or restore a sequence, including zero. Read and restore the current
number exactly. The drawn number equals the newly advanced state. Reading
the current number without a draw leaves the sequence unchanged.

Let `lo` and `hi` be the low and high base-256 digits of the current number.
Here `div` is whole-number division and `mod` is the nonnegative remainder.

## Rules

### AdvanceRandom

- kind: mechanic
- summary: a completed draw advances and returns the shared random number.
- params: none
- switch: fixed
- Draw → let `biased = 5 * lo + 256 * ((5 * hi mod 256) + 1)`. Set the
  current number to `(biased + 17 + (biased div 65536)) mod 65536` and
  return that value [R-first, R-adjacent, C-zero, C-carry-low-product,
  C-before-final-wrap, C-final-wrap, C-before-carried-overflow,
  C-carried-overflow, C-high-product-overflow, C-max, C-restored].
- checked: oracle (R-first, R-adjacent), with constructed initial states,
  boundaries and a restored continuation.

## Order that matters

The result becomes available when a draw completes. The next opportunity uses that result, including after a restore. Multiple consumers draw from
the same state in the order their caller establishes. A current-number
read observes the most recent completed draw and leaves the state intact.
