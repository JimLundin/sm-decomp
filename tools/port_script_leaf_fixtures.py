#!/usr/bin/env python3
"""Native bank86 script calls across every projectile slot and real ROM operands."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'tools/port/ceres-platform.json').read_text())
    routines = []
    for symbol in ('EprojInstr_Delete', 'EprojInstr_SetPreInstr_',
                   'EprojInstr_ClearPreInstr', 'EprojInstr_Goto'):
        cases = []
        for slot in range(0, 36, 2):
            for pointer in (0x868000, 0x868161, 0x869754, 0x86a28b, 0x86fffd):
                if symbol == 'EprojInstr_Goto' and pointer == 0x868161:
                    continue
                values = [(0x1997+slot, 0xbeef), (0x1a03+slot, 0xabcd),
                          (0x1997+(slot+2)%36, 0xface), (0x1a03+(slot+2)%36, 0x1234)]
                cases.append(dict(name=f'{symbol}-{slot}-{pointer:x}', a=slot, b=pointer,
                                  writes=[[p, [v&255, v>>8]] for p, v in values]))
        routines.append(dict(symbol=symbol, arg_types=['uint16', 'rom_ptr'],
                             **{'return': 'rom_ptr'}, pointer_bank=0x86, cases=cases, live_eligible=True))
    manifest['family'] = 'projectile-script-leaves'
    manifest['routines'] = routines
    (ROOT / 'tools/port/projectile-script-leaves.json').write_text(json.dumps(manifest, separators=(',', ':'))+'\n')


if __name__ == '__main__':
    main()
