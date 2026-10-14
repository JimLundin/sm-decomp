use std::{fmt::Write, fs};

#[derive(Default)]
struct Case {
    name: String,
    a: u32,
    b: u32,
    fill: Option<u8>,
    predecessor: Option<String>,
    writes: Vec<(usize, Vec<u8>)>,
    result: Option<u32>,
}
impl Case {
    fn new(name: String, fill: u8) -> Self { Self { name, fill: Some(fill), ..Self::default() } }
    fn continuation(name: &str, before: &str) -> Self { Self { name: name.into(), predecessor: Some(before.into()), ..Self::default() } }
    fn word(&mut self, offset: usize, value: u16) { self.writes.push((offset,value.to_le_bytes().to_vec())); }
    fn byte(&mut self, offset: usize, value: u8) { self.writes.push((offset,vec![value])); }
    fn render(&self, text: &mut String) {
        writeln!(text,"\n[[routines.cases]]\nname = {:?}",self.name).unwrap();
        if self.a != 0 {writeln!(text,"a = {}",self.a).unwrap()}
        if self.b != 0 {writeln!(text,"b = {}",self.b).unwrap()}
        if let Some(f) = self.fill {writeln!(text,"fill = {f}").unwrap()}
        if let Some(c) = &self.predecessor {writeln!(text,"continue_from = {c:?}").unwrap()}
        if let Some(r) = self.result {writeln!(text,"expected_return = {r}").unwrap()}
        if !self.writes.is_empty() {
            writeln!(text,"writes = [").unwrap();
            for (offset,bytes) in &self.writes {
                writeln!(text,"  {{ offset = 0x{offset:x}, data = {bytes:?} }},").unwrap();
            }
            writeln!(text,"]").unwrap();
        }
    }
}
fn routine(out: &mut String, symbol: &str, args: usize, returns: &str, cases: &[Case], packed: bool) {
    writeln!(out,"\n[[routines]]\nsymbol = {symbol:?}\nargs = {args}\nreturn = {returns:?}").unwrap();
    if packed { writeln!(out,"arg_types = [\"uint32\", \"uint32\"]\npacked_u16_args = 4").unwrap(); }
    writeln!(out,"live_eligible = false\nlive_limitation = \"Offline full-WRAM/raw-return fixtures; live pause scheduling and hardware acceptance remain pending.\"").unwrap();
    for c in cases {c.render(out)}
}
fn reserve(name: String, max: u16, health: u16, phase: u8, timer: u16, frame: u16, cursor: u16) -> Case {
    let mut c=Case::new(name,0xa5);
    c.word(0x9d4,max); c.word(0x9d6,health); c.byte(0x5b5,phase);
    c.word(0x72f,timer); c.word(0x743,frame); c.word(0x590,cursor);
    c
}
fn main() {
    let mut out=String::from("# Bounded pause equipment renderer fixtures, original bank-82 ROM assets.\n# Every pair compares all 128 KiB WRAM and raw return; no masks or C stubs.\n# Direct DrawPauseScreenSpriteAnim uses the separate exact-three-argument gate;\n# selector cases below also execute its real C implementation as a dependency.\nfamily = \"pause-equipment-render\"\nbank = \"sm_82.c\"\nc_support = [\"sm_81.c\", \"sm_rtl.c\"]\nrust_module = \"crates/port/src/pause_equipment_render.rs\"\ndomain = \"Original ROM; bounded menu sprites with OAM cursor 0..511; used pause selector frames 0..13; all four category rows; reserve capacities/amounts 0..500 with amount <= capacity; rounding/flicker; equipped-items wireframe bits; seven-byte queue within 0xD0..0x330. All-WRAM/raw-return comparison. Live replay remains pending.\"\n\n[rom]\nfile = \"/home/jim/sm-decomp/rom/sm.sfc\"\nsha256 = \"12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72\"\n");
    let mut counts=Vec::new();
    let mut cases=Vec::new();
    for id in [0u16,0x1b,0x1f,0x20,0x21,0x27,0x2b,0x40,0x64] {
        for (i,cursor) in [0u16,4,28,31,508,511].into_iter().enumerate() {
            let x=[0u16,255,256,0xffff,0x8000,0x100][i];
            let y=[0u16,95,255,0xffff,0x8000,0x1ff][i];
            let attributes=[0u16,0x600,0xc00,0xffff,0x2400,0x100][i];
            let mut c=Case::new(format!("menu-id-{id:02x}-cursor-{cursor}"),[0,0xa5,0xff][i%3]);
            c.a=u32::from(id)|(u32::from(x)<<16); c.b=u32::from(y)|(u32::from(attributes)<<16);
            c.word(0x590,cursor);
            cases.push(c);
        }
    }
    counts.push(("DrawMenuSpritemap",cases.len()));
    routine(&mut out,"DrawMenuSpritemap",2,"void",&cases,true);

    let mut cases=Vec::new();
    for (category,rows) in [2u16,5,6,3].into_iter().enumerate() {
        for row in 0..rows {
            for (scenario,(timer,frame,cursor)) in [(0u16,13u16,508u16),(1,0,0),(2,7,31),(0x8000,4,511),(0x8001,13,28)].into_iter().enumerate() {
                let mut c=Case::new(format!("selector-category-{category}-row-{row}-timer-{timer:04x}"),[0,0xa5,0xff][scenario%3]);
                c.word(0x9d4,if category==0 {100}else{0});
                c.word(0x9a4,if category>=2 {0xffff}else{0});
                c.word(0x9a8,if category==1 {1}else{0});
                c.word(0x755,(row<<8)|category as u16);
                c.word(0x72d,timer); c.word(0x741,frame); c.word(0x590,cursor);
                cases.push(c);
            }
        }
    }
    for fill in [0,0xa5,0xff] {
        let mut c=Case::new(format!("selector-empty-fill-{fill}"),fill);
        c.word(0x9d4,0); c.word(0x9a4,0); c.word(0x9a8,0);
        c.word(0x755,0xffff); c.word(0x72d,0xffff); c.word(0x741,0xffff); c.word(0x590,0xffff);
        cases.push(c);
    }
    let mut start=Case::new("chain-selector-0".into(),0xa5);
    start.word(0x9d4,400); start.word(0x9d6,199); start.word(0x9a4,0xffff); start.word(0x9a8,0xffff);
    start.word(0x755,0x0401); start.word(0x72d,1); start.word(0x741,13); start.word(0x72f,1); start.word(0x743,13);
    start.word(0x590,508); start.word(0x9a2,0x0101); start.word(0x330,7);
    cases.push(start);
    for index in 1..4 {cases.push(Case::continuation(&format!("chain-selector-{index}"),&format!("chain-selector-{}",index-1)));}
    counts.push(("EquipmentScreenDrawItemSelector",cases.len()));
    routine(&mut out,"EquipmentScreenDrawItemSelector",0,"void",&cases,false);

    let mut cases=Vec::new();
    for (maximum,health) in [(0u16,0u16),(100,0),(100,1),(100,13),(100,14),(100,15),(100,55),(100,56),(100,57),(100,98),(100,99),(100,100),(200,101),(200,114),(200,199),(200,200),(400,0),(400,250),(400,399),(400,400),(500,500),(350,349)] {
        for phase in [0,4] {
            let cursor=if phase==0 {508}else{31};
            let c=reserve(format!("reserve-max-{maximum}-health-{health}-phase-{phase}"),maximum,health,phase,1,13,cursor);
            cases.push(c);
        }
    }
    cases.push(Case::continuation("chain-reserve-0","chain-selector-3"));
    for index in 1..4 {cases.push(Case::continuation(&format!("chain-reserve-{index}"),&format!("chain-reserve-{}",index-1)));}
    counts.push(("EquipmentScreenDisplayReserveTankAmount",cases.len()));
    routine(&mut out,"EquipmentScreenDisplayReserveTankAmount",0,"void",&cases,false);

    let cases:Vec<_>=[(0u16,0u16),(100,0),(100,14),(100,99),(400,100),(400,199),(400,400),(500,499)].into_iter().enumerate().map(|(i,(max,health))| reserve(format!("reserve-helper-{i}"),max,health,(i as u8&1)*4,2,7,[0,511][i%2])).collect();
    counts.push(("EquipmentScreenDisplayReserveTankAmount_",cases.len()));
    routine(&mut out,"EquipmentScreenDisplayReserveTankAmount_",0,"void",&cases,false);

    let mut cases=Vec::new();
    for health in [0u16,1] {
        for timer in [0u16,1,2,0x8000,0x8001,0xffff] {
            for frame in [0u16,13] {
                let mut c=Case::new(format!("palette-health-{health}-timer-{timer:04x}-frame-{frame}"),0xa5);
                c.word(0x9d6,health); c.word(0x72f,timer); c.word(0x743,frame);
                c.result=Some(if health==0 {0x600}else{0xc00}); cases.push(c);
            }
        }
    }
    counts.push(("EquipmentScreenDisplayReserves_PaletteSetup",cases.len()));
    routine(&mut out,"EquipmentScreenDisplayReserves_PaletteSetup",0,"uint16",&cases,false);

    let mut cases=Vec::new();
    for bits in [0u16,0x100,1,0x101,0xfefe,0xfffe,0xfeff,0xffff] {
        let mut c=Case::new(format!("wireframe-items-{bits:04x}"),0xa5); c.word(0x9a2,bits); cases.push(c);
    }
    counts.push(("WriteSamusWireframeTilemap",cases.len()));
    routine(&mut out,"WriteSamusWireframeTilemap",0,"void",&cases,false);

    let mut cases=Vec::new();
    for bits in [0u16,0x100,1,0x101] {
        for tail in [0u16,7,593,601] {
            let mut c=Case::new(format!("wireframe-queue-items-{bits:04x}-tail-{tail}"),0xa5);
            c.word(0x9a2,bits); c.word(0x330,tail); cases.push(c);
        }
    }
    cases.push(Case::continuation("chain-wireframe-0","chain-reserve-3"));
    cases.push(Case::continuation("chain-wireframe-1","chain-wireframe-0"));
    counts.push(("WriteSamusWireframeTilemapAndQueue",cases.len()));
    routine(&mut out,"WriteSamusWireframeTilemapAndQueue",0,"void",&cases,false);
    fs::write("/tmp/sm-decomp-parallel-pause-equipment-render-offline-1/tools/port/pause-equipment-render.toml",out).unwrap();
    println!("purposeful fixture pairs: {}",counts.iter().map(|(_,n)|n).sum::<usize>());
    for (symbol,count) in counts {println!("{symbol}: {count}")}
}
