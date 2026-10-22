use crate::c_library::puts;
use crate::game_state::Game;
use crate::snes::dsp::{dsp_cycle, dsp_init, dsp_read, dsp_reset, dsp_saveload, dsp_write};
use crate::snes::spc::{spc_init, spc_reset, spc_runOpcode, spc_saveload};
use crate::tracing::getProcessorStateSpc;
pub use crate::types::{size_t, Apu, ApuRegisters, DspRegWriteHistory, Timer};
use ::core::ffi::{c_char, c_void};
use ::core::mem::size_of;
// Register serialization retains the forty bytes following legacy APU RAM.
const _: () = {
    assert!(core::mem::offset_of!(ApuRegisters, history) == 40);
    assert!(size_of::<ApuRegisters>() == 556);
};
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
static bootRom: [u8; 64] = [
    0xcd, 0xef, 0xbd, 0xe8, 0, 0xc6, 0x1d, 0xd0, 0xfc, 0x8f, 0xaa, 0xf4, 0x8f, 0xbb, 0xf5, 0x78,
    0xcc, 0xf4, 0xd0, 0xfb, 0x2f, 0x19, 0xeb, 0xf4, 0xd0, 0xfc, 0x7e, 0xf4, 0xd0, 0xb, 0xe4, 0xf5,
    0xcb, 0xf4, 0xd7, 0, 0xfc, 0xd0, 0xf3, 0xab, 0x1, 0x10, 0xef, 0x7e, 0xf4, 0x10, 0xeb, 0xba,
    0xf6, 0xda, 0, 0xba, 0xf4, 0xc4, 0xf4, 0xdd, 0x5d, 0xd0, 0xdb, 0x1f, 0, 0, 0xc0, 0xff,
];
pub unsafe fn apu_init() -> *mut Apu {
    let apu = Box::into_raw(Box::new(Apu {
        spc: core::ptr::null_mut(),
        dsp: core::ptr::null_mut(),
        ram: crate::apu_ram::ApuRam::new(),
        registers: ApuRegisters {
            romReadable: false,
            dspAdr: 0,
            snapshot_padding0: [0; 2],
            cycles: 0,
            inPorts: [0; 6],
            outPorts: [0; 4],
            timer: [Timer {
                cycles: 0,
                divider: 0,
                target: 0,
                counter: 0,
                enabled: false,
            }; 3],
            cpuCyclesLeft: 0,
            pad: [0; 6],
            history: DspRegWriteHistory {
                count: 0,
                addr: [0; 256],
                val: [0; 256],
            },
        },
    }));
    (*apu).spc = spc_init(apu);
    (*apu).dsp = dsp_init((*apu).ram.as_mut_ptr());
    apu
}
pub unsafe fn apu_reset(apu: *mut Apu) {
    (*apu).registers.romReadable = true;
    spc_reset((*apu).spc);
    dsp_reset((*apu).dsp);
    (*apu).ram.fill(0);
    set_fields! {[(*apu).registers], 0; dspAdr, cycles}
    (*apu).registers.inPorts.fill(0);
    (*apu).registers.outPorts.fill(0);
    for timer in &mut (*apu).registers.timer {
        set_fields! {[timer], 0; cycles, divider, target, counter}
        timer.enabled = false;
    }
    (*apu).registers.cpuCyclesLeft = 7;
    (*apu).registers.history.count = 0;
}
pub unsafe fn apu_saveload(apu: *mut Apu, io: &mut crate::snapshot_io::SnapshotIo<'_>) {
    crate::rtl::snapshot_result(io.state(&mut *apu));
    dsp_saveload((*apu).dsp, io);
    spc_saveload((*apu).spc, io);
}

