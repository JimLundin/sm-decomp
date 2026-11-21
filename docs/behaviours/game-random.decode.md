# Shared random sequence decode and harness

Companion to the [shared random sequence](game-random.md). Keep this note
outside confined exports.

## Sources and ownership

| Entry | Shared sequence evidence |
| --- | --- |
| AdvanceRandom | $80:8111 reads and replaces the current sequence number. The ordinary gameplay dispatch at $82:894F and the draw sites at $A0:B8F7 and $A6:B324 enter that same update on the same word. These sites establish that completed draws share one sequence. |

The arithmetic inside the draw comes from the sequence update. The observations at its callers establish sharing across callers. A consumer reading the current number does not perform another draw.

[Bank 80](https://patrickjohnston.org/bank/80#f8111) at $80:8111–8145 reads
both halves of the shared sequence at $7E:05E5–05E6. It obtains a complete
first product and the low byte of the second product. The first product's
high byte participates in the high-byte addition. That addition starts
with carry set and its carry-out contributes one to the final word
addition. The final word addition wraps independently. Comparing the
intermediate wide value before truncation gives the same carry; reducing
the whole product modulo 65536 first can lose it.

The earlier [Ridley decode](boss-ceres-ridley.decode.md#inputs) used
the shorthand five times the previous number plus 273. The bank log
identifies the extra unit when the high-byte addition carries. The
simplified formula fails C-carried-overflow and C-high-product-overflow.
The current evidence concerns the ordinary Ceres prelude. It preserves the
earlier note's observed door timing and does not repeat the escape census.

[Gameplay dispatcher](https://patrickjohnston.org/bank/82#894F) calls the
advance before main gameplay. [Bank A0](https://patrickjohnston.org/bank/A0#B8F7)
and [bank A6](https://patrickjohnston.org/bank/A6#B324) have independent
call sites entering the same advance. They change the same $7E:05E5 word.
Ridley's door logic's read of that word is a consumer rather than an
additional demonstrated draw in the escape census. These uses establish
a shared sequence in the original. They do not establish the order or
count of draws across all production consumers.

## RAM mapping

| Quantity | Source | Comparison |
| --- | --- | --- |
| current number | $7E:05E5 before the actual opportunity | Compare the full little-endian word. |
| resulting number | $7E:05E5 at the completed advance | Compare the full word with no mask. Subsequent reads observe this value until the next draw. |

## Harness contract

Use the pinned lsnes row chain-any-0003-DFD7, two consecutive requested
native returns from postRTI9232, and the separate held-input control.
R-first samples the original advance from postRTI9232 to postRTI9233.
R-adjacent samples postRTI9233 to postRTI9234. The second input equals the
actual first terminal value both at the cut and when the next producer
reads it. The matched terminal numbers also persist to their requested
cuts in both trials. The reference matches two of two samples versus zero
for frozen state. It agrees with all eleven vectors and with an independent
byte-step projection for all 65,536 initial values. Planted loss of either
carry, saturation and recycling the final carry fail their named vectors.
Using the first starting value again fails the adjacent result.

The current timed service lasts 614 intrinsic master ticks in both
intervals. The actual timing owner writes two operand pairs and exposes
the first product's low and high samples and the second product's low
sample. It owns the physical samples throughout pauses or native
interruptions. The semantic state commits only after the actual operation.
Compare completed live device results and the sequence, and never supply
captured products or the terminal number as production input. The first
and adjacent intervals use the same exposure profile. Production proof of
a wrong-product rejection, split budgets, suspend and restore, and the
next cross-domain consumer remains an integration obligation.

## Budget

Fixed base 9525e5966f0c158faa3e33b8054066013932c6f1. No attributable portable
advance exists at that base. The timed service prepares and samples the
hardware while a borrowed receipt supplies sequence completion. Timing
remains outside this model area.

k = 37.5 weighted tokens per spec unit.

- Area sequence: AdvanceRandom

The spec-units counter reports one entry, one arm and ten operators.
Budget: 4.3 units, 162 weighted tokens.
