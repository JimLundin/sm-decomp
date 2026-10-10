#!/usr/bin/env python3
"""Declared crawler examples for actual-C/full-WRAM comparison."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def build():
    routines=[]
    names=['StoneZoomer_Init','FireZoomer_Init','BigEyeBugs_Init','WreckedShipOrangeZoomer_Init','StoneZoomer_E67A','StoneZoomer_Main','WreckedShipOrangeZoomer_Main','FireZoomer_Func_1','FireZoomer_Func_2','FireZoomer_Func_3','FireZoomer_E8A5','WreckedShipOrangeZoomer_Func_2','sub_A3E168','EnemyFunc_C8AD']
    for name in names:
        cases=[]
        for n in range(56):
            k=[0,64,128,1984][n%4];e=0xf78+k;x=0x7800+k;writes=[]
            def w(a,v):writes.append([a,list((v&65535).to_bytes(2,'little'))])
            w(0xe54,k);w(0x7a5,16);w(e+2,128);w(e+4,0x9000);w(e+6,128);w(e+8,0x7000);w(e+10,8);w(e+12,8)
            w(e+0xe,n%4);w(e+0x1a,n%4);w(e+0x30,[128,256,0xff80,0xff00][n%4]);w(e+0x32,[128,0xff80,256,0xff00][n%4]);w(e+0x3c,n%8);w(e+0x3e,(n%6)*2)
            w(x+2,[0,0xc000,0x8000,0xffff][n%4]);w(x+4,[0,2,4,0xffff][n%4]);w(x+6,0xe7f2);w(x+8,n%5)
            state=0xe091 if name=='WreckedShipOrangeZoomer_Main' else [0xe6c8,0xe785,0xe7f2,0xe6c1][n%4]
            w(e+0x3a,state);w(0xaf6,[96,160][n%2]);w(0xafa,[160,96][n%2]);w(0x1840,30 if n%3==0 else 0);w(0x183e,20)
            if n%8==7 and name!='WreckedShipOrangeZoomer_Init':w(e+0x3c,255)
            terrain=n//8
            # Surround the record with solid, air or supported slope terrain.
            for row in range(6,10):
                for col in range(6,10):
                    idx=row*16+col
                    if terrain==6:
                        w(0x10002+2*idx,0xa000);writes.append([0x16402+idx,[15]])
                    elif terrain in [1,3]:w(0x10002+2*idx,0x8000)
                    elif terrain in [2,4,5]:
                        w(0x10002+2*idx,0x1000);writes.append([0x16402+idx,[5|([0,64,128][terrain-3] if terrain>=3 else 0)]])
            cases.append(dict(name=f'{name}-{n}',a=k,writes=writes,**({'minimum_callbacks':1} if terrain==6 and not (name=='FireZoomer_Func_2' and n%4==0) and name in ['FireZoomer_Func_1','FireZoomer_Func_2','FireZoomer_Func_3','WreckedShipOrangeZoomer_Func_2','sub_A3E168'] else {})))
        routines.append(dict(symbol=name,arg_types=[] if name.endswith(('_Init','_Main')) else ['uint16'],return_type='int32' if name=='FireZoomer_E8A5' else 'uint8' if name=='EnemyFunc_C8AD' else 'void',cases=cases))
    for r in routines:r['return']=r.pop('return_type');r['domain']='aligned enemy offset<0x800; speed index<32 or Stone/Fire sentinel255; animation index<6; bounded terrain; native shared collision and explicit PLM host'
    for name in ['Zoomer_Instr_SetPreinstr','WreckedShipOrangeZoomer_Func_1']:
        routines.append(dict(symbol=name,arg_types=['uint16','rom_ptr'],**{'return':'rom_ptr'},domain='aligned record; readable bank A3 script word, next pointer within bank',cases=[dict(name=f'{name}-{i}',a=i*64,b=0xa3e600+2*i,writes=[[0xf78+i*64+0x2e,[0xa3,0]]]) for i in range(8)]))
    return dict(family='enemy-crawler',rust_module='crates/port/src/enemy_crawler.rs',rom=dict(file='/home/jim/sm-decomp/rom/sm.sfc',sha256='12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72'),routines=routines)
if __name__=='__main__':(ROOT/'tools/port/enemy-crawler.json').write_text(json.dumps(build(),separators=(',',':'))+'\n')
