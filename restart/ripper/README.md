# Ripper initialization and patrol

`sm-ripper` owns four recovered roots: JetPowerRipper_Init A2E318,
JetPowerRipper_Main A2E353, Ripper_Init A2E49F and Ripper_Main A2E4DA.
It does not own shot/frozen/contact/death/draw, the instruction interpreter,
enemy scheduling or terrain collision.

The typed `execute(entry, ram, rom, services)` boundary returns
`Result<Option<()>, String>`. `None` refuses without modifying WRAM or issuing
a request. It requires exactly128KiB WRAM, a current64-byte slot below0800,
and the complete520-byte runtime linear-speed table at file offset110187.
The caller binds that ROM to its selected native identity. The implementation
reads runtime table bytes, never computes replacement speeds or silently
substitutes an original-ROM table for a different prepared table.

Init admits `parameter1.wrapping_mul(8) <= 0200`, including row64 and wrapping
aliases such as2001 for row1. Parameter2 is tested for nonzero. Init modifies
only instruction, stored rowE, integerD and fractionC; it preserves timers and
all other fields. Main admits stored E<=0200 divisible8 and obtains availability
before the native request. Its signed16.16 amount comes from D:C, not A:B.

The single `Services::move_horizontal` call performs actual
Enemy_MoveRight_IgnoreSlopes A0C6AB with synchronous full-WRAM publication and
copyback. Result0 leaves all returned state intact. Result1 reads the stored row
and D from the original record after copyback, reloads the opposite table
direction and resets animation/timers. This deliberately differs from simply
negating current speed. Invalid returned collision, changed invalid row or
accepted native error is fatal to the owned invocation, never C fallback.

Feature `fixture-support` exports `BodyFixture { name, entry, wram }` and
`body_fixtures()`:46 shared valid-C inputs (20Init,26Main) for native controls.
The10 unit tests use independently transcribed endpoint table rows and explicit
service models; real terrain and PLM behavior require root's actual-C controls.
See [HANDOFF.md](HANDOFF.md) for the exact input contract and pending gates.
