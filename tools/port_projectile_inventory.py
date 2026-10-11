#!/usr/bin/env python3
"""Bounded actual-C projectile execution inventory; diagnostic, never admission."""
import argparse
import datetime
import json
import subprocess
import shutil
from pathlib import Path
import port_family

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames', type=int, default=129712)
    parser.add_argument('--output', type=Path, default=ROOT / 'out/port/projectile-inventory')
    args = parser.parse_args()
    if not 1 <= args.frames <= 129712:
        parser.error('frames must be in1..129712')
    run = args.output.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
    run.mkdir(parents=True)
    source = ROOT / 'deps/snesrev-sm/src'
    source_identity = port_family.verify_source(source.parent)
    original = (source / 'sm_86.c').read_text()
    support = '''
#include <stdio.h>
static unsigned long inventory_hits[4][65536];
static int inventory_first[4][65536], inventory_last[4][65536];
static void InventoryHit(unsigned kind,uint16 address) {
 if(!inventory_hits[kind][address])inventory_first[kind][address]=snes_frame_counter;
 inventory_hits[kind][address]++;inventory_last[kind][address]=snes_frame_counter;
}
void InventorySave(void) {
 FILE *f=fopen("projectiles.tsv","w");if(!f)exit(2);
 fprintf(f,"kind\\taddress\\thits\\tfirst_frame\\tlast_frame\\n");
 const char *names[]={"id","init","pre","instruction"};
 for(unsigned kind=0;kind<4;kind++)for(unsigned i=0;i<65536;i++)if(inventory_hits[kind][i])
 fprintf(f,"%s\\t%04x\\t%lu\\t%d\\t%d\\n",names[kind],i,inventory_hits[kind][i],inventory_first[kind][i],inventory_last[kind][i]);
 if(fclose(f))exit(2);
}
'''
    # Definitions are exact replacement anchors, never declarations or callers.
    anchors = [('static void EprojRunOne(uint16 k) {', 'InventoryHit(0,eproj_id[k>>1]);'),
               ('static void CallEprojInit(uint32 ea, uint16 j) {', 'InventoryHit(1,(uint16)ea);'),
               ('static void CallEprojPreInstr(uint32 ea, uint16 k) {', 'InventoryHit(2,(uint16)ea);'),
               ('static const uint8 *CallEprojInstr(uint32 ea, uint16 k, const uint8 *j) {', 'InventoryHit(3,(uint16)ea);')]
    transformed = original
    for anchor, statement in anchors:
        assert transformed.count(anchor) == 1, anchor
        transformed = transformed.replace(anchor, anchor + '\n  ' + statement)
    # Prefix support after the include block, before first function.
    point = transformed.index('static void CallEprojPreInstr')
    transformed = transformed[:point] + support + transformed[point:]
    (run / 'sm_86.c').write_text(transformed)
    host = (ROOT / 'tools/decomp/headless.c').read_text()
    start = host.index('static void SaveRoutineState')
    end = host.index('// Linker wrapping', start)
    host = host[:start] + host[end:]
    start = host.index('void __wrap_DecompressToMem')
    end = host.index('// The host has no audio thread', start)
    host = host[:start] + '''void __wrap_DecompressToMem(uint32 src,uint8 *dst) {
 decompression_calls++;__real_DecompressToMem(src,dst);
}
extern void InventorySave(void);
''' + host[end:]
    host = host.replace('  fclose(inputs);\n  return 0;', '  fclose(inputs);\n  InventorySave();\n  return 0;')
    (run / 'headless.c').write_text(host)
    (run / 'headless_frame.h').write_bytes((ROOT / 'tools/decomp/headless_frame.h').read_bytes())
    units = [run / 'sm_86.c' if p.name == 'sm_86.c' else p for p in sorted(source.glob('sm_*.c'))]
    units += [source / n for n in ('spc_player.c', 'tracing.c', 'util.c')]
    units += sorted((source / 'snes').glob('*.c'))
    originals = run / 'original-source'
    shutil.copytree(source, originals)
    command = ['cc', '-O2', '-fno-strict-aliasing', '-ffunction-sections', '-fdata-sections', '-iquote', str(source), *map(str, units), str(run / 'headless.c'), '-Wl,--gc-sections', '-Wl,--wrap=Call', '-Wl,--wrap=cpu_runOpcode', '-Wl,--wrap=DecompressToMem', '-lm', '-o', str(run / 'inventory')]
    port_family.execute(command, run, 'build')
    rom = ROOT / 'rom/sm.sfc'
    inputs = ROOT / 'movies/super-metroid-any-3653.inputs'
    replay = [str(run / 'inventory'), str(rom), str(inputs), str(args.frames), str(run / 'terminal.bin')]
    with (run / 'commands.jsonl').open('a') as log:
        log.write(json.dumps(replay)+'\n')
    try:
        with (run / 'replay.stdout').open('wb') as stdout, (run / 'replay.stderr').open('wb') as stderr:
            subprocess.run(replay, cwd=run, stdout=stdout, stderr=stderr, timeout=300, check=True)
    except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
        failure = dict(status='diagnostic-failed', error=str(error), frames=args.frames,
                       source=source_identity, tool_sha256=port_family.digest(Path(__file__)),
                       artifacts={p.name: port_family.digest(p) for p in run.iterdir() if p.is_file()})
        (run / 'verdict.json').write_text(json.dumps(failure, indent=2)+'\n')
        raise
    # Includes original source identity and every transformed/observed small artifact.
    artifacts = {str(p.relative_to(run)): port_family.digest(p) for p in sorted(run.rglob('*')) if p.is_file()}
    verdict = dict(status='diagnostic-complete', frames=args.frames, live_exercised=False,
                   scope='fresh C trajectory; counters only; no Rust replacement/parity claim',
                   source=source_identity,
                   original_sm86_sha256=port_family.digest(source / 'sm_86.c'),
                   tool_sha256=port_family.digest(Path(__file__)),
                   rom_sha256=port_family.digest(rom), inputs_sha256=port_family.digest(inputs), artifacts=artifacts)
    (run / 'verdict.json').write_text(json.dumps(verdict, indent=2)+'\n')
    print(json.dumps(dict(status=verdict['status'], run=str(run))))


if __name__ == '__main__':
    main()
