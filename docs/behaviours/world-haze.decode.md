# Ceres haze decode and harness

Companion to the [Ceres haze behaviour](world-haze.md). Keep this note
outside confined exports.

## Sources

| Entry | Origin of the ordinary haze decision |
| --- | --- |
| HazeBrightens | $88:DE2D through $88:DE6C. Sets black and the blend intent, prepares the gradient, and advances its scheduled peak. |
| HazeSettles | $88:DE2D through $88:DE45 and $88:DE6D through $88:DE73. Selects the steady owner and preserves the current band backing. |
| HazeHolds | $88:DE74 through $88:DE8E. Sets black and the blend intent, checks that the room remains, and preserves the visible bands. |

The display consumer at $88:8142 interprets the selected colour combination. The scanline placement at $88:DF03 is display content.

- HazeBrightens and HazeSettles use [bank 88](https://patrickjohnston.org/bank/88#fDE2D),
  $88:DE2D through $88:DE73. The fade-in computes sixteen component
  levels and increments the maximum below sixteen. At sixteen it changes
  the pre-instruction to the faded-in handler without touching bands.
- HazeHolds uses [bank 88](https://patrickjohnston.org/bank/88#fDE74),
  $88:DE74 through $88:DE95. The ordinary no-departure arm preserves the
  bands. The departure arm is outside this scope.
- All three request black for the base backdrop and the haze blend
  configuration. The display-side combination belongs to [the
  layer blend handler](https://patrickjohnston.org/bank/88#f8142).
  The arrangement of line spans is display content. It gives the band
  indices meaning without adding the original transfer format to the model.

## RAM mapping

| Quantity | Source | Comparison |
| --- | --- | --- |
| fade | $7E:18F0 and $7E:1914 for the active backdrop object | Decode the role and maximum. Faded-in at maximum sixteen is Steady. |
| colour | $7E:1920 for the active backdrop object | Decode the one selected colour component. |
| bands | $7E:9D00 through $7E:9D0F | Compare the sixteen levels and each byte's single colour component. |
| backdrop and blend decision | $7E:0074 through $7E:0076 and $7E:1986 at the producing operation | Compare the three zero components and third-background exclusion before dependent display work. |

Band levels are the low five bits; component selection is the upper three.
Decode both. No observed band bits are thrown away. The ordinary hold
does not interpret the latched maximum as a fresh ramp. The adjacent
capture observes the fade role write and has zero band-buffer writes.
The first opportunity writes all sixteen band bytes, then publishes the
advanced maximum. It reaches the band-ready cut before later display
preparation consumes the completed owner. Transport object countdown and
eligibility state belong to scheduling rather than the visible haze state.

## Harness contract

Use the pinned lsnes start and pads from row chain-any-0003-DFD7 in
[#595](https://github.com/JimLundin/sm-decomp/issues/595) and the
observer's continuous native capture. The first wram and callback record
are checked through the independent first witness. The recorded vectors
cover postRTI9232 to 9233 (R-bright-edge), 9233 to 9234 (R-reaches-steady)
and 9234 to 9235 (R-steady). The last vector observes the completed fade
and band fields at both cuts, without an all-bus lifetime assertion. Run
the held-input intervention from the same seed, changing only the second
sample to hold the first sample. It independently yields the same three
fade states and full band arrays. The pad input changes the native tail
duration while preserving these haze observations.

Replay should load one state and carry it through the three opportunities.
Observe the band's publication for the first opportunity and the mode
transition without a band write for the second. Compare the completed state
and backdrop and blend intent separately from physical clock and native
video/DMA observations. Use the full band field, with zero masked bands.
The reference model covers eight vectors, including constructed initial and
restored states. State equality on recorded opportunities is 3/3 versus
1/3 for frozen state. Premature peak advance, missed settling, recreated bands on settling and
suppressed replacement at equal levels fail their named vectors. Integrated display, unrelated
layer and transport lifetimes, and played-alone coverage require production
replay.

## Budget

Fixed base 9525e5966f0c158faa3e33b8054066013932c6f1. No attributable portable
haze simulation exists there. The production prelude carries externally
borrowed haze readiness and terminal receipts. Their scheduler and transfer
consumers also serve timing responsibilities outside these three rules.

k = 37.5 weighted tokens per spec unit.

- Area backdrop: HazeBrightens, HazeSettles, HazeHolds

The spec-units counter reports three entries, three arms and three
operators. Budget: 4.0 units, 150 weighted tokens.
