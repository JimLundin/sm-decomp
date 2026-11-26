# Blackout in the shaft

Vectors: [screens-blackout.toml](../../scenarios/vectors/screens-blackout.toml).

## Header

The rotating shaft reports that Samus reached the elevator during ordinary play. This
scene carries her from that report to the pending [station explosion](https://github.com/JimLundin/sm-decomp/blob/d45f84b0d/docs/behaviours/screens-explosion.md).
It owns the wait on the platform, dimming and the ordered handoff. The shaft decides
whether she reaches the elevator and continues to own its rotation, actors, platform and
beacon. Samus's movement and appearance owners apply the standing pose and lock.
Timer owns the remaining escape time. Save files owns the battery and its pending checks.
The scheduler supplies room work opportunities, the end of the vertical interval, the
completion of the save and the requested display publication. The scene supplies its sound
requests to audio and the handoff to the explosion.

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| phase | Shaft, Hold, Fade, AwaitBlank, Saving, Explosion, Dispatched | ElevatorStopsSamus, PlatformWait, DepartureHandoff |
| wait | room work passes remaining, 1 through 60 in Hold | PlatformWait |
| intended light | lit 1 through 15 or forced blank | PlatformWait, DepartureHandoff, display |
| displayed light and pending picture | platform publication and carried room writes | DepartureHandoff, display |
| carried route | typed destination separate from current area and station | DepartureHandoff, save, explosion |
| frozen countdown | existing timer with position, digits and activity | PlatformWait, DepartureHandoff, drawing |
| pending save | active file, live capture and writing/check progress | save, DepartureHandoff |

Restore the scene phase with the corresponding pending room picture, scheduler counts, save
state and destination. A suspended cut may have completed room work without publishing its
picture. The active room, current placement, full Health State and the palette continue across
the handoff. Explosion setup owns subsequent changes to those that it uses.

### ElevatorStopsSamus
- kind: Mechanic
- summary: Samus stops on the elevator and the escape clock holds.
- switch: Fixed
- ordinary shaft work reports the reach after Samus's movement and room drawing → ask for a standing pose with her current facing, cancel a pending pose change and lock her. Freeze the countdown at the value produced by that room pass. Set the wait to 60 room work passes, with light held at its current level [B-reach].
- A missing reach or a duplicate report while the scene is active leaves the wait alone [B-no-reach, B-duplicate].
- checked: oracle (B-reach, B-no-reach, B-duplicate)

### PlatformWait
- kind: Timing
- summary: The shaft keeps running through the wait and the fade.
- switch: Fixed
- a new work pass while waiting or dimming → draw the frozen countdown when active, then run the locked room, including its shaft, actors, weapon updates, game clock and shake. Keep both the drawing it prepares and the state it leaves [B-wait, B-dim, B-blank].
- a waiting pass ends → subtract one from the wait. At zero enter dimming with no delay between brightness steps and leave brightness alone for that pass [B-wait, B-enter-fade].
- a dimming pass ends → decrease brightness by one. From level 1 set forced blank and suspend after the room work to wait for the end of a vertical interval [B-dim, B-blank].
- A scheduler interruption after work holds room clocks, the prepared picture and its earlier publication until that work resumes [B-no-repeat, B-blank].
- checked: oracle (B-wait, B-enter-fade, B-dim, B-blank, B-no-repeat)

### DepartureHandoff
- kind: Mechanic
- summary: The blackout finishes the save and hands the scene to the explosion.
- switch: Fixed
- end of the vertical interval while awaiting blank → shut off the shaft's interrupt and raster work and prepare the gameplay display mode with only sprites on its primary image, with colour blending and window masking off. Select the Zebes flight route and capture the active file from the live Ceres area and station, including her actual energy and elapsed play time [B-save-start].
- save suspended after payload → hold the in-progress save, destination and room, along with the still-active timer and shaft stage. The earlier display publication and the pending final shaft picture both survive the interruption [B-save-start, B-save-restore].
- save completed → clear the shaft stage and timer activity. Retain the timer digits and affine pose. Mark the station explosion pending with its carried destination. A pending explosion cannot dispatch before the final room publication [B-save-finish]. Request a music stop with delay 8 work intervals, then one silence on each of the three sound channels in order. The explosion owns its next setup and music requests [B-save-finish, B-save-restore].
- the requested display publication finishes the work → commit forced blank and the final prepared room picture with the updated display settings, count one finished work pass and retain the resulting state for the explosion. Keep the prepared timer sprites through the commit even after retiring the timer. This publication enables dispatch on a later work opportunity [B-publish, B-save-restore].
- a new work opportunity after the final room publication → dispatch the pending explosion once [B-dispatch-once].
- checked: oracle (B-save-start, B-save-finish, B-publish, B-save-restore, B-dispatch-once)
