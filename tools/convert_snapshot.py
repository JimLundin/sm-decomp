#!/usr/bin/env python3
"""Convert the reviewed original Snes9x v17 Golden observation to the existing seed format."""
import argparse
import csv
import gzip
import hashlib
import json
import struct
from pathlib import Path

# Serialized v1430 fields from official snes9x-rr snapshot.cpp at
# 3a0e01872a92049789a5b3200633d159f1a373aa, independently matched to v17 descriptors.
LAYOUT = {
    'PPU': (2641, [
        ('BGMode', 0, 1, 1),
        ('BG3Priority', 1, 1, 1),
        ('Brightness', 2, 1, 1),
        ('VMA.High', 3, 1, 1),
        ('VMA.Increment', 4, 1, 1),
        ('VMA.Address', 5, 2, 1),
        ('VMA.FullGraphicCount', 9, 2, 1),
        ('BG[0].SCBase', 13, 2, 1),
        ('BG[0].VOffset', 15, 2, 1),
        ('BG[0].HOffset', 17, 2, 1),
        ('BG[0].BGSize', 19, 1, 1),
        ('BG[0].NameBase', 20, 2, 1),
        ('BG[0].SCSize', 22, 2, 1),
        ('BG[1].SCBase', 24, 2, 1),
        ('BG[1].VOffset', 26, 2, 1),
        ('BG[1].HOffset', 28, 2, 1),
        ('BG[1].BGSize', 30, 1, 1),
        ('BG[1].NameBase', 31, 2, 1),
        ('BG[1].SCSize', 33, 2, 1),
        ('BG[2].SCBase', 35, 2, 1),
        ('BG[2].VOffset', 37, 2, 1),
        ('BG[2].HOffset', 39, 2, 1),
        ('BG[2].BGSize', 41, 1, 1),
        ('BG[2].NameBase', 42, 2, 1),
        ('BG[2].SCSize', 44, 2, 1),
        ('BG[3].SCBase', 46, 2, 1),
        ('BG[3].VOffset', 48, 2, 1),
        ('BG[3].HOffset', 50, 2, 1),
        ('BG[3].BGSize', 52, 1, 1),
        ('BG[3].NameBase', 53, 2, 1),
        ('BG[3].SCSize', 55, 2, 1),
        ('CGFLIP', 57, 1, 1),
        ('CGDATA', 58, 2, 256),
        ('OAMPriorityRotation', 1979, 1, 1),
        ('OAMAddr', 1980, 2, 1),
        ('OAMFlip', 1982, 1, 1),
        ('VBeamPosLatched', 1989, 2, 1),
        ('HBeamPosLatched', 1991, 2, 1),
        ('HBeamFlip', 1993, 1, 1),
        ('VBeamFlip', 1994, 1, 1),
        ('HVBeamCounterLatched', 1995, 1, 1),
        ('MatrixA', 1996, 2, 1),
        ('MatrixB', 1998, 2, 1),
        ('MatrixC', 2000, 2, 1),
        ('MatrixD', 2002, 2, 1),
        ('CentreX', 2004, 2, 1),
        ('CentreY', 2006, 2, 1),
        ('CGADD', 2011, 1, 1),
        ('FixedColourRed', 2012, 1, 1),
        ('FixedColourGreen', 2013, 1, 1),
        ('FixedColourBlue', 2014, 1, 1),
        ('SavedOAMAddr', 2015, 2, 1),
        ('ScreenHeight', 2017, 2, 1),
        ('ForcedBlanking', 2023, 1, 1),
        ('OBJNameSelect', 2024, 2, 1),
        ('OBJSizeSelect', 2026, 1, 1),
        ('OBJNameBase', 2027, 2, 1),
        ('Mosaic', 2034, 1, 1),
        ('Mode7HFlip', 2035, 1, 1),
        ('Mode7VFlip', 2036, 1, 1),
        ('Mode7Repeat', 2037, 1, 1),
        ('Window1Left', 2038, 1, 1),
        ('Window1Right', 2039, 1, 1),
        ('Window2Left', 2040, 1, 1),
        ('Window2Right', 2041, 1, 1),
        ('ClipWindowOverlapLogic[0]', 2042, 1, 1),
        ('ClipWindow1Enable[0]', 2043, 1, 1),
        ('ClipWindow2Enable[0]', 2044, 1, 1),
        ('ClipWindow1Inside[0]', 2045, 1, 1),
        ('ClipWindow2Inside[0]', 2046, 1, 1),
        ('ClipWindowOverlapLogic[1]', 2047, 1, 1),
        ('ClipWindow1Enable[1]', 2048, 1, 1),
        ('ClipWindow2Enable[1]', 2049, 1, 1),
        ('ClipWindow1Inside[1]', 2050, 1, 1),
        ('ClipWindow2Inside[1]', 2051, 1, 1),
        ('ClipWindowOverlapLogic[2]', 2052, 1, 1),
        ('ClipWindow1Enable[2]', 2053, 1, 1),
        ('ClipWindow2Enable[2]', 2054, 1, 1),
        ('ClipWindow1Inside[2]', 2055, 1, 1),
        ('ClipWindow2Inside[2]', 2056, 1, 1),
        ('ClipWindowOverlapLogic[3]', 2057, 1, 1),
        ('ClipWindow1Enable[3]', 2058, 1, 1),
        ('ClipWindow2Enable[3]', 2059, 1, 1),
        ('ClipWindow1Inside[3]', 2060, 1, 1),
        ('ClipWindow2Inside[3]', 2061, 1, 1),
        ('ClipWindowOverlapLogic[4]', 2062, 1, 1),
        ('ClipWindow1Enable[4]', 2063, 1, 1),
        ('ClipWindow2Enable[4]', 2064, 1, 1),
        ('ClipWindow1Inside[4]', 2065, 1, 1),
        ('ClipWindow2Inside[4]', 2066, 1, 1),
        ('ClipWindowOverlapLogic[5]', 2067, 1, 1),
        ('ClipWindow1Enable[5]', 2068, 1, 1),
        ('ClipWindow2Enable[5]', 2069, 1, 1),
        ('ClipWindow1Inside[5]', 2070, 1, 1),
        ('ClipWindow2Inside[5]', 2071, 1, 1),
        ('BGMosaic', 2074, 1, 4),
        ('OAMData', 2078, 1, 544),
        ('OAMWriteRegister', 2624, 2, 1),
        ('OpenBus1', 2627, 1, 1),
        ('OpenBus2', 2628, 1, 1),
        ('RangeTimeOver', 2640, 1, 1),
    ]),
    'CPU': (68, [
        ('Flags', 0, 4, 1),
        ('NMIActive', 5, 1, 1),
        ('IRQActive', 6, 1, 1),
        ('WaitingForInterrupt', 7, 1, 1),
        ('Cycles', 9, 4, 1),
        ('V_Counter', 17, 4, 1),
        ('InDMA', 38, 1, 1),
    ]),
    'REG': (16, [
        ('PB', 0, 1, 1),
        ('DB', 1, 1, 1),
        ('P.W', 2, 2, 1),
        ('A.W', 4, 2, 1),
        ('D.W', 6, 2, 1),
        ('S.W', 8, 2, 1),
        ('X.W', 10, 2, 1),
        ('Y.W', 12, 2, 1),
        ('PCw', 14, 2, 1),
    ]),
    'GBJ': (112, [
        ('OpenBus', 33, 1, 1),
    ]),
}

