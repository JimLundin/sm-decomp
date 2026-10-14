# Reserve palette fixture correction

Follow-up manifest commit:
`fa6e9cd747acabcb4568ccd2c0fbf532a9525c18`.

Exactly 12 nonempty-reserve cases changed their independent expected return
from 3072 to 1536. All 24 palette cases now explicitly expect 1536 (0x0600).
The 12 empty-reserve expectations were already correct. Gameplay source and
Rust test bytes remain those of `a026c883cbd651663c0945155fe409886306e384`;
the implementer worktree is clean and no verification writer was launched.

The rejected generic gate reported only these expectation mismatches:
`/home/jim/sm-decomp/out/port/decomp-coverage-20261013/pause-equipment-render/1791910513749072-819571-0/verdict.toml`.

Independent evidence:

- Original ROM `rom/sm.sfc`, file offset 0x14100 (82176), contains bytes
  `00 06`. This is `$82C100`, table `$82C0FA` plus 3 words:
  `kPAuseSpritePaletteIndexValues[3]` in pinned `sm_82.c`.
- Actual-C `palette-health-1-timer-0000-frame-0/c.stdout` reports `1536`;
  its paired Rust stdout reports `1536`.
- Actual-C `palette-health-0-timer-0000-frame-0/c.stdout` reports `1536`;
  the empty branch in pinned C explicitly returns `1536`.

The initial scratch fixture generator incorrectly borrowed the synthetic
Rust test ROM palette (0x0c00) as an original-ROM expectation. The maintained
manifest now contains the correct original-ROM expectation. The retained
historical generator must be corrected before any future regeneration; it is
not the authoritative fixture input. Parent owns the corrected generic gate,
authentication, cleanup, result admission and progress publication. The sealed
360-pair direct-animation gate is unaffected by this manifest-only edit.
