# Super Metroid rebuild

The language of a purpose-built rebuild that preserves Super Metroid's gameplay and content. These terms distinguish the player's progress, a running game, and the material used to reconstruct it.

## Play and time

**Session**:
One running instance of the game, including its current room, actors, player, progression, and game mode.
_Avoid_: Engine, world, global game

**Tick**:
One numbered advance of the Session according to the selected reference profile. A display refresh is not necessarily a Tick.
_Avoid_: Frame, update

**Reference profile**:
The selected release and region of the original game, together with the observable behavior and timing used to judge the rebuild.
_Avoid_: Original version, compatibility mode

**Game mode**:
The Session's current play context, such as exploration, a door transition, the pause map, or an ending.
_Avoid_: Host pause, application state

**Host pause**:
A suspension of Session advancement imposed by the application, such as while the window is inactive or a device recovers.
_Avoid_: Pause menu, game mode

**Timeline epoch**:
An uninterrupted run of Session history; loading, restoring, or restarting begins a new epoch even when the Tick number repeats.
_Avoid_: Content version, frame ID

## World and identity

**Actor**:
A live, independently identifiable participant in a room, such as an enemy, projectile, or boss part. A multipart boss can own the behavior of several Actors.
_Avoid_: Object, entity when Actor is intended

**Actor handle**:
A temporary identity for one live Actor incarnation. It ceases to name that Actor when the incarnation ends.
_Avoid_: Content ID, persistent ID

**Simulation order key**:
An Actor's place in the defined order of gameplay evaluation. Identity and evaluation order are separate concepts.
_Avoid_: Actor handle, storage index

**Room instance**:
The temporary, live realization of an authored room on a particular visit.
_Avoid_: Room definition, level

**Progression**:
The acquired abilities, collected items, explored map, and persistent events that outlive individual room visits.
_Avoid_: Entire game state, checkpoint

## Content and development

**Authored ID**:
The lasting identity of a room, item, flag, media asset, or other authored definition across content revisions.
_Avoid_: Pack index, Actor handle

**Content revision**:
One validated set of gameplay definitions and media prepared for use together.
_Avoid_: Timeline epoch, build number

**Gameplay fingerprint**:
The identity of the rules and definitions that determine Session behavior, including the reference profile.
_Avoid_: Media revision, file timestamp

**Media revision**:
The identity of the visual and sound material used to present a Session without changing its gameplay rules.
_Avoid_: Gameplay fingerprint, save version

**Scenario**:
A named, reproducible starting situation for developing or checking a specific piece of the game.
_Avoid_: Arbitrary live edit, save slot

**Runtime effect**:
A request produced by a committed Tick for something outside the Session, such as playing a sound or writing a save.
_Avoid_: Scene, generic event

**Scene**:
A complete description of what to draw for a committed Tick, including the game image and its interface.
_Avoid_: Mutable world, framebuffer

## Persistence and evidence

**Player save**:
The player's durable progression and restart location, sufficient to resume through the game's normal loading rules.
_Avoid_: Checkpoint, memory dump

**Checkpoint**:
A complete recoverable description of a Session at a committed Tick, including transient gameplay state.
_Avoid_: Player save, audiovisual snapshot

**Replay**:
A compatible starting state and an ordered record of Tick inputs and admitted external results, sufficient to reproduce Session history.
_Avoid_: Video, reference trace

**Reference trace**:
Observations captured independently from the selected original game for comparison with the rebuild.
_Avoid_: Replay of the rebuild, golden output generated only by our code