def snapshot_blocks(path):
    data = gzip.decompress(path.read_bytes())
    header, rest = data.split(b'\n', 1)
    if header != b'#!snes9x:1430':
        raise ValueError(f'unsupported snapshot header {header!r}')
    at, blocks = 0, {}
    while at < len(rest):
        chunk = rest[at:at + 11]
        if len(chunk) != 11 or chunk[3:4] != b':' or chunk[10:] != b':':
            raise ValueError('invalid snapshot block header')
        name = chunk[:3].decode('ascii')
        size = int.from_bytes(chunk[6:10], 'big') if chunk[4:6] == b'--' else int(chunk[4:10])
        at += 11
        if name in blocks or at + size > len(rest):
            raise ValueError('duplicate or truncated snapshot block')
        blocks[name] = rest[at:at + size]
        at += size
    for name, size in {'RAM': 131072, 'SRA': 131072, 'VRA': 65536, 'ARA': 65536, 'FIL': 32768}.items():
        if len(blocks[name]) != size:
            raise ValueError(f'wrong {name} size')
    return blocks


def decode(name, data):
    if name == 'DMA':
        if len(data) != 136:
            raise ValueError('unsupported DMA snapshot size')
        fields = ('TransferDirection', 'AAddressFixed', 'AAddressDecrement', 'TransferMode',
                  'ABank', 'AAddress', 'Address', 'BAddress', 'HDMAIndirectAddressing',
                  'IndirectAddress', 'IndirectBank', 'Repeat', 'LineCount', 'FirstLine')
        return {f'{channel}.{field}': value for channel in range(8)
                for field, value in zip(fields, struct.unpack_from('>5B2H2BH4B', data, channel * 17))}
    size, fields = LAYOUT[name]
    if len(data) != size:
        raise ValueError('unsupported ' + name + ' snapshot size')
    result = {}
    for field, offset, unit, count in fields:
        values = [int.from_bytes(data[offset + i * unit:offset + (i + 1) * unit], 'big')
                  for i in range(count)]
        result[field] = values[0] if count == 1 else values
    return result


