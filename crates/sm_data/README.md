# sm_data

Validated game content. Each record is built from bytes by an importer and
checked as it is built; no ROM layout, RAM or emulator type crosses this
boundary. The package has no dependencies and forbids `unsafe`.

Today the bytes come from the original cartridge. `cartridge.rs` holds the
shared cartridge image (`RomImage`), which counts writes so a record can tell
when a bug-fix patch has made it stale. `table.rs` provides fixed-size
little-endian word tables. The other modules are one record family each, for
example enemy speeds, beam gradients, boss and enemy lists, Kraid, Crocomire,
Phantoon, Ridley, Mother Brain, the Chozo and Tourian statues, credits,
title screen and menus. `raster.rs` and `scene.rs` hold the value records shared with
`sm_runtime`.

## ROM import check

[`tests/rom_import.rs`](tests/rom_import.rs) imports every record from a real
ROM and fails if any rejects it; a rejected record would leave the game on its
old ROM reads. It needs an absolute path:

```sh
SM_ROM="$PWD/rom/sm.smc" cargo test -p sm_data --test rom_import
```

Without `SM_ROM` it passes without checking. PR CI runs it with the ROM from
repository secrets and prints only pass/fail, the rejected record names and
the failing assertion's location. Add each new record to this test.