pub unsafe fn apu_cycle(apu: *mut Apu, game: &mut Game) {
    if (*apu).registers.cpuCyclesLeft == 0 {
        if game.g_debug_apu_cycles {
            let mut line: [c_char; 80] = [0; 80];
            getProcessorStateSpc(apu, line.as_mut_ptr());
            puts(line.as_mut_ptr());
        }
        (*apu).registers.cpuCyclesLeft = spc_runOpcode((*apu).spc) as u8;
    }
    (*apu).registers.cpuCyclesLeft = (*apu).registers.cpuCyclesLeft.wrapping_sub(1);
    if (*apu).registers.cycles & 0x1f == 0 {
        dsp_cycle((*apu).dsp);
    }
    for i in 0..3 {
        let timer = &mut (*apu).registers.timer[i];
        if timer.cycles == 0 {
            timer.cycles = if i == 2 { 16 } else { 128 };
            if timer.enabled {
                timer.divider = timer.divider.wrapping_add(1);
                if timer.divider == timer.target {
                    timer.divider = 0;
                    timer.counter = timer.counter.wrapping_add(1) & 0xf;
                }
            }
        }
        timer.cycles = timer.cycles.wrapping_sub(1);
    }
    (*apu).registers.cycles = (*apu).registers.cycles.wrapping_add(1);
}
#[export_name = "apu_cpuRead"]
pub unsafe extern "C-unwind" fn apu_cpuRead(apu: *mut Apu, adr: u16) -> u8 {
    match adr {
        0xf0 | 0xf1 | 0xfa..=0xfc => return 0,
        0xf2 => return (*apu).registers.dspAdr,
        0xf3 => return dsp_read((*apu).dsp, (*apu).registers.dspAdr & 0x7f),
        0xf4..=0xf9 => return (*apu).registers.inPorts[usize::from(adr - 0xf4)],
        0xfd..=0xff => {
            let timer = &mut (*apu).registers.timer[usize::from(adr - 0xfd)];
            crate::update_place! { previous value = timer.counter => 0; }
            return value;
        }
        _ => {}
    }
    if (*apu).registers.romReadable && adr >= 0xffc0 {
        return bootRom[usize::from(adr - 0xffc0)];
    }
    (*apu).ram.read(adr)
}
fn record_dsp_write(history: &mut DspRegWriteHistory, address: u8, value: u8) {
    let index = history.count as i32;
    if index != 256 {
        history.count = (index + 1) as u32;
        history.addr[index as usize] = address;
        history.val[index as usize] = value;
    }
}

#[export_name = "apu_cpuWrite"]
pub unsafe extern "C-unwind" fn apu_cpuWrite(apu: *mut Apu, adr: u16, val: u8) {
    match adr {
        0xf1 => {
            for i in 0..3 {
                let enabled = val & (1 << i) != 0;
                let timer = &mut (*apu).registers.timer[i];
                if !timer.enabled && enabled {
                    set_fields! {[timer], 0; divider, counter}
                }
                timer.enabled = enabled;
            }
            if val & 0x10 != 0 {
                (*apu).registers.inPorts[0] = 0;
                (*apu).registers.inPorts[1] = 0;
            }
            if val & 0x20 != 0 {
                (*apu).registers.inPorts[2] = 0;
                (*apu).registers.inPorts[3] = 0;
            }
            (*apu).registers.romReadable = val & 0x80 != 0;
        }
        0xf2 => (*apu).registers.dspAdr = val,
        0xf3 => {
            record_dsp_write(&mut (*apu).registers.history, (*apu).registers.dspAdr, val);
            if ((*apu).registers.dspAdr as i32) < 0x80 {
                dsp_write((*apu).dsp, (*apu).registers.dspAdr, val);
            }
        }
        0xf4..=0xf7 => (*apu).registers.outPorts[usize::from(adr - 0xf4)] = val,
        0xf8..=0xf9 => (*apu).registers.inPorts[usize::from(adr - 0xf4)] = val,
        0xfa..=0xfc => (*apu).registers.timer[usize::from(adr - 0xfa)].target = val,
        _ => {}
    }
    (*apu).ram.write(adr, val);
}