def registers(blocks,decoded,live,history):
 p,c,u=decoded['PPU'],decoded['CPU'],decoded['REG'];ram=blocks['RAM'];fill=blocks['FIL']
 word=lambda a:int.from_bytes(ram[a:a+2],'little')
 frame=live['movie_frame'];assert history['movie_frame']==frame
 assert history['last_m7_port']==0x2120 and history['last_m7_pc']==0x809339
 assert history['m7_latch']==p['CentreY']>>8
 assert history['cgram_low']==(p['CGDATA'][(p['CGADD']-1)&255]&255) and p['CGFLIP']==0
 assert c['NMIActive']==0 and c['InDMA']==0 and c['Flags']==0
 assert c['IRQActive']==0 and c['WaitingForInterrupt']==0
 out={'seed_movie_frame':frame,'seed_room':word(0x79b),'seed_nmi':word(0x5b6),
      'source_emulator_snes9x_rr_version':17,'source_snapshot_version':1430,
      'a':u['A.W'],'x':u['X.W'],'y':u['Y.W'],'s':u['S.W'],'d':u['D.W'],
      'db':u['DB'],'pb':u['PB'],'pc':u['PCw'],'pbpc':u['PB']<<16|u['PCw'],
      'p':u['P.W']&255,'e':(u['P.W']>>8)&1,'irq':c['IRQActive'],'wai':c['WaitingForInterrupt'],
      'hcounter':c['Cycles'],'vcounter':c['V_Counter'],'mdr':decoded['GBJ']['OpenBus']}
 assert out['pbpc']==history['pc'] and out['pbpc']!=0x808349
 # No bsnes opcode-fetch or normal-wait metadata: this is the actual v17 frame boundary.
 for address in (0x4200,0x4201,0x4207,0x4208,0x4209,0x420a,0x420d):out[f'io_{address:04X}']=fill[address]
 direct={'display_brightness':'Brightness','bg_mode':'BGMode','vram_incsize':'VMA.Increment',
  'vram_addr':'VMA.Address','display_disabled':'ForcedBlanking','bg3_priority':'BG3Priority',
  'vram_incmode':'VMA.High','oam_basesize':'OBJSizeSelect','oam_priority':'OAMPriorityRotation',
  'oam_tdaddr':'OBJNameBase','oam_baseaddr':'SavedOAMAddr','mode7_hflip':'Mode7HFlip',
  'mode7_vflip':'Mode7VFlip','mode7_repeat':'Mode7Repeat','window1_left':'Window1Left',
  'window1_right':'Window1Right','window2_left':'Window2Left','window2_right':'Window2Right',
  'color_r':'FixedColourRed','color_g':'FixedColourGreen','color_b':'FixedColourBlue',
  'ppu1_mdr':'OpenBus1','ppu2_mdr':'OpenBus2','hcounter':'HBeamPosLatched','vcounter':'VBeamPosLatched',
  'latch_hcounter':'HBeamFlip','latch_vcounter':'VBeamFlip','counters_latched':'HVBeamCounterLatched'}
 for name,field in direct.items():out['ppu_'+name]=p[field]
 for name,field in [('m7a','MatrixA'),('m7b','MatrixB'),('m7c','MatrixC'),('m7d','MatrixD')]:out['ppu_'+name]=p[field]
 for name,field in [('m7x','CentreX'),('m7y','CentreY'),('m7_hofs','BG[0].HOffset'),('m7_vofs','BG[0].VOffset')]:out['ppu_'+name]=p[field]&0x1fff
 out.update({'ppu_vram_mapping':(fill[0x2115]>>2)&3,'ppu_vram_readbuffer':live['state']['VRAMReadBuffer'],
  'ppu_m7_latch':history['m7_latch'],'ppu_cgram_latchdata':history['cgram_low'],
  'ppu_cgram_addr':p['CGADD']*2+(p['CGFLIP']&1),'ppu_oam_latchdata':p['OAMWriteRegister']&255,
  'ppu_oam_addr':p['OAMAddr']*2+(p['OAMFlip']&1),'ppu_oam_nameselect':p['OBJNameSelect']>>13,
  'ppu_mosaic_size':p['Mosaic']-1,'ppu_direct_color':fill[0x2130]&1,'ppu_addsub_mode':(fill[0x2130]>>1)&1,
  'ppu_color_mask':fill[0x2130]>>6,'ppu_colorsub_mask':(fill[0x2130]>>4)&3,
  'ppu_color_halve':(fill[0x2131]>>6)&1,'ppu_color_mode':fill[0x2131]>>7,
  'ppu_interlace':fill[0x2133]&1,'ppu_oam_interlace':(fill[0x2133]>>1)&1,
  'ppu_overscan':(fill[0x2133]>>2)&1,'ppu_pseudo_hires':(fill[0x2133]>>3)&1,
  'ppu_mode7_extbg':(fill[0x2133]>>6)&1,'ppu_time_over':(p['RangeTimeOver']>>7)&1,
  'ppu_range_over':(p['RangeTimeOver']>>6)&1})
 assert p['ScreenHeight']==(239 if out['ppu_overscan'] else 224)
 assert p['VMA.FullGraphicCount']==[0,32,64,128][out['ppu_vram_mapping']]
 for i in range(4):
  # v17 stores whole shared-latch words; target hardware scroll fields are 10 bits.
  out[f'ppu_bg_hofs[{i}]']=p[f'BG[{i}].HOffset']&0x3ff
  out[f'ppu_bg_vofs[{i}]']=p[f'BG[{i}].VOffset']&0x3ff
  out[f'ppu_bg_tilesize[{i}]']=p[f'BG[{i}].BGSize'];out[f'ppu_mosaic_enabled[{i}]']=p['BGMosaic'][i]
  out[f'ppu_bg_scaddr[{i}]']=p[f'BG[{i}].SCBase']*2;out[f'ppu_bg_tdaddr[{i}]']=p[f'BG[{i}].NameBase']*2
  out[f'ppu_bg_scsize[{i}]']=p[f'BG[{i}].SCSize']
  assert fill[0x2107+i]&0x80==0,'unrepresented tilemap address bit'
  assert ((fill[0x210b+i//2]>>(4*(i%2)))&8)==0,'unrepresented tile address bit'
 for i in range(5):
  for name,address in [('bg_enabled',0x212c),('bgsub_enabled',0x212d),('window_enabled',0x212e),('sub_window_enabled',0x212f)]:out[f'ppu_{name}[{i}]']=(fill[address]>>i)&1
 for i in range(6):
  for name,field in [('window1_enabled','ClipWindow1Enable'),('window2_enabled','ClipWindow2Enable'),('window_mask','ClipWindowOverlapLogic')]:out[f'ppu_{name}[{i}]']=p[f'{field}[{i}]']
  out[f'ppu_window1_invert[{i}]']=1-p[f'ClipWindow1Inside[{i}]'];out[f'ppu_window2_invert[{i}]']=1-p[f'ClipWindow2Inside[{i}]']
  out[f'ppu_color_enabled[{i}]']=(fill[0x2131]>>i)&1
 for ch in range(8):
  d=lambda name:decoded['DMA'][f'{ch}.{name}']
  control=(d('TransferDirection')<<7)|(d('HDMAIndirectAddressing')<<6)|(fill[0x4300+16*ch]&0x20)|(d('AAddressDecrement')<<4)|(d('AAddressFixed')<<3)|d('TransferMode')
  assert control==fill[0x4300+16*ch]
  data=[control,d('BAddress'),d('AAddress')&255,d('AAddress')>>8,d('ABank'),d('IndirectAddress')&255,d('IndirectAddress')>>8,d('IndirectBank'),d('Address')&255,d('Address')>>8,d('LineCount')|((1-d('Repeat'))<<7)]
  for offset,value in enumerate(data):out[f'dma_{ch}_{offset}']=value
 return out

def convert(snapshot, live_path, history_path, rom, output):
    live = json.loads(live_path.read_text())
    proof = json.loads(history_path.read_text())
    if (hashlib.sha256(rom.read_bytes()).hexdigest() != '12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72'
            or hashlib.sha256(snapshot.read_bytes()).hexdigest() != 'd034c893a276238becf16c40642c65d545726c0a0586735cbca2cf875ce8af05'
            or live['snapshot_sha256'] != hashlib.sha256(snapshot.read_bytes()).hexdigest()
            or live['movie_frame'] != 186800 or live['state']['VRAMReadBuffer'] != 11279
            or proof['status'] != 'PASS for original frame186800 PPU latch translation only'
            or proof['trace_sha256'] != 'b83eb5176da697aeeac9efea628481fabd9a413d3c5bef00fc983cac663eb8d4'
            or proof['last_m7'] != {'line': 18288, 'pc': 0x809336, 'port': 0x2120, 'value': 0}
            or proof['last_cgram'] != {'line': 18039, 'pc': 0x809372, 'frame': 186800, 'low': 0}
            or proof['m7_latch'] != 0 or proof['cgram_latchdata'] != 0):
        raise ValueError('unqualified original observation or missing latch history')
    blocks = snapshot_blocks(snapshot)
    decoded = {name: decode(name, blocks[name]) for name in (*LAYOUT, 'DMA')}
    history = dict(movie_frame=186800, pc=0x8dc535, last_m7_port=0x2120,
                   last_m7_pc=0x809339, m7_latch=proof['m7_latch'], cgram_low=proof['cgram_latchdata'])
    regs = registers(blocks, decoded, live, history)
    regions = {'wram': blocks['RAM'], 'sram': blocks['SRA'][:8192], 'vram': blocks['VRA'],
               'cgram': b''.join(value.to_bytes(2, 'little') for value in decoded['PPU']['CGDATA']),
               'oam': bytes(decoded['PPU']['OAMData']), 'apuram': blocks['ARA']}
    text = 'name,value\n' + ''.join(f'{name},{value}\n' for name, value in sorted(regs.items()))
    digest = hashlib.sha256(b''.join(regions.values()) + text.encode()).hexdigest()
    if digest != '3d65a01ecba9dc550449d70db11d6172211ab4fc728334ae9ccad93a6b5d226d':
        raise ValueError('converted observation differs from reviewed seed')
    output.mkdir(parents=True, exist_ok=False)
    rows = []
    for name, data in regions.items():
        filename = f'0001-B283-{name}.bin'
        (output / filename).write_bytes(data)
        rows.append([1, 'B283', 186800, name, len(data), hashlib.sha256(data).hexdigest(), filename])
    (output / '0001-B283-registers.csv').write_bytes(text.encode())
    with (output / 'rooms.csv').open('w', newline='') as file:
        writer = csv.writer(file, lineterminator='\n')
        writer.writerow(['room_index', 'room', 'movie_frame', 'region', 'size', 'sha256', 'file'])
        writer.writerows(rows)
    return digest


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('snapshot', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--live', type=Path, required=True)
    parser.add_argument('--history', type=Path, required=True)
    parser.add_argument('--rom', type=Path, required=True)
    args = parser.parse_args()
    print(convert(args.snapshot, args.live, args.history, args.rom, args.output))
