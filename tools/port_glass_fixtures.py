#!/usr/bin/env python3
"""Reproduce native-ROM glass PLM callers, allocation and scheduled lifetime."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def patches(values):
    return [[offset, [value & 255, (value >> 8) & 255]] for offset, value in values]


def next_random(seed):
    low = (seed & 255) * 5
    carry = (low >> 8) + (((seed >> 8) * 5) & 255) + 1
    return (((low & 255) | ((carry & 255) << 8)) + (carry >> 8) + 17) & 65535


def generate():
    rom_path = ROOT / 'rom/sm.sfc'
    rom = rom_path.read_bytes()
    base = dict(bank='sm_86.c', rust_module='crates/port/src/enemy_projectiles.rs',
                c_support=['sm_rtl.c'], projectile_host=True,
                rom=dict(file=str(rom_path), sha256=hashlib.sha256(rom).hexdigest()),
                domain='Even projectile slots 0..34; even PLM slots 0..78. Shard '
                       'parameters 0..5; sparkle sources 0..35, both floored /2 '
                       'as in C. Room widths use the native low byte, including '
                       'zero-divisor hardware result. Actual unmodified ROM; '
                       'full WRAM, raw return and eight external host words.')
    host = [11, 22, 33, 44, 55, 66, 77, 88]
    control = dict(symbol='EprojInstruction', arg_types=['uint16', 'rom_ptr'],
                   **{'return': 'rom_ptr'}, cases=[dict(name='dispatcher-control', a=0,
                                                       b=0x869754, host_words=host)])

    def case(name, values, a=0, b=0, fill=165):
        return dict(name=name, a=a, b=b, fill=fill, writes=patches(values), host_words=host)

    def save(family, routines):
        manifest = dict(base, family=family, routines=[control, *routines])
        (ROOT / f'tools/port/{family}.json').write_text(json.dumps(manifest, separators=(',', ':')) + '\n')
        print(f'{family}: {sum(len(r["cases"]) for r in manifest["routines"])} cases')

    def pool(free):
        return [(0x1997 + k, 0 if k in free else 0xcefc) for k in range(0, 36, 2)]

    def plm(seed=0, index=0, width=16, block=33):
        return [(0x5e5, seed), (0x1c27, index), (0x1c87 + index, block), (0x7a5, width)]

    shards, sparkles, direct = [], [], []
    # Multiplication by five covers every RNG phase byte; every native script
    # orientation is selected. Orthogonal offset, PLM and width boundaries.
    for free in (0, 2, 16, 34):
        for seed in range(256):
            parameter = seed % 6
            index = (0, 2, 40, 78)[seed % 4]
            width = (0, 1, 16, 255, 256, 0xffff)[seed % 6]
            block = (0, 1, 33, 0xfffe, 0xffff)[seed % 5]
            shards.append(case(f'shard-{free}-{seed}', pool([free]) + plm(seed, index, width, block), 0xcefc, parameter))
    for free in range(-1, 18):
        for parameter in range(6):
            shards.append(case(f'shard-pool-{free}-{parameter}', pool([free*2]) + plm(0xffff, 78, 1, 0xffff), 0xcefc, parameter))
    for destination in (0, 16, 34):
        for parameter in range(36):
            source = parameter & ~1
            for seed in (0, 1, 0x7fff, 0xffff):
                values = pool([destination]) + [(0x5e5, seed), (0x1a4b+source,0xffff), (0x1a93+source,0)]
                sparkles.append(case(f'sparkle-{destination}-{parameter}-{seed}', values, 0xcf0a, parameter))
    for k in range(0, 36, 2):
        for address, parameter in ((0xcdc5,0), (0xcdc5,5), (0xce6d,k), (0xce6d,35)):
            values = plm(0xffff,78,256,0xffff) + [(0x1993,parameter),(0x1a4b+(parameter&~1),0xffff),(0x1a93+(parameter&~1),0)]
            direct.append(case(f'initializer-{k}-{address}-{parameter}',values,k,address))
    save('mother-brain-glass-allocation',[
        dict(symbol='SpawnEprojWithRoomGfx',args=2,**{'return':'void'},cases=shards+sparkles),
        dict(symbol='EprojInitializer',args=2,**{'return':'void'},cases=direct)])

    updates, preinstructions = [], []
    for k in range(0,36,2):
        for y in (0,1,127,255,256,0xffff):
            for vx,vy,fraction in ((0,0,0),(0x0180,0xff80,0xffff),(0x7fff,0x8000,0x8000),(0xffff,0xffff,255)):
                values = pool([32 if k != 32 else 30]) + [(0x5e5,0),(0x1a4b+k,0xffff),(0x1a93+k,y),
                          (0x1a27+k,fraction),(0x1a6f+k,fraction),(0x1ab7+k,vx),(0x1adb+k,vy)]
                updates.append(case(f'move-{k}-{y}-{vx}',values,k))
        for free in (-1,0,34):
            for wanted in (0,0x20,0x400,0x420):
                seed = next(s for s in range(65536) if next_random(s) & 0x420 == wanted)
                values = pool([free]) + [(0x5e5,seed),(0x1a4b+k,100),(0x1a93+k,100),
                          (0x1a27+k,0),(0x1a6f+k,0),(0x1ab7+k,0),(0x1adb+k,0)]
                preinstructions.append(case(f'mask-{k}-{free}-{wanted}',values,k,0xce9b))
    save('mother-brain-glass-motion',[
        dict(symbol='EprojPreInstr_MotherBrainGlassShatteringShard',args=1,**{'return':'void'},cases=updates),
        dict(symbol='EprojPreinstruction',args=2,**{'return':'void'},cases=preinstructions)])

    frames = []
    def frame_state(k,address,script,timer):
        return [(0x198d,0x8000),(0x1997+k,0xcefc if address==0xce9b else 0xcf0a),
                (0x1a03+k,address),(0x1b47+k,script),(0x1b8f+k,timer),
                (0x1a4b+k,100),(0x1a93+k,100),(0x5e5,1)]
    for k in (0,16,34):
        for script in (0xcc93,0xccb7,0xccdb,0xccff,0xcd23,0xcd47,0xcd6b,0xcd8f):
            for offset in range(0,36,4):
                for timer in (0,1,2,0xffff):
                    frames.append(case(f'shard-script-{k}-{script}-{offset}-{timer}',frame_state(k,0xce9b,script+offset,timer),fill=0))
        for offset in range(0,20,4):
            for timer in (0,1,2,0xffff):
                frames.append(case(f'sparkle-script-{k}-{offset}-{timer}',frame_state(k,0x84fb,0xcdb3+offset,timer),fill=0))
    for k in range(0,36,2):
        for free in (-1,0,16,34):
            # Native sparkle allocation above a running slot waits until the
            # next frame; lower slots execute their native animation immediately.
            values = pool([free])
            for other in range(0,36,2):
                values += [(0x1a03+other,0x8170),(0x1b8f+other,2),(0x1b47+other,0xcc93)]
            values += frame_state(k,0xce9b,0xcc93,1) + [(0x5e5,0)]
            frames.append(case(f'frame-spawn-{k}-{free}',values,fill=0))
        for y in (0,255,256,0xffff):
            values = frame_state(k,0xce9b,0xccb3,1)+[(0x1a93+k,y),(0x1adb+k,0x100)]
            frames.append(case(f'frame-cull-{k}-{y}',values,fill=0))
    save('mother-brain-glass-frames',[dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=frames)])

    callers, instructions = [], []
    for index in (0,40,78):
        for free_count in (0,1,2,3,4,18):
            for parameter in range(6):
                values = pool(list(range(0,free_count*2,2)))+plm(parameter,index,16,0xffff)
                callers.append(case(f'caller-{index}-{free_count}-{parameter}',values,parameter))
            for sound in (0,1,15):
                for pc in (0xd231,0xd267,0xd289,0xd297,0xd2ab):
                    values = pool(list(range(0,free_count*2,2)))+plm(0,index,16,33)
                    values += [(0x645,0),(0x648,sound),(0x5f5,0),(0x998,8),(0x592,0)]
                    instructions.append(case(f'plm-{index}-{free_count}-{sound}-{pc}',values,0x840000|pc,index))
    save('mother-brain-glass-callers',[
        dict(symbol='SpawnMotherBrainGlassShatteringShard',args=1,**{'return':'void'},cases=callers),
        dict(symbol='PlmInstr_SpawnFourMotherBrainGlass',arg_types=['rom_ptr','uint16'],
             **{'return':'rom_ptr'},pointer_bank=0x84,cases=instructions)])


def sequence(source_run, caller_run):
    """Capture actual four-shard PLM allocation through 128 native frames."""
    source_run, caller_run = source_run.resolve(), caller_run.resolve()
    output = ROOT / 'out/port/mother-brain-glass-sequence-inputs'
    output.mkdir(parents=True, exist_ok=True)
    initial = caller_run / 'plm-0-18-0-53897/c-after.wram'
    state = bytearray(initial.read_bytes())
    state[0x198d:0x198f] = (0x8000).to_bytes(2, 'little')
    host = output / 'host.bin'
    host.write_bytes((caller_run / 'plm-0-18-0-53897/c-host-after.bin').read_bytes())
    manifest = json.loads((ROOT / 'tools/port/mother-brain-glass-frames.json').read_text())
    manifest['family'] = 'mother-brain-glass-sequence'
    cases = []
    for frame in range(128):
        before, after = output / f'{frame:03d}-before.wram', output / f'{frame:03d}-after.wram'
        before.write_bytes(state)
        result = subprocess.run([str(source_run / 'c'), 'EprojRunAll', '0', '0', str(before),
                                 str(after), str(source_run / 'rom.bin'), str(host),
                                 str(output / 'host-after.bin')], check=True, capture_output=True, timeout=30)
        cases.append(dict(name=f'sequence-{frame:03d}', before_file=str(before),
                          before_sha256=hashlib.sha256(before.read_bytes()).hexdigest(),
                          expected_after_file=str(after),
                          expected_after_sha256=hashlib.sha256(after.read_bytes()).hexdigest(),
                          expected_return=int(result.stdout), host_words=[11,22,33,44,55,66,77,88]))
        state = bytearray(after.read_bytes())
        if (output / 'host-after.bin').read_bytes() != host.read_bytes():
            raise ValueError('glass sequence unexpectedly changed external host')
    manifest['routines'] = [manifest['routines'][0], dict(symbol='EprojRunAll',args=0,**{'return':'void'},cases=cases)]
    (ROOT / 'tools/port/mother-brain-glass-sequence.json').write_text(json.dumps(manifest,separators=(',',':')) + '\n')
    print(f'mother-brain-glass-sequence: {len(cases)+1} cases; terminal active projectiles '
          f'{sum(int.from_bytes(state[0x1997+k:0x1999+k],"little") != 0 for k in range(0,36,2))}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sequence-source',type=Path)
    parser.add_argument('--caller-source',type=Path)
    args = parser.parse_args()
    if args.sequence_source:
        if not args.caller_source:
            parser.error('--sequence-source requires --caller-source')
        sequence(args.sequence_source,args.caller_source)
    else:
        generate()
