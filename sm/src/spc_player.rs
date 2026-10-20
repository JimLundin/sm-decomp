use crate::c_library::{__assert_fail, malloc, memcpy, memset, printf};
use crate::set_fields;
use crate::snes::dsp::{dsp_cycle, dsp_init, dsp_reset, dsp_write};
pub use crate::types::{
    size_t, Channel, Dsp, DspChannel, DspRegWriteHistory, SfxChan, SfxRegs, SpcPlayer,
};
use ::core::ffi::{c_char, c_void};
use ::core::mem::size_of;
use ::core::ptr::null_mut;
pub const EVOLL: DspReg = 44;
pub const EVOLR: DspReg = 60;
pub const EFB: DspReg = 13;
pub const EON: DspReg = 77;
pub const FLG: DspReg = 108;
pub const KON: DspReg = 76;
pub const KOF: DspReg = 92;
pub const NON: DspReg = 61;
pub const PMON: DspReg = 45;
pub const V0PITCHH: DspReg = 3;
pub const V0PITCHL: DspReg = 2;
pub const V0VOLL: DspReg = 0;
pub const V0GAIN: DspReg = 7;
pub const V0ADSR2: DspReg = 6;
pub const V0ADSR1: DspReg = 5;
pub const V0SRCN: DspReg = 4;
pub const FIR0: DspReg = 15;
pub const ESA: DspReg = 109;
pub const EDL: DspReg = 125;
pub const DIR: DspReg = 93;
pub const MVOLR: DspReg = 28;
pub const MVOLL: DspReg = 12;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct MemMapSized {
        pub off, org_off, size: u16;
    }
}
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct MemMap {
        pub off, org_off: u16;
    }
}

impl MemMapSized {
    const fn new(native_offset: u16, spc_offset: u16, byte_count: u16) -> Self {
        Self {
            off: native_offset,
            org_off: spc_offset,
            size: byte_count,
        }
    }
}

impl MemMap {
    // The high bit of encoded_spc_offset selects a two-byte copy.
    const fn new(native_offset: u16, encoded_spc_offset: u16) -> Self {
        Self {
            off: native_offset,
            org_off: encoded_spc_offset,
        }
    }
}

pub use crate::types::DspReg;
pub const FIR7: DspReg = 127;
pub const ENDX: DspReg = 124;
pub const V7OUTX: DspReg = 121;
pub const V7ENVX: DspReg = 120;
pub const V7GAIN: DspReg = 119;
pub const V7ADSR2: DspReg = 118;
pub const V7ADSR1: DspReg = 117;
pub const V7SRCN: DspReg = 116;
pub const V7PH: DspReg = 115;
pub const V7PL: DspReg = 114;
pub const V7VOLR: DspReg = 113;
pub const V7VOLL: DspReg = 112;
pub const FIR6: DspReg = 111;
pub const V6OUTX: DspReg = 105;
pub const V6ENVX: DspReg = 104;
pub const V6GAIN: DspReg = 103;
pub const V6ADSR2: DspReg = 102;
pub const V6ADSR1: DspReg = 101;
pub const V6SRCN: DspReg = 100;
pub const V6PH: DspReg = 99;
pub const V6PL: DspReg = 98;
pub const V6VOLR: DspReg = 97;
pub const V6VOLL: DspReg = 96;
pub const FIR5: DspReg = 95;
pub const V5OUTX: DspReg = 89;
pub const V5ENVX: DspReg = 88;
pub const V5GAIN: DspReg = 87;
pub const V5ADSR2: DspReg = 86;
pub const V5ADSR1: DspReg = 85;
pub const V5SRCN: DspReg = 84;
pub const V5PH: DspReg = 83;
pub const V5PL: DspReg = 82;
pub const V5VOLR: DspReg = 81;
pub const V5VOLL: DspReg = 80;
pub const FIR4: DspReg = 79;
pub const V4OUTX: DspReg = 73;
pub const V4ENVX: DspReg = 72;
pub const V4GAIN: DspReg = 71;
pub const V4ADSR2: DspReg = 70;
pub const V4ADSR1: DspReg = 69;
pub const V4SRCN: DspReg = 68;
pub const V4PH: DspReg = 67;
pub const V4PL: DspReg = 66;
pub const V4VOLR: DspReg = 65;
pub const V4VOLL: DspReg = 64;
pub const FIR3: DspReg = 63;
pub const V3OUTX: DspReg = 57;
pub const V3ENVX: DspReg = 56;
pub const V3GAIN: DspReg = 55;
pub const V3ADSR2: DspReg = 54;
pub const V3ADSR1: DspReg = 53;
pub const V3SRCN: DspReg = 52;
pub const V3PH: DspReg = 51;
pub const V3PL: DspReg = 50;
pub const V3VOLR: DspReg = 49;
pub const V3VOLL: DspReg = 48;
pub const FIR2: DspReg = 47;
pub const V2OUTX: DspReg = 41;
pub const V2ENVX: DspReg = 40;
pub const V2GAIN: DspReg = 39;
pub const V2ADSR2: DspReg = 38;
pub const V2ADSR1: DspReg = 37;
pub const V2SRCN: DspReg = 36;
pub const V2PH: DspReg = 35;
pub const V2PL: DspReg = 34;
pub const V2VOLR: DspReg = 33;
pub const V2VOLL: DspReg = 32;
pub const FIR1: DspReg = 31;
pub const V1OUTX: DspReg = 25;
pub const V1ENVX: DspReg = 24;
pub const V1GAIN: DspReg = 23;
pub const V1ADSR2: DspReg = 22;
pub const V1ADSR1: DspReg = 21;
pub const V1SRCN: DspReg = 20;
pub const V1PH: DspReg = 19;
pub const V1PL: DspReg = 18;
pub const V1VOLR: DspReg = 17;
pub const V1VOLL: DspReg = 16;
pub const V0OUTX: DspReg = 9;
pub const V0ENVX: DspReg = 8;
pub const V0VOLR: DspReg = 1;
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
static kChannel_Maps: [MemMap; 47] = [
    MemMap::new(0, 0x8030),
    MemMap::new(3, 0x70),
    MemMap::new(4, 0x71),
    MemMap::new(5, 0x80),
    MemMap::new(6, 0x90),
    MemMap::new(7, 0x91),
    MemMap::new(8, 0xa0),
    MemMap::new(9, 0xa1),
    MemMap::new(10, 0xb0),
    MemMap::new(11, 0xb1),
    MemMap::new(12, 0xc0),
    MemMap::new(13, 0xc1),
    MemMap::new(14, 0x100),
    MemMap::new(15, 0x200),
    MemMap::new(16, 0x201),
    MemMap::new(17, 0x210),
    MemMap::new(18, 0x211),
    MemMap::new(20, 0x8220),
    MemMap::new(22, 0x8230),
    MemMap::new(24, 0x8240),
    MemMap::new(26, 0x280),
    MemMap::new(27, 0x281),
    MemMap::new(28, 0x290),
    MemMap::new(29, 0x291),
    MemMap::new(30, 0x2a0),
    MemMap::new(31, 0x2a1),
    MemMap::new(32, 0x2b0),
    MemMap::new(33, 0x2b1),
    MemMap::new(34, 0x2c0),
    MemMap::new(35, 0x2c1),
    MemMap::new(36, 0x2d0),
    MemMap::new(37, 0x2d1),
    MemMap::new(38, 0x2e0),
    MemMap::new(39, 0x2f0),
    MemMap::new(40, 0x8300),
    MemMap::new(42, 0x8310),
    MemMap::new(44, 0x320),
    MemMap::new(45, 0x321),
    MemMap::new(46, 0x8330),
    MemMap::new(48, 0x8340),
    MemMap::new(50, 0x350),
    MemMap::new(51, 0x351),
    MemMap::new(52, 0x8360),
    MemMap::new(54, 0x8370),
    MemMap::new(56, 0x380),
    MemMap::new(57, 0x381),
    MemMap::new(58, 0x400),
];
static kSpcPlayer_Maps: [MemMapSized; 283] = [
    MemMapSized::new(855, 0, 4),
    MemMapSized::new(24, 0x4, 4),
    MemMapSized::new(28, 0x8, 4),
    MemMapSized::new(32, 0xc, 2),
    MemMapSized::new(34, 0xe, 2),
    MemMapSized::new(36, 0x10, 2),
    MemMapSized::new(38, 0x12, 1),
    MemMapSized::new(39, 0x13, 1),
    MemMapSized::new(40, 0x14, 2),
    MemMapSized::new(42, 0x16, 2),
    MemMapSized::new(44, 0x18, 2),
    MemMapSized::new(46, 0x1a, 1),
    MemMapSized::new(47, 0x1b, 1),
    MemMapSized::new(8, 0x1c, 1),
    MemMapSized::new(48, 0x20, 2),
    MemMapSized::new(592, 0x22, 2),
    MemMapSized::new(582, 0x24, 2),
    MemMapSized::new(586, 0x26, 2),
    MemMapSized::new(588, 0x28, 2),
    MemMapSized::new(658, 0x2a, 2),
    MemMapSized::new(684, 0x2c, 2),
    MemMapSized::new(710, 0x2e, 2),
    MemMapSized::new(50, 0x40, 2),
    MemMapSized::new(52, 0x42, 1),
    MemMapSized::new(53, 0x43, 1),
    MemMapSized::new(54, 0x44, 1),
    MemMapSized::new(55, 0x45, 1),
    MemMapSized::new(56, 0x46, 1),
    MemMapSized::new(57, 0x47, 1),
    MemMapSized::new(58, 0x48, 1),
    MemMapSized::new(59, 0x49, 1),
    MemMapSized::new(60, 0x4a, 1),
    MemMapSized::new(61, 0x4c, 1),
    MemMapSized::new(62, 0x4d, 1),
    MemMapSized::new(63, 0x4e, 1),
    MemMapSized::new(64, 0x50, 1),
    MemMapSized::new(65, 0x51, 1),
    MemMapSized::new(66, 0x52, 2),
    MemMapSized::new(68, 0x54, 1),
    MemMapSized::new(69, 0x55, 1),
    MemMapSized::new(70, 0x56, 2),
    MemMapSized::new(72, 0x58, 2),
    MemMapSized::new(74, 0x5a, 1),
    MemMapSized::new(75, 0x5b, 1),
    MemMapSized::new(76, 0x5c, 2),
    MemMapSized::new(78, 0x5e, 1),
    MemMapSized::new(79, 0x5f, 1),
    MemMapSized::new(80, 0x60, 2),
    MemMapSized::new(82, 0x62, 2),
    MemMapSized::new(84, 0x64, 2),
    MemMapSized::new(86, 0x66, 2),
    MemMapSized::new(88, 0x68, 1),
    MemMapSized::new(89, 0x69, 1),
    MemMapSized::new(90, 0x6a, 1),
    MemMapSized::new(736, 0xd0, 2),
    MemMapSized::new(92, 0xd2, 2),
    MemMapSized::new(614, 0xd4, 2),
    MemMapSized::new(604, 0xd6, 2),
    MemMapSized::new(608, 0xd8, 2),
    MemMapSized::new(610, 0xda, 2),
    MemMapSized::new(762, 0xdc, 2),
    MemMapSized::new(788, 0xde, 2),
    MemMapSized::new(636, 0xe0, 2),
    MemMapSized::new(626, 0xe2, 2),
    MemMapSized::new(630, 0xe4, 2),
    MemMapSized::new(632, 0xe6, 2),
    MemMapSized::new(814, 0xe8, 2),
    MemMapSized::new(840, 0xea, 2),
    MemMapSized::new(94, 0xee, 2),
    MemMapSized::new(96, 0x390, 1),
    MemMapSized::new(584, 0x392, 1),
    MemMapSized::new(598, 0x393, 1),
    MemMapSized::new(651, 0x394, 1),
    MemMapSized::new(677, 0x395, 1),
    MemMapSized::new(703, 0x396, 1),
    MemMapSized::new(729, 0x397, 1),
    MemMapSized::new(647, 0x398, 1),
    MemMapSized::new(673, 0x399, 1),
    MemMapSized::new(699, 0x39a, 1),
    MemMapSized::new(725, 0x39b, 1),
    MemMapSized::new(668, 0x39c, 1),
    MemMapSized::new(694, 0x39d, 1),
    MemMapSized::new(720, 0x39e, 1),
    MemMapSized::new(746, 0x39f, 1),
    MemMapSized::new(596, 0x3a0, 1),
    MemMapSized::new(595, 0x3a1, 1),
    MemMapSized::new(601, 0x3a2, 1),
    MemMapSized::new(597, 0x3a3, 1),
    MemMapSized::new(600, 0x3a4, 1),
    MemMapSized::new(599, 0x3a5, 1),
    MemMapSized::new(646, 0x3a6, 1),
    MemMapSized::new(672, 0x3a7, 1),
    MemMapSized::new(698, 0x3a8, 1),
    MemMapSized::new(724, 0x3a9, 1),
    MemMapSized::new(666, 0x3aa, 1),
    MemMapSized::new(692, 0x3ab, 1),
    MemMapSized::new(718, 0x3ac, 1),
    MemMapSized::new(744, 0x3ad, 1),
    MemMapSized::new(581, 0x3ae, 1),
    MemMapSized::new(652, 0x3af, 1),
    MemMapSized::new(678, 0x3b0, 1),
    MemMapSized::new(704, 0x3b1, 1),
    MemMapSized::new(730, 0x3b2, 1),
    MemMapSized::new(594, 0x3b3, 1),
    MemMapSized::new(653, 0x3b4, 1),
    MemMapSized::new(679, 0x3b5, 1),
    MemMapSized::new(705, 0x3b6, 1),
    MemMapSized::new(731, 0x3b7, 1),
    MemMapSized::new(655, 0x3b8, 1),
    MemMapSized::new(657, 0x3b9, 1),
    MemMapSized::new(681, 0x3ba, 1),
    MemMapSized::new(683, 0x3bb, 1),
    MemMapSized::new(707, 0x3bc, 1),
    MemMapSized::new(709, 0x3bd, 1),
    MemMapSized::new(733, 0x3be, 1),
    MemMapSized::new(735, 0x3bf, 1),
    MemMapSized::new(662, 0x3c0, 1),
    MemMapSized::new(654, 0x3c1, 1),
    MemMapSized::new(688, 0x3c2, 1),
    MemMapSized::new(680, 0x3c3, 1),
    MemMapSized::new(714, 0x3c4, 1),
    MemMapSized::new(706, 0x3c5, 1),
    MemMapSized::new(740, 0x3c6, 1),
    MemMapSized::new(732, 0x3c7, 1),
    MemMapSized::new(660, 0x3c8, 1),
    MemMapSized::new(686, 0x3c9, 1),
    MemMapSized::new(712, 0x3ca, 1),
    MemMapSized::new(738, 0x3cb, 1),
    MemMapSized::new(649, 0x3cc, 1),
    MemMapSized::new(675, 0x3cd, 1),
    MemMapSized::new(701, 0x3ce, 1),
    MemMapSized::new(727, 0x3cf, 1),
    MemMapSized::new(664, 0x3d0, 2),
    MemMapSized::new(690, 0x3d2, 2),
    MemMapSized::new(716, 0x3d4, 2),
    MemMapSized::new(742, 0x3d6, 2),
    MemMapSized::new(670, 0x3d8, 1),
    MemMapSized::new(696, 0x3d9, 1),
    MemMapSized::new(722, 0x3da, 1),
    MemMapSized::new(748, 0x3db, 1),
    MemMapSized::new(650, 0x3dc, 1),
    MemMapSized::new(648, 0x3dd, 1),
    MemMapSized::new(669, 0x3de, 1),
    MemMapSized::new(656, 0x3df, 1),
    MemMapSized::new(661, 0x3e0, 1),
    MemMapSized::new(667, 0x3e1, 1),
    MemMapSized::new(663, 0x3e2, 1),
    MemMapSized::new(676, 0x3e3, 1),
    MemMapSized::new(674, 0x3e4, 1),
    MemMapSized::new(695, 0x3e5, 1),
    MemMapSized::new(682, 0x3e6, 1),
    MemMapSized::new(687, 0x3e7, 1),
    MemMapSized::new(693, 0x3e8, 1),
    MemMapSized::new(689, 0x3e9, 1),
    MemMapSized::new(702, 0x3ea, 1),
    MemMapSized::new(700, 0x3eb, 1),
    MemMapSized::new(721, 0x3ec, 1),
    MemMapSized::new(708, 0x3ed, 1),
    MemMapSized::new(713, 0x3ee, 1),
    MemMapSized::new(719, 0x3ef, 1),
    MemMapSized::new(715, 0x3f0, 1),
    MemMapSized::new(728, 0x3f1, 1),
    MemMapSized::new(726, 0x3f2, 1),
    MemMapSized::new(747, 0x3f3, 1),
    MemMapSized::new(734, 0x3f4, 1),
    MemMapSized::new(739, 0x3f5, 1),
    MemMapSized::new(745, 0x3f6, 1),
    MemMapSized::new(741, 0x3f7, 1),
    MemMapSized::new(606, 0x3f8, 1),
    MemMapSized::new(620, 0x3f9, 1),
    MemMapSized::new(755, 0x3fa, 1),
    MemMapSized::new(781, 0x3fb, 1),
    MemMapSized::new(751, 0x3fc, 1),
    MemMapSized::new(777, 0x3fd, 1),
    MemMapSized::new(772, 0x3fe, 1),
    MemMapSized::new(798, 0x3ff, 1),
    MemMapSized::new(618, 0x440, 1),
    MemMapSized::new(617, 0x441, 1),
    MemMapSized::new(623, 0x442, 1),
    MemMapSized::new(619, 0x443, 1),
    MemMapSized::new(622, 0x444, 1),
    MemMapSized::new(621, 0x445, 1),
    MemMapSized::new(750, 0x446, 1),
    MemMapSized::new(776, 0x447, 1),
    MemMapSized::new(770, 0x448, 1),
    MemMapSized::new(796, 0x449, 1),
    MemMapSized::new(603, 0x44a, 1),
    MemMapSized::new(756, 0x44b, 1),
    MemMapSized::new(782, 0x44c, 1),
    MemMapSized::new(616, 0x44d, 1),
    MemMapSized::new(757, 0x44e, 1),
    MemMapSized::new(783, 0x44f, 1),
    MemMapSized::new(759, 0x450, 1),
    MemMapSized::new(761, 0x451, 1),
    MemMapSized::new(785, 0x452, 1),
    MemMapSized::new(787, 0x453, 1),
    MemMapSized::new(766, 0x454, 1),
    MemMapSized::new(758, 0x455, 1),
    MemMapSized::new(792, 0x456, 1),
    MemMapSized::new(784, 0x457, 1),
    MemMapSized::new(764, 0x458, 1),
    MemMapSized::new(790, 0x459, 1),
    MemMapSized::new(753, 0x45a, 1),
    MemMapSized::new(779, 0x45b, 1),
    MemMapSized::new(768, 0x45c, 2),
    MemMapSized::new(794, 0x45e, 2),
    MemMapSized::new(774, 0x460, 1),
    MemMapSized::new(800, 0x461, 1),
    MemMapSized::new(754, 0x462, 1),
    MemMapSized::new(752, 0x463, 1),
    MemMapSized::new(773, 0x464, 1),
    MemMapSized::new(760, 0x465, 1),
    MemMapSized::new(765, 0x466, 1),
    MemMapSized::new(771, 0x467, 1),
    MemMapSized::new(767, 0x468, 1),
    MemMapSized::new(780, 0x469, 1),
    MemMapSized::new(778, 0x46a, 1),
    MemMapSized::new(799, 0x46b, 1),
    MemMapSized::new(786, 0x46c, 1),
    MemMapSized::new(791, 0x46d, 1),
    MemMapSized::new(797, 0x46e, 1),
    MemMapSized::new(793, 0x46f, 1),
    MemMapSized::new(628, 0x470, 1),
    MemMapSized::new(642, 0x471, 1),
    MemMapSized::new(807, 0x472, 1),
    MemMapSized::new(833, 0x473, 1),
    MemMapSized::new(803, 0x474, 1),
    MemMapSized::new(829, 0x475, 1),
    MemMapSized::new(824, 0x476, 1),
    MemMapSized::new(850, 0x477, 1),
    MemMapSized::new(640, 0x478, 1),
    MemMapSized::new(639, 0x479, 1),
    MemMapSized::new(645, 0x47a, 1),
    MemMapSized::new(641, 0x47b, 1),
    MemMapSized::new(644, 0x47c, 1),
    MemMapSized::new(643, 0x47d, 1),
    MemMapSized::new(802, 0x47e, 1),
    MemMapSized::new(828, 0x47f, 1),
    MemMapSized::new(822, 0x480, 1),
    MemMapSized::new(848, 0x481, 1),
    MemMapSized::new(625, 0x482, 1),
    MemMapSized::new(808, 0x483, 1),
    MemMapSized::new(834, 0x484, 1),
    MemMapSized::new(638, 0x485, 1),
    MemMapSized::new(809, 0x486, 1),
    MemMapSized::new(835, 0x487, 1),
    MemMapSized::new(811, 0x488, 1),
    MemMapSized::new(813, 0x489, 1),
    MemMapSized::new(837, 0x48a, 1),
    MemMapSized::new(839, 0x48b, 1),
    MemMapSized::new(818, 0x48c, 1),
    MemMapSized::new(810, 0x48d, 1),
    MemMapSized::new(844, 0x48e, 1),
    MemMapSized::new(836, 0x48f, 1),
    MemMapSized::new(816, 0x490, 1),
    MemMapSized::new(842, 0x491, 1),
    MemMapSized::new(805, 0x492, 1),
    MemMapSized::new(831, 0x493, 1),
    MemMapSized::new(820, 0x494, 2),
    MemMapSized::new(846, 0x496, 2),
    MemMapSized::new(826, 0x498, 1),
    MemMapSized::new(852, 0x499, 1),
    MemMapSized::new(806, 0x49a, 1),
    MemMapSized::new(804, 0x49b, 1),
    MemMapSized::new(825, 0x49c, 1),
    MemMapSized::new(812, 0x49d, 1),
    MemMapSized::new(817, 0x49e, 1),
    MemMapSized::new(823, 0x49f, 1),
    MemMapSized::new(819, 0x4a0, 1),
    MemMapSized::new(832, 0x4a1, 1),
    MemMapSized::new(830, 0x4a2, 1),
    MemMapSized::new(851, 0x4a3, 1),
    MemMapSized::new(838, 0x4a4, 1),
    MemMapSized::new(843, 0x4a5, 1),
    MemMapSized::new(849, 0x4a6, 1),
    MemMapSized::new(845, 0x4a7, 1),
    MemMapSized::new(97, 0x4a9, 1),
    MemMapSized::new(98, 0x4b1, 1),
    MemMapSized::new(634, 0x4ba, 1),
    MemMapSized::new(580, 0x4bb, 1),
    MemMapSized::new(602, 0x4bc, 1),
    MemMapSized::new(624, 0x4bd, 1),
    MemMapSized::new(854, 0x4be, 1),
];
unsafe fn Dsp_Write(p: *mut SpcPlayer, mut reg: u8, value: u8) {
    let hist = (*p).reg_write_history;
    if !hist.is_null() && (*hist).count < 256u32 {
        if (*hist).count == 9u32 && reg as i32 == 0x5di32 {
            reg = (reg as i32 + 0i32) as u8;
        }
        (*hist).addr[(*hist).count as usize] = reg;
        (*hist).val[(*hist).count as usize] = value;
        (*hist).count = (*hist).count.wrapping_add(1);
    }
    if !(*p).dsp.is_null() {
        dsp_write((*p).dsp, reg, value);
    }
}
unsafe fn Not_Implemented() {
    crate::native_assertion! {
        '_c2rust_label;
        b"0\0", b"src/spc_player.c\0", 383u32, b"void Not_Implemented(void)\0"
    };
    printf(b"Not Implemented\n\0" as *const u8 as *const c_char);
}
unsafe fn SpcDivHelper(mut a: i32, b: u8) -> u16 {
    let org_a: i32 = a;
    if a & 0x100i32 != 0 {
        a = -a;
    }
    let q: i32 = if b as i32 != 0 {
        (a & 0xffi32) / b as i32
    } else {
        0xffi32
    };
    let r: i32 = if b as i32 != 0 {
        (a & 0xffi32) % b as i32
    } else {
        a & 0xffi32
    };
    let t: i32 = (q << 8i32)
        + (if b as i32 != 0 {
            (r << 8i32) / b as i32 & 0xffi32
        } else {
            0xffi32
        });
    ((if org_a & 0x100i32 != 0 { -t } else { t }) as u16)
}
unsafe fn SetupEchoParameter_EDL(p: *mut SpcPlayer, mut a: u8) {
    (*p).echo_parameter_EDL = a;
    if a as i32 != (*p).last_written_edl as i32 {
        a = ((*p).last_written_edl & 0xf ^ 0xff);
        if (*p).echo_stored_time & 0x80 != 0 {
            a = (a as i32 + (*p).echo_stored_time as i32) as u8;
        }
        (*p).echo_stored_time = a;
        Dsp_Write(p, EON as i32 as u8, 0u8);
        Dsp_Write(p, EFB as i32 as u8, 0u8);
        Dsp_Write(p, EVOLR as i32 as u8, 0u8);
        Dsp_Write(p, EVOLL as i32 as u8, 0u8);
        Dsp_Write(p, FLG as i32 as u8, ((*p).reg_FLG | 0x20));
        (*p).last_written_edl = (*p).echo_parameter_EDL;
        Dsp_Write(p, EDL as i32 as u8, (*p).echo_parameter_EDL);
    }
    Dsp_Write(
        p,
        ESA as i32 as u8,
        (((*p).echo_parameter_EDL as i32 * 8i32 ^ 0xffi32) + 0x16i32) as u8,
    );
}
unsafe fn WriteVolumeToDsp(p: *mut SpcPlayer, c: *mut Channel, mut volume: u16) {
    static kVolumeTable: [u8; 22] = [
        0, 1, 3, 7, 13, 21, 30, 41, 52, 66, 81, 94, 103, 110, 115, 119, 122, 124, 125, 126, 127,
        127,
    ];
    if (*p).is_chan_on & (*p).cur_chan_bit != 0 {
        return;
    }
    let mut i: i32 = 0i32;
    while i < 2i32 {
        let j: i32 = volume as i32 >> 8i32;
        let mut t: u8 = 0;
        if j >= 21i32 {
            t = ((*p).ram[(j + 0x1e1di32) as usize] as i32
                + (((*p).ram[(j + 0x1e1di32 + 1i32) as usize] as i32
                    - (*p).ram[(j + 0x1e1di32) as usize] as i32)
                    * volume as u8 as i32
                    >> 8i32)) as u8;
        } else {
            t = (kVolumeTable[j as usize] as i32
                + ((kVolumeTable[(j + 1i32) as usize] as i32 - kVolumeTable[j as usize] as i32)
                    * volume as u8 as i32
                    >> 8i32)) as u8;
        }
        t = (t as i32 * (*c).final_volume as i32 >> 8i32) as u8;
        if ((*c).pan_flag_with_phase_invert as i32) << i & 0x80i32 != 0 {
            t = -(t as i32) as u8;
        }
        Dsp_Write(p, (V0VOLL as i32 + i + (*c).index as i32 * 16i32) as u8, t);
        volume = (0x1400i32 - volume as i32) as u16;
        i += 1;
    }
}
unsafe fn WritePitchInner(p: *mut SpcPlayer, c: *mut Channel, pitch: u16) {
    static kBaseNoteFreqs: [u16; 13] = [
        2143, 2270, 2405, 2548, 2700, 2860, 3030, 3211, 3402, 3604, 3818, 4045, 4286,
    ];
    let pp: u8 = (pitch as i32 >> 8i32 & 0x7fi32) as u8;
    let mut q: u8 = (pp as i32 / 12i32) as u8;
    let r: u8 = (pp as i32 % 12i32) as u8;
    let mut t: u16 = (kBaseNoteFreqs[r as usize] as i32
        + ((kBaseNoteFreqs[(r as i32 + 1i32) as usize] as i32 - kBaseNoteFreqs[r as usize] as i32)
            as u8 as i32
            * pitch as u8 as i32
            >> 8i32)) as u16;
    t = (t as i32 * 2i32) as u16;
    while q as i32 != 6i32 {
        t = (t >> 1);
        q = q.wrapping_add(1);
    }
    t = ((*c).instrument_pitch_base as i32 * t as i32 >> 8i32) as u16;
    if (*p).cur_chan_bit & (*p).is_chan_on == 0 {
        let reg: u8 = ((*c).index as i32 * 16i32) as u8;
        Dsp_Write(p, (reg as i32 + V0PITCHL as i32) as u8, (t as u8 & 0xff));
        Dsp_Write(p, (reg as i32 + V0PITCHH as i32) as u8, (t >> 8) as u8);
    }
}
unsafe fn WritePitch(p: *mut SpcPlayer, c: *mut Channel, mut pitch: u16) {
    if pitch as i32 >> 8i32 >= 0x34i32 {
        pitch = (pitch as i32 + ((pitch as i32 >> 8i32) - 0x34i32)) as u16;
    } else if (pitch as i32 >> 8i32) < 0x13i32 {
        pitch = (pitch as i32 + ((((pitch as i32 >> 8i32) - 0x13i32) * 2i32) as u8 as i32 - 256i32))
            as u16;
    }
    WritePitchInner(p, c, pitch);
}
unsafe fn Music_ResetChan(p: *mut SpcPlayer) {
    let mut c = (&raw mut (*p).channel as *mut Channel).offset(7);
    (*p).cur_chan_bit = 0x80u8;
    loop {
        *(&raw mut (*c).channel_volume as *mut u8).offset(
            (size_of::<u16>() as usize)
                .wrapping_div(size_of::<u8>() as usize)
                .wrapping_sub(1_usize) as isize,
        ) = 0xffu8;
        (*c).pan_flag_with_phase_invert = 10u8;
        (*c).pan_value = ((10i32) << 8i32) as u16;
        set_fields! {[(*c)], 0u8; instrument_id, fine_tune, channel_transposition, pitch_envelope_num_ticks, cutk, vib_depth, tremolo_depth}
        c = c.offset(-1);
        (*p).cur_chan_bit = ((*p).cur_chan_bit >> 1);
        if ((*p).cur_chan_bit == 0) {
            break;
        }
    }
    set_fields! {[(*p)], 0u8; master_volume_fade_ticks, echo_volume_fade_ticks, tempo_fade_num_ticks, global_transposition, block_count, percussion_base_id}
    *(&raw mut (*p).master_volume as *mut u8).offset(
        (size_of::<u16>() as usize)
            .wrapping_div(size_of::<u8>() as usize)
            .wrapping_sub(1_usize) as isize,
    ) = 0xc0u8;
    *(&raw mut (*p).tempo as *mut u8).offset(
        (size_of::<u16>() as usize)
            .wrapping_div(size_of::<u8>() as usize)
            .wrapping_sub(1_usize) as isize,
    ) = 0x20u8;
}
unsafe fn Channel_SetInstrument_NoSetId(p: *mut SpcPlayer, c: *mut Channel, mut instrument: u8) {
    if instrument & 0x80 != 0 {
        instrument = (instrument as i32 + 54i32 + (*p).percussion_base_id as i32) as u8;
    }
    let ip: *const u8 = (&raw mut (*p).ram as *mut u8)
        .offset((instrument as i32 * 6i32) as isize)
        .offset(0x6c00);
    if (*p).is_chan_on & (*p).cur_chan_bit != 0 {
        return;
    }
    let reg: u8 = ((*c).index as i32 * 16i32) as u8;
    if *ip.offset(0) & 0x80 != 0 {
        (*p).reg_FLG = ((*p).reg_FLG & 0x20 | *ip.offset(0) & 0x1f);
        (*p).reg_NON = ((*p).reg_NON | (*p).cur_chan_bit);
        Dsp_Write(p, (reg as i32 + V0SRCN as i32) as u8, 0u8);
    } else {
        Dsp_Write(p, (reg as i32 + V0SRCN as i32) as u8, *ip.offset(0));
    }
    Dsp_Write(p, (reg as i32 + V0ADSR1 as i32) as u8, *ip.offset(1));
    Dsp_Write(p, (reg as i32 + V0ADSR2 as i32) as u8, *ip.offset(2));
    Dsp_Write(p, (reg as i32 + V0GAIN as i32) as u8, *ip.offset(3));
    (*c).instrument_pitch_base = ((*ip.offset(4) as i32) << 8i32 | *ip.offset(5) as i32) as u16;
}
unsafe fn Channel_SetInstrument(p: *mut SpcPlayer, c: *mut Channel, instrument: u8) {
    (*c).instrument_id = instrument;
    Channel_SetInstrument_NoSetId(p, c, instrument);
}
unsafe fn ComputePitchAdd(c: *mut Channel, pitch: u8) {
    (*c).pitch_target = (pitch & 0x7f);
    (*c).pitch_add_per_tick = SpcDivHelper(
        (*c).pitch_target as i32 - ((*c).pitch as i32 >> 8i32),
        (*c).pitch_slide_length,
    );
}
unsafe fn PitchSlideToNote_Check(p: *mut SpcPlayer, c: *mut Channel) {
    if (*c).pitch_slide_length as i32 != 0
        || (*p).ram[(*c).pattern_order_ptr_for_chan as usize] as i32 != 0xf9i32
    {
        return;
    }
    (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
    let fresh3 = (*c).pattern_order_ptr_for_chan;
    (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
    (*c).pitch_slide_delay_left = (*p).ram[fresh3 as usize];
    let fresh4 = (*c).pattern_order_ptr_for_chan;
    (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
    (*c).pitch_slide_length = (*p).ram[fresh4 as usize];
    let fresh5 = (*c).pattern_order_ptr_for_chan;
    (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
    ComputePitchAdd(
        c,
        ((*p).ram[fresh5 as usize] as i32
            + (*p).global_transposition as i32
            + (*c).channel_transposition as i32) as u8,
    );
}
static kEffectByteLength: [u8; 31] = [
    1, 1, 2, 3, 0, 1, 2, 1, 2, 1, 1, 3, 0, 1, 2, 3, 1, 3, 3, 0, 1, 3, 0, 3, 3, 3, 1, 2, 0, 0, 0,
];
unsafe fn HandleEffect(p: *mut SpcPlayer, c: *mut Channel, effect: u8) {
    let arg: u8 = (if kEffectByteLength[(effect as i32 - 0xe0i32) as usize] as i32 != 0 {
        let fresh9 = (*c).pattern_order_ptr_for_chan;
        (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
        (*p).ram[fresh9 as usize] as i32
    } else {
        0i32
    }) as u8;
    match effect as i32 {
        224 => Channel_SetInstrument(p, c, arg),
        225 => {
            (*c).pan_flag_with_phase_invert = arg;
            (*c).pan_value = ((arg as i32 & 0x1fi32) << 8i32) as u16;
        }
        226 => {
            (*c).pan_num_ticks = arg;
            let fresh10 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pan_target_value = (*p).ram[fresh10 as usize];
            (*c).pan_add_per_tick = SpcDivHelper(
                (*c).pan_target_value as i32 - ((*c).pan_value as i32 >> 8i32),
                arg,
            );
        }
        227 => {
            (*c).vibrato_delay_ticks = arg;
            let fresh11 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).vibrato_rate = (*p).ram[fresh11 as usize];
            let fresh12 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).vib_depth = (*p).ram[fresh12 as usize];
            (*c).vibrato_depth_target = (*c).vib_depth;
            (*c).vibrato_fade_num_ticks = 0u8;
        }
        228 => {
            (*c).vib_depth = 0u8;
            (*c).vibrato_depth_target = (*c).vib_depth;
            (*c).vibrato_fade_num_ticks = 0u8;
        }
        229 => (*p).master_volume = ((arg as i32) << 8i32) as u16,
        230 => {
            (*p).master_volume_fade_ticks = arg;
            let fresh13 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).master_volume_fade_target = (*p).ram[fresh13 as usize];
            (*p).master_volume_fade_add_per_tick = SpcDivHelper(
                (*p).master_volume_fade_target as i32 - ((*p).master_volume as i32 >> 8i32),
                arg,
            );
        }
        231 => (*p).tempo = ((arg as i32) << 8i32) as u16,
        232 => {
            (*p).tempo_fade_num_ticks = arg;
            let fresh14 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).tempo_fade_final = (*p).ram[fresh14 as usize];
            (*p).tempo_fade_add = SpcDivHelper(
                (*p).tempo_fade_final as i32 - ((*p).tempo as i32 >> 8i32),
                arg,
            );
        }
        233 => (*p).global_transposition = arg,
        234 => (*c).channel_transposition = arg,
        235 => {
            (*c).tremolo_delay_ticks = arg;
            let fresh15 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).tremolo_rate = (*p).ram[fresh15 as usize];
            let fresh16 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).tremolo_depth = (*p).ram[fresh16 as usize];
        }
        236 => (*c).tremolo_depth = 0u8,
        237 => (*c).channel_volume = ((arg as i32) << 8i32) as u16,
        238 => {
            (*c).volume_fade_ticks = arg;
            let fresh17 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).volume_fade_target = (*p).ram[fresh17 as usize];
            (*c).volume_fade_addpertick = SpcDivHelper(
                (*c).volume_fade_target as i32 - ((*c).channel_volume as i32 >> 8i32),
                arg,
            );
        }
        239 => {
            let fresh18 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pattern_start_ptr =
                (((*p).ram[fresh18 as usize] as i32) << 8i32 | arg as i32) as u16;
            let fresh19 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).subroutine_num_loops = (*p).ram[fresh19 as usize];
            (*c).saved_pattern_ptr = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_start_ptr;
        }
        240 => {
            (*c).vibrato_fade_num_ticks = arg;
            (*c).vibrato_fade_add_per_tick = (if arg as i32 != 0 {
                (*c).vib_depth as i32 / arg as i32
            } else {
                0xffi32
            }) as u8;
        }
        241 | 242 => {
            (*c).pitch_envelope_direction = (effect as i32 == 0xf1i32) as i32 as u8;
            (*c).pitch_envelope_delay = arg;
            let fresh20 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pitch_envelope_num_ticks = (*p).ram[fresh20 as usize];
            let fresh21 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pitch_envelope_slide_value = (*p).ram[fresh21 as usize];
        }
        243 => (*c).pitch_envelope_num_ticks = 0u8,
        244 => (*c).fine_tune = arg,
        245 => {
            (*p).reg_EON = arg;
            let fresh22 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).echo_volume_left = (((*p).ram[fresh22 as usize] as i32) << 8i32) as u16;
            let fresh23 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).echo_volume_right = (((*p).ram[fresh23 as usize] as i32) << 8i32) as u16;
            (*p).reg_FLG = ((*p).reg_FLG & !(0x20));
        }
        246 => {
            set_fields! {[(*p)], 0u16; echo_volume_left, echo_volume_right}
            (*p).reg_FLG = ((*p).reg_FLG | 0x20);
        }
        247 => {
            static kEchoFirParameters: [i8; 32] = [
                127, 0, 0, 0, 0, 0, 0, 0, 88, -65, -37, -16, -2, 7, 12, 12, 12, 33, 43, 43, 19, -2,
                -13, -7, 52, 51, 0, -39, -27, 1, -4, -21,
            ];
            SetupEchoParameter_EDL(p, arg);
            let fresh24 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).reg_EFB = (*p).ram[fresh24 as usize];
            let fresh25 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            let mut ep = (&raw const kEchoFirParameters as *const i8)
                .offset(((*p).ram[fresh25 as usize] as i32 * 8i32) as isize);
            let mut i: i32 = 0i32;
            while i < 8i32 {
                let fresh26 = ep;
                ep = ep.offset(1);
                Dsp_Write(p, (FIR0 as i32 + i * 16i32) as u8, *fresh26 as u8);
                i += 1;
            }
        }
        248 => {
            (*p).echo_volume_fade_ticks = arg;
            let fresh27 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).echo_volume_fade_target_left = (*p).ram[fresh27 as usize];
            let fresh28 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*p).echo_volume_fade_target_right = (*p).ram[fresh28 as usize];
            (*p).echo_volume_fade_add_left = SpcDivHelper(
                (*p).echo_volume_fade_target_left as i32 - ((*p).echo_volume_left as i32 >> 8i32),
                arg,
            );
            (*p).echo_volume_fade_add_right = SpcDivHelper(
                (*p).echo_volume_fade_target_right as i32 - ((*p).echo_volume_right as i32 >> 8i32),
                arg,
            );
        }
        249 => {
            (*c).pitch_slide_delay_left = arg;
            let fresh29 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            (*c).pitch_slide_length = (*p).ram[fresh29 as usize];
            let fresh30 = (*c).pattern_order_ptr_for_chan;
            (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1);
            ComputePitchAdd(
                c,
                ((*p).ram[fresh30 as usize] as i32
                    + (*p).global_transposition as i32
                    + (*c).channel_transposition as i32) as u8,
            );
        }
        250 => (*p).percussion_base_id = arg,
        251 => (*c).pattern_order_ptr_for_chan = (*c).pattern_order_ptr_for_chan.wrapping_add(1),
        252 => (*c).cutk = (arg as i32 + 1i32) as u8,
        253 => {
            (*p).fast_forward = (arg as i32 + 1i32) as u8;
            (*p).key_OFF = ((*p).key_OFF | !((*p).is_chan_on));
        }
        254 => {
            (*p).fast_forward = arg;
            (*p).key_OFF = ((*p).key_OFF | !((*p).is_chan_on));
        }
        _ => Not_Implemented(),
    };
}
unsafe fn WantWriteKof(p: *mut SpcPlayer, c: *mut Channel) -> bool {
    let mut loops: i32 = (*c).subroutine_num_loops as i32;
    let mut ptr: i32 = (*c).pattern_order_ptr_for_chan as i32;
    loop {
        let fresh7 = ptr;
        ptr = ptr + 1;
        let mut cmd: u8 = (*p).ram[fresh7 as usize];
        if cmd as i32 == 0i32 {
            if loops == 0i32 {
                return true_0 != 0;
            }
            loops -= 1;
            ptr = if loops == 0i32 {
                (*c).saved_pattern_ptr as i32
            } else {
                (*c).pattern_start_ptr as i32
            };
        } else {
            while cmd & 0x80 == 0 {
                let fresh8 = ptr;
                ptr = ptr + 1;
                cmd = (*p).ram[fresh8 as usize];
            }
            if cmd as i32 == 0xc8i32 {
                return false_0 != 0;
            }
            if cmd as i32 == 0xefi32 {
                ptr = (*p).ram[(ptr + 0i32) as usize] as i32
                    | ((*p).ram[(ptr + 1i32) as usize] as i32) << 8i32;
            } else if cmd as i32 >= 0xe0i32 {
                ptr += kEffectByteLength[(cmd as i32 - 0xe0i32) as usize] as i32;
            } else {
                return true_0 != 0;
            }
        }
    }
}
pub unsafe fn CalcFinalVolume(p: *mut SpcPlayer, c: *mut Channel, vol: u8) {
    let mut t: i32 = ((*p).master_volume as i32 >> 8i32) * vol as i32 >> 8i32;
    t = t * (*c).channel_volume_master as i32 >> 8i32;
    t = t * ((*c).channel_volume as i32 >> 8i32) >> 8i32;
    (*c).final_volume = (t * t >> 8i32) as u8;
}
unsafe fn CalcTremolo(p: *mut SpcPlayer, c: *mut Channel, mut a: u8) {
    if a & 0x80 != 0 {
        a = (a as i32 * 2i32 ^ 0xffi32) as u8;
    } else {
        a = (a as i32 * 2i32) as u8;
    }
    a = (a as i32 * (*c).tremolo_depth as i32 >> 8i32) as u8;
    CalcFinalVolume(p, c, (a ^ 0xff));
}
unsafe fn HandleTremolo(p: *mut SpcPlayer, c: *mut Channel) {
    (*p).did_affect_volumepitch_flag = ((*p).did_affect_volumepitch_flag | 0x80);
    CalcTremolo(
        p,
        c,
        ((*c).tremolo_rate as i32 * (*p).main_tempo_accum as i32 >> 8i32) as u8,
    );
}
unsafe fn CalcVibratoAddPitch(p: *mut SpcPlayer, c: *mut Channel, pitch: u16, value: u8) {
    let mut t: i32 = (value as i32) << 2i32;
    t ^= if t & 0x100i32 != 0 { 0xffi32 } else { 0i32 };
    let r: i32 = if (*c).vib_depth as i32 >= 0xf1i32 {
        t as u8 as i32 * ((*c).vib_depth as i32 & 0xfi32)
    } else {
        t as u8 as i32 * (*c).vib_depth as i32 >> 8i32
    };
    WritePitch(
        p,
        c,
        (pitch as i32 + (if value & 0x80 != 0 { -r } else { r })) as u16,
    );
}
unsafe fn HandlePanAndSweep(p: *mut SpcPlayer, c: *mut Channel) {
    (*p).did_affect_volumepitch_flag = 0u8;
    if (*c).tremolo_depth as i32 != 0
        && (*c).tremolo_hold_count as i32 == (*c).tremolo_delay_ticks as i32
    {
        HandleTremolo(p, c);
    }
    let mut volume: u16 = (*c).pan_value;
    if (*c).pan_num_ticks != 0 {
        (*p).did_affect_volumepitch_flag = 0x80u8;
        volume = (volume as i32
            + (*p).main_tempo_accum as i32 * (*c).pan_add_per_tick as i16 as i32 / 256i32)
            as u16;
    }
    if (*p).did_affect_volumepitch_flag != 0 {
        WriteVolumeToDsp(p, c, volume);
    }
    (*p).did_affect_volumepitch_flag = 0u8;
    let mut pitch: u16 = (*c).pitch;
    if (*c).pitch_slide_length as i32 != 0 && (*c).pitch_slide_delay_left == 0 {
        (*p).did_affect_volumepitch_flag = ((*p).did_affect_volumepitch_flag | 0x80);
        pitch = (pitch as i32
            + (*p).main_tempo_accum as i32 * (*c).pitch_add_per_tick as i16 as i32 / 256i32)
            as u16;
    }
    if (*c).vib_depth as i32 != 0
        && (*c).vibrato_delay_ticks as i32 == (*c).vibrato_hold_count as i32
    {
        CalcVibratoAddPitch(
            p,
            c,
            pitch,
            (((*p).main_tempo_accum as i32 * (*c).vibrato_rate as i32 >> 8i32)
                + (*c).vibrato_count as i32) as u8,
        );
        return;
    }
    if (*p).did_affect_volumepitch_flag != 0 {
        WritePitch(p, c, pitch);
    }
}
unsafe fn HandleNoteTick(p: *mut SpcPlayer, c: *mut Channel) {
    if (*c).note_keyoff_ticks_left as i32 != 0i32
        && {
            (*c).note_keyoff_ticks_left = (*c).note_keyoff_ticks_left.wrapping_sub(1);
            (*c).note_keyoff_ticks_left as i32 == 0i32 || (*c).note_ticks_left as i32 == 2i32
        }
        && WantWriteKof(p, c) as i32 != 0
        && (*p).cur_chan_bit & (*p).is_chan_on == 0
    {
        Dsp_Write(p, KOF as i32 as u8, (*p).cur_chan_bit);
    }
    (*p).did_affect_volumepitch_flag = 0u8;
    if (*c).pitch_slide_length != 0 {
        if (*c).pitch_slide_delay_left != 0 {
            (*c).pitch_slide_delay_left = (*c).pitch_slide_delay_left.wrapping_sub(1);
        } else {
            (*p).did_affect_volumepitch_flag = 0x80u8;
            (*c).pitch_slide_length = (*c).pitch_slide_length.wrapping_sub(1);
            if (*c).pitch_slide_length as i32 == 0i32 {
                (*c).pitch = (((*c).pitch_target as i32) << 8i32 | (*c).fine_tune as i32) as u16;
            } else {
                (*c).pitch = ((*c).pitch as i32 + (*c).pitch_add_per_tick as i32) as u16;
            }
        }
    }
    let pitch: u16 = (*c).pitch;
    if (*c).vib_depth != 0 {
        if (*c).vibrato_delay_ticks as i32 == (*c).vibrato_hold_count as i32 {
            if (*c).vibrato_change_count as i32 == (*c).vibrato_fade_num_ticks as i32 {
                (*c).vib_depth = (*c).vibrato_depth_target;
            } else {
                let fresh6 = (*c).vibrato_change_count;
                (*c).vibrato_change_count = (*c).vibrato_change_count.wrapping_add(1);
                (*c).vib_depth = ((if fresh6 as i32 == 0i32 {
                    0i32
                } else {
                    (*c).vib_depth as i32
                }) + (*c).vibrato_fade_add_per_tick as i32) as u8;
            }
            (*c).vibrato_count = ((*c).vibrato_count as i32 + (*c).vibrato_rate as i32) as u8;
            CalcVibratoAddPitch(p, c, pitch, (*c).vibrato_count);
            return;
        }
        (*c).vibrato_hold_count = (*c).vibrato_hold_count.wrapping_add(1);
    }
    if (*p).did_affect_volumepitch_flag != 0 {
        WritePitch(p, c, pitch);
    }
}
unsafe fn Chan_HandleTick(p: *mut SpcPlayer, c: *mut Channel) {
    if (*c).volume_fade_ticks != 0 {
        (*c).volume_fade_ticks = (*c).volume_fade_ticks.wrapping_sub(1);
        if (*c).volume_fade_ticks as i32 == 0i32 {
            (*c).channel_volume = (((*c).volume_fade_target as i32) << 8i32) as u16;
        } else {
            (*c).channel_volume =
                ((*c).channel_volume as i32 + (*c).volume_fade_addpertick as i32) as u16;
        }
        (*p).vol_dirty = ((*p).vol_dirty | (*p).cur_chan_bit);
    }
    if (*c).tremolo_depth != 0 {
        if (*c).tremolo_delay_ticks as i32 == (*c).tremolo_hold_count as i32 {
            (*p).vol_dirty = ((*p).vol_dirty | (*p).cur_chan_bit);
            if (*c).tremolo_count & 0x80 != 0 && (*c).tremolo_depth as i32 == 0xffi32 {
                (*c).tremolo_count = 0x80u8;
            } else {
                (*c).tremolo_count = ((*c).tremolo_count as i32 + (*c).tremolo_rate as i32) as u8;
            }
            CalcTremolo(p, c, (*c).tremolo_count);
        } else {
            (*c).tremolo_hold_count = (*c).tremolo_hold_count.wrapping_add(1);
            CalcFinalVolume(p, c, 0xffu8);
        }
    } else {
        CalcFinalVolume(p, c, 0xffu8);
    }
    if (*c).pan_num_ticks != 0 {
        (*c).pan_num_ticks = (*c).pan_num_ticks.wrapping_sub(1);
        if (*c).pan_num_ticks as i32 == 0i32 {
            (*c).pan_value = (((*c).pan_target_value as i32) << 8i32) as u16;
        } else {
            (*c).pan_value = ((*c).pan_value as i32 + (*c).pan_add_per_tick as i32) as u16;
        }
        (*p).vol_dirty = ((*p).vol_dirty | (*p).cur_chan_bit);
    }
    if (*p).vol_dirty & (*p).cur_chan_bit != 0 {
        WriteVolumeToDsp(p, c, (*c).pan_value);
    }
}
unsafe fn Music_HandleCmdFromSnes(p: *mut SpcPlayer) {
    let mut c = null_mut::<Channel>();
    let mut t: i32 = 0;
    let mut a: u8 = (*p).input_ports[0i32 as usize];
    (*p).input_ports[0i32 as usize] = 255u8;
    if a as i32 == 0xf0i32 {
        (*p).key_OFF = ((*p).key_OFF | !((*p).is_chan_on));
    } else {
        let mut read_toplevel_word: bool;
        if !(a as i32 != 0xf1i32
            && a as i32 != 255i32
            && a as i32 != (*p).port_to_snes[0i32 as usize] as i32)
        {
            if (*p).port_to_snes[0i32 as usize] as i32 == 0i32 {
                return;
            }
            read_toplevel_word = !((*p).counter_sf0c as i32 == 0i32)
                && ({
                    (*p).counter_sf0c = (*p).counter_sf0c.wrapping_sub(1);
                    if (*p).counter_sf0c as i32 != 0i32 {
                        Music_ResetChan(p);
                        return;
                    }
                    true
                });
            '_label_a: loop {
                if read_toplevel_word {
                    t = *((&raw mut (*p).ram as *mut u8).offset((*p).music_ptr_toplevel as isize)
                        as *mut u16) as i32;
                    (*p).music_ptr_toplevel = ((*p).music_ptr_toplevel as i32 + 2i32) as u16;
                    if t >> 8i32 != 0i32 {
                        let mut i: i32 = 0i32;
                        while i < 8i32 {
                            (*p).channel[i as usize].pattern_order_ptr_for_chan =
                                *((&raw mut (*p).ram as *mut u8).offset(t as isize) as *mut u16);
                            t += 2i32;
                            i += 1;
                        }
                        c = &raw mut (*p).channel as *mut Channel;
                        (*p).cur_chan_bit = 1u8;
                        loop {
                            if *(&raw mut (*c).pattern_order_ptr_for_chan as *mut u8).offset(
                                (size_of::<u16>() as usize)
                                    .wrapping_div(size_of::<u8>() as usize)
                                    .wrapping_sub(1_usize) as isize,
                            ) as i32
                                != 0
                                && (*c).instrument_id as i32 == 0i32
                            {
                                Channel_SetInstrument(p, c, 0u8);
                            }
                            set_fields! {[(*c)], 0u8; subroutine_num_loops, volume_fade_ticks, pan_num_ticks}
                            (*c).note_ticks_left = 1u8;
                            c = c.offset(1);
                            (*p).cur_chan_bit = (((*p).cur_chan_bit as i32) << 1i32) as u8;
                            if ((*p).cur_chan_bit == 0) {
                                break;
                            }
                        }
                        read_toplevel_word = false;
                    } else if t == 0i32 {
                        a = 0u8;
                        break;
                    } else {
                        if t == 0x80i32 {
                            (*p).fast_forward = 0x80u8;
                        } else if t == 0x81i32 {
                            (*p).fast_forward = 0u8;
                        } else {
                            (*p).block_count = (*p).block_count.wrapping_sub(1);
                            if (*p).block_count & 0x80 != 0 {
                                (*p).block_count = t as u8;
                            }
                            t = *((&raw mut (*p).ram as *mut u8)
                                .offset((*p).music_ptr_toplevel as isize)
                                as *mut u16) as i32;
                            (*p).music_ptr_toplevel =
                                ((*p).music_ptr_toplevel as i32 + 2i32) as u16;
                            if (*p).block_count as i32 != 0i32 {
                                (*p).music_ptr_toplevel = t as u16;
                            }
                        }
                        read_toplevel_word = true;
                    }
                } else {
                    (*p).vol_dirty = 0u8;
                    c = &raw mut (*p).channel as *mut Channel;
                    (*p).cur_chan_bit = 1u8;
                    loop {
                        if (*(&raw mut (*c).pattern_order_ptr_for_chan as *mut u8).offset(
                            (size_of::<u16>() as usize)
                                .wrapping_div(size_of::<u8>() as usize)
                                .wrapping_sub(1_usize) as isize,
                        ) != 0)
                        {
                            (*c).note_ticks_left = (*c).note_ticks_left.wrapping_sub(1);
                            if (*c).note_ticks_left == 0 {
                                loop {
                                    let fresh0 = (*c).pattern_order_ptr_for_chan;
                                    (*c).pattern_order_ptr_for_chan =
                                        (*c).pattern_order_ptr_for_chan.wrapping_add(1);
                                    let mut cmd: u8 = (*p).ram[fresh0 as usize];
                                    if cmd as i32 == 0i32 {
                                        if (*c).subroutine_num_loops == 0 {
                                            read_toplevel_word = true;
                                            continue '_label_a;
                                        }
                                        (*c).subroutine_num_loops =
                                            (*c).subroutine_num_loops.wrapping_sub(1);
                                        (*c).pattern_order_ptr_for_chan =
                                            (if (*c).subroutine_num_loops as i32 == 0i32 {
                                                (*c).saved_pattern_ptr as i32
                                            } else {
                                                (*c).pattern_start_ptr as i32
                                            }) as u16;
                                    } else {
                                        if cmd & 0x80 == 0 {
                                            static kNoteVol: [u8; 16] = [
                                                25, 50, 76, 101, 114, 127, 140, 152, 165, 178, 191,
                                                203, 216, 229, 242, 252,
                                            ];
                                            static kNoteGateOffPct: [u8; 8] =
                                                [50, 101, 127, 152, 178, 203, 229, 252];
                                            (*c).note_length = cmd;
                                            let fresh1 = (*c).pattern_order_ptr_for_chan;
                                            (*c).pattern_order_ptr_for_chan =
                                                (*c).pattern_order_ptr_for_chan.wrapping_add(1);
                                            cmd = (*p).ram[fresh1 as usize];
                                            if cmd & 0x80 == 0 {
                                                (*c).note_gate_off_fixedpt = kNoteGateOffPct
                                                    [(cmd as i32 >> 4i32 & 7i32) as usize];
                                                (*c).channel_volume_master =
                                                    kNoteVol[(cmd as i32 & 0xfi32) as usize];
                                                let fresh2 = (*c).pattern_order_ptr_for_chan;
                                                (*c).pattern_order_ptr_for_chan =
                                                    (*c).pattern_order_ptr_for_chan.wrapping_add(1);
                                                cmd = (*p).ram[fresh2 as usize];
                                            }
                                        }
                                        if cmd as i32 >= 0xe0i32 {
                                            HandleEffect(p, c, cmd);
                                        } else {
                                            if (*c).cutk | (*p).fast_forward == 0 {
                                                PlayNote(p, c, cmd);
                                            }
                                            (*c).note_ticks_left = (*c).note_length;
                                            t = (*c).note_ticks_left as i32
                                                * (*c).note_gate_off_fixedpt as i32
                                                >> 8i32;
                                            (*c).note_keyoff_ticks_left =
                                                (if t != 0i32 { t } else { 1i32 }) as u8;
                                            PitchSlideToNote_Check(p, c);
                                            break;
                                        }
                                    }
                                }
                            } else if (*p).fast_forward == 0 {
                                HandleNoteTick(p, c);
                                PitchSlideToNote_Check(p, c);
                            }
                        }
                        c = c.offset(1);
                        (*p).cur_chan_bit = (((*p).cur_chan_bit as i32) << 1i32) as u8;
                        if ((*p).cur_chan_bit == 0) {
                            break;
                        }
                    }
                    if (*p).tempo_fade_num_ticks != 0 {
                        (*p).tempo_fade_num_ticks = (*p).tempo_fade_num_ticks.wrapping_sub(1);
                        (*p).tempo = (if (*p).tempo_fade_num_ticks as i32 == 0i32 {
                            ((*p).tempo_fade_final as i32) << 8i32
                        } else {
                            (*p).tempo as i32 + (*p).tempo_fade_add as i32
                        }) as u16;
                    }
                    if (*p).echo_volume_fade_ticks != 0 {
                        (*p).echo_volume_left = ((*p).echo_volume_left as i32
                            + (*p).echo_volume_fade_add_left as i32)
                            as u16;
                        (*p).echo_volume_right = ((*p).echo_volume_right as i32
                            + (*p).echo_volume_fade_add_right as i32)
                            as u16;
                        (*p).echo_volume_fade_ticks = (*p).echo_volume_fade_ticks.wrapping_sub(1);
                        if (*p).echo_volume_fade_ticks as i32 == 0i32 {
                            (*p).echo_volume_left =
                                (((*p).echo_volume_fade_target_left as i32) << 8i32) as u16;
                            (*p).echo_volume_right =
                                (((*p).echo_volume_fade_target_right as i32) << 8i32) as u16;
                        }
                    }
                    if (*p).master_volume_fade_ticks != 0 {
                        (*p).master_volume_fade_ticks =
                            (*p).master_volume_fade_ticks.wrapping_sub(1);
                        (*p).master_volume = (if (*p).master_volume_fade_ticks as i32 == 0i32 {
                            ((*p).master_volume_fade_target as i32) << 8i32
                        } else {
                            (*p).master_volume as i32 + (*p).master_volume_fade_add_per_tick as i32
                        }) as u16;
                        (*p).vol_dirty = 0xffu8;
                    }
                    c = &raw mut (*p).channel as *mut Channel;
                    (*p).cur_chan_bit = 1u8;
                    loop {
                        if *(&raw mut (*c).pattern_order_ptr_for_chan as *mut u8).offset(
                            (size_of::<u16>() as usize)
                                .wrapping_div(size_of::<u8>() as usize)
                                .wrapping_sub(1_usize) as isize,
                        ) != 0
                        {
                            Chan_HandleTick(p, c);
                        }
                        c = c.offset(1);
                        (*p).cur_chan_bit = (((*p).cur_chan_bit as i32) << 1i32) as u8;
                        if ((*p).cur_chan_bit == 0) {
                            break;
                        }
                    }
                    return;
                }
            }
        }
        (*p).port_to_snes[0i32 as usize] = a;
        (*p).music_ptr_toplevel = *((&raw mut (*p).ram as *mut u8)
            .offset((0x5820i32 + (a as i32 - 1i32) * 2i32) as isize)
            as *mut u16);
        (*p).counter_sf0c = 2u16;
        (*p).key_OFF = ((*p).key_OFF | !((*p).is_chan_on));
    }
}
unsafe fn PlayNote(p: *mut SpcPlayer, c: *mut Channel, mut note: u8) {
    if note as i32 >= 0xcai32 {
        Channel_SetInstrument(p, c, note);
        note = 0xa4u8;
    }
    if note as i32 >= 0xc8i32 || (*p).is_chan_on & (*p).cur_chan_bit != 0 {
        return;
    }
    (*c).pitch = (((note as i32 & 0x7fi32)
        + (*p).global_transposition as i32
        + (*c).channel_transposition as i32)
        << 8i32
        | (*c).fine_tune as i32) as u16;
    (*c).vibrato_count = (((*c).vibrato_fade_num_ticks as i32) << 7i32) as u8;
    set_fields! {[(*c)], 0u8; vibrato_hold_count, vibrato_change_count, tremolo_count, tremolo_hold_count}
    (*p).vol_dirty = ((*p).vol_dirty | (*p).cur_chan_bit);
    (*p).key_ON = ((*p).key_ON | (*p).cur_chan_bit);
    (*c).pitch_slide_length = (*c).pitch_envelope_num_ticks;
    if (*c).pitch_slide_length != 0 {
        (*c).pitch_slide_delay_left = (*c).pitch_envelope_delay;
        if (*c).pitch_envelope_direction == 0 {
            (*c).pitch =
                ((*c).pitch as i32 - (((*c).pitch_envelope_slide_value as i32) << 8i32)) as u16;
        }
        ComputePitchAdd(
            c,
            (((*c).pitch as i32 >> 8i32) + (*c).pitch_envelope_slide_value as i32) as u8,
        );
    }
    WritePitch(p, c, (*c).pitch);
}
unsafe fn Spc_Loop_Part1(p: *mut SpcPlayer) {
    Dsp_Write(p, KOF as i32 as u8, (*p).key_OFF);
    Dsp_Write(p, PMON as i32 as u8, 0u8);
    Dsp_Write(p, NON as i32 as u8, (*p).reg_NON);
    Dsp_Write(p, KOF as i32 as u8, 0u8);
    Dsp_Write(p, KON as i32 as u8, (*p).key_ON);
    if (*p).echo_stored_time & 0x80 == 0 {
        Dsp_Write(p, FLG as i32 as u8, (*p).reg_FLG);
        if (*p).echo_stored_time as i32 == (*p).echo_parameter_EDL as i32 {
            Dsp_Write(p, EON as i32 as u8, (*p).reg_EON);
            Dsp_Write(p, EFB as i32 as u8, (*p).reg_EFB);
            Dsp_Write(p, EVOLR as i32 as u8, ((*p).echo_volume_right >> 8) as u8);
            Dsp_Write(p, EVOLL as i32 as u8, ((*p).echo_volume_left >> 8) as u8);
        }
    }
    (*p).key_ON = 0u8;
    (*p).key_OFF = (*p).key_ON;
}
unsafe fn Spc_Loop_Part2(p: *mut SpcPlayer, ticks: u8) {
    let mut t: i32 = (*p).sfx_timer_accum as i32 + (ticks as i32 * 0x20i32) as u8 as i32;
    (*p).sfx_timer_accum = t as u8;
    if t >= 0x100i32 {
        Sfx1_HandleCmdFromSnes(p);
        Sfx2_HandleCmdFromSnes(p);
        Sfx3_HandleCmdFromSnes(p);
        if (*p).echo_stored_time as i32 != (*p).echo_parameter_EDL as i32 {
            (*p).echo_stored_time = (*p).echo_stored_time.wrapping_add(1);
        }
    }
    t = (*p).main_tempo_accum as i32
        + (ticks as i32
            * *(&raw mut (*p).tempo as *mut u8).offset(
                (size_of::<u16>() as usize)
                    .wrapping_div(size_of::<u8>() as usize)
                    .wrapping_sub(1_usize) as isize,
            ) as i32) as u8 as i32;
    (*p).main_tempo_accum = t as u8;
    if t >= 256i32 {
        loop {
            Music_HandleCmdFromSnes(p);
            if ((*p).fast_forward == 0) {
                break;
            }
        }
    } else if (*p).port_to_snes[0i32 as usize] != 0 {
        let mut c = &raw mut (*p).channel as *mut Channel;
        (*p).cur_chan_bit = 1u8;
        while (*p).cur_chan_bit as i32 != 0i32 {
            if *(&raw mut (*c).pattern_order_ptr_for_chan as *mut u8).offset(
                (size_of::<u16>() as usize)
                    .wrapping_div(size_of::<u8>() as usize)
                    .wrapping_sub(1_usize) as isize,
            ) != 0
            {
                HandlePanAndSweep(p, c);
            }
            (*p).cur_chan_bit = (((*p).cur_chan_bit as i32) << 1i32) as u8;
            c = c.offset(1);
        }
    }
}
unsafe fn Vector_Reset_Spc(p: *mut SpcPlayer) {
    dsp_reset((*p).dsp);
    memset(
        (&raw mut (*p).ram as *mut u8).offset(0x500) as *mut c_void,
        0i32,
        0x1000 as size_t,
    );
    memset(
        (&raw mut (*p).ram as *mut u8).offset(0x20) as *mut c_void,
        0i32,
        0xf as size_t,
    );
    memset(
        (&raw mut (*p).ram as *mut u8).offset(0xd0) as *mut c_void,
        0i32,
        0x1f as size_t,
    );
    memset(
        (&raw mut (*p).ram as *mut u8).offset(0x391) as *mut c_void,
        0i32,
        0x6f as size_t,
    );
    memset(
        (&raw mut (*p).ram as *mut u8).offset(0x440) as *mut c_void,
        0i32,
        0x7f as size_t,
    );
    SpcPlayer_CopyVariablesFromRam(p);
    let mut i: i32 = 0i32;
    while i < 8i32 {
        (*p).channel[i as usize].index = i as u8;
        i += 1;
    }
    SetupEchoParameter_EDL(p, 1u8);
    (*p).reg_FLG = ((*p).reg_FLG | 0x20);
    Dsp_Write(p, MVOLL as i32 as u8, 0x60u8);
    Dsp_Write(p, MVOLR as i32 as u8, 0x60u8);
    Dsp_Write(p, DIR as i32 as u8, 0x6du8);
    *(&raw mut (*p).tempo as *mut u8).offset(
        (size_of::<u16>() as usize)
            .wrapping_div(size_of::<u8>() as usize)
            .wrapping_sub(1_usize) as isize,
    ) = 16u8;
    (*p).timer_cycles = 0u8;
}
pub unsafe fn Sfx_Chan_Reset(p: *mut SpcPlayer, sr: *mut SfxRegs, s: *mut SfxChan) {
    (*s).disable = 0xffu8;
    (*s).update_adsr = 0u8;
    (*sr).enabled_voices = ((*sr).enabled_voices & (*s).channel_mask);
    (*p).is_chan_on = ((*p).is_chan_on & (*s).channel_mask);
    (*p).cur_chan_bit = ((*p).cur_chan_bit | (*s).voice_bitset);
    (*p).key_OFF = ((*p).key_OFF | (*s).voice_bitset);
    let c = (&raw mut (*p).channel as *mut Channel).offset(((*s).voice_index >> 1) as isize);
    Channel_SetInstrument_NoSetId(p, c, (*c).instrument_id);
    (*c).final_volume = (*s).volume;
    (*c).pan_flag_with_phase_invert = (*s).phase_invert;
    if (*sr).enabled_voices == 0 {
        set_fields! {[(*sr)], 0u8; cur_sound, priority, init_flag}
    }
}
static kSfx1InstrListPtrs: [u16; 66] = [
    0x2b71, 0x2baf, 0x2bb7, 0x2bc4, 0x2bd1, 0x2bfc, 0x2c2f, 0x2c37, 0x2cff, 0x2d12, 0x2d1a, 0x2d27,
    0x2d4b, 0x2d5d, 0x2d5f, 0x2d76, 0x2d95, 0x2d97, 0x2d99, 0x2da6, 0x2da8, 0x2daa, 0x2dac, 0x2dc8,
    0x2de7, 0x2dfe, 0x2e00, 0x2e17, 0x2e19, 0x2e1b, 0x2e1d, 0x2e34, 0x2e36, 0x2e38, 0x2e3a, 0x2e68,
    0x2eaf, 0x2ed0, 0x2ef6, 0x2f2c, 0x2f37, 0x2f3f, 0x2f47, 0x2f4a, 0x2f4d, 0x2f50, 0x2fb0, 0x2fb8,
    0x2fc3, 0x2fdd, 0x2fe5, 0x3040, 0x3048, 0x3055, 0x305d, 0x3065, 0x3070, 0x3078, 0x307b, 0x308b,
    0x309b, 0x30a8, 0x30d6, 0x30e1, 0x312a, 0x313f,
];
static kSfx1Conf: [u8; 66] = [
    5, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 2, 0, 0, 2, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3,
    2, 2,
];
pub unsafe fn Sfx1_HandleCmdFromSnes(p: *mut SpcPlayer) {
    let a: u8 = (*p).input_ports[1i32 as usize];
    (*p).input_ports[1i32 as usize] = 255u8;
    if a as i32 == 255i32
        || a as i32 == 0i32
        || a as i32 != 1i32 && a as i32 != 2i32 && (*p).sfx1.priority as i32 != 0
    {
        if (*p).sfx1.cur_sound != 0 {
            Sfx1_Process(p);
        }
        return;
    }
    if (*p).sfx1.cur_sound != 0 {
        (*p).sfx1.enabled_voices = 0u8;
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(0),
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(1),
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(2),
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(3),
        );
    }
    let mut i: i32 = 0i32;
    while i != 4i32 {
        (*p).sfx_chans_1[i as usize].legato = 0u8;
        i += 1;
    }
    (*p).sfx1.cur_sound_index = ((a as i32 - 1i32) * 2i32) as u8;
    (*p).sfx1.cur_ptr = kSfx1InstrListPtrs[((*p).sfx1.cur_sound_index >> 1) as usize];
    (*p).sfx1.cur_sound = a;
    match kSfx1Conf[(a as i32 - 1i32) as usize] as i32 {
        0 => {
            (*p).sfx1.voices_to_setup = 1u8;
            (*p).sfx1.priority = 0u8;
        }
        1 => {
            set_fields! {[(*p).sfx1], 1u8; voices_to_setup, priority}
        }
        2 => {
            (*p).sfx1.voices_to_setup = 2u8;
            (*p).sfx1.priority = 0u8;
        }
        3 => {
            (*p).sfx1.voices_to_setup = 3u8;
            (*p).sfx1.priority = 1u8;
        }
        4 => {
            (*p).sfx1.voices_to_setup = 4u8;
            (*p).sfx1.priority = 0u8;
        }
        5 => {
            (*p).sfx1.voices_to_setup = 4u8;
            (*p).sfx1.priority = 0u8;
        }
        _ => {}
    };
}
static kSfx2InstrListPtrs: [u16; 127] = [
    0x3ab1, 0x3ac3, 0x3ad5, 0x3af1, 0x3af3, 0x3af5, 0x3b0c, 0x3b28, 0x3b2a, 0x3b3a, 0x3b42, 0x3b5e,
    0x3b73, 0x3b85, 0x3b92, 0x3ba9, 0x3bb4, 0x3bc1, 0x3be7, 0x3c08, 0x3c33, 0x3c3b, 0x3c43, 0x3c56,
    0x3c90, 0x3cff, 0x3d46, 0x3d4e, 0x3d65, 0x3d81, 0x3d9b, 0x3da8, 0x3dbf, 0x3dd6, 0x3ded, 0x3e04,
    0x3e20, 0x3e41, 0x3e66, 0x3e94, 0x3eb0, 0x3ecc, 0x3ece, 0x3f03, 0x3f18, 0x3f20, 0x3f44, 0x3f5e,
    0x3f75, 0x3f82, 0x3f8a, 0x3f97, 0x3f9f, 0x401f, 0x4036, 0x4066, 0x407c, 0x407e, 0x4086, 0x4088,
    0x408a, 0x408c, 0x409e, 0x40a6, 0x40b1, 0x40b4, 0x40bc, 0x40ce, 0x40d1, 0x40de, 0x40f5, 0x410c,
    0x412c, 0x4143, 0x415a, 0x4162, 0x416f, 0x41a9, 0x41c7, 0x41d9, 0x41ee, 0x421c, 0x4224, 0x4236,
    0x424d, 0x4268, 0x4288, 0x429a, 0x42af, 0x42bf, 0x42d4, 0x42d6, 0x42fe, 0x4310, 0x4322, 0x4334,
    0x433c, 0x4347, 0x4352, 0x43c1, 0x43cc, 0x43e8, 0x43fa, 0x4422, 0x442f, 0x4441, 0x4443, 0x4446,
    0x444e, 0x446f, 0x447f, 0x448f, 0x449c, 0x44a4, 0x44b9, 0x44ce, 0x44ed, 0x4598, 0x459a, 0x4618,
    0x462d, 0x4642, 0x4657, 0x466c, 0x4679, 0x4686, 0x468e,
];
static kSfx2Conf: [u8; 127] = [
    1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 2, 2, 0, 1, 0, 3, 1, 0,
    0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 3, 1, 3, 0, 0, 0, 0, 0, 0, 1, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 3, 3, 0, 0, 1, 0, 2, 0, 2, 3, 2, 0, 0, 0, 0, 0, 0,
    0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, 3, 3, 3, 3, 0, 3, 2, 2, 2, 2, 1, 1, 1, 2,
];
pub unsafe fn Sfx2_HandleCmdFromSnes(p: *mut SpcPlayer) {
    let a: u8 = (*p).input_ports[2i32 as usize];
    (*p).input_ports[2i32 as usize] = 255u8;
    if a as i32 == 255i32
        || a as i32 == 0i32
        || a as i32 != 0x71i32 && a as i32 != 0x7ei32 && (*p).sfx2.priority as i32 != 0
    {
        if (*p).sfx2.cur_sound != 0 {
            Sfx2_Process(p);
        }
        return;
    }
    if (*p).sfx2.cur_sound != 0 {
        (*p).sfx2.enabled_voices = 0u8;
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx2,
            (&raw mut (*p).sfx_chans_2 as *mut SfxChan).offset(0),
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx2,
            (&raw mut (*p).sfx_chans_2 as *mut SfxChan).offset(1),
        );
    }
    let mut i: i32 = 0i32;
    while i != 2i32 {
        (*p).sfx_chans_2[i as usize].legato = 0u8;
        i += 1;
    }
    (*p).sfx2.cur_sound_index = ((a as i32 - 1i32) * 2i32) as u8;
    (*p).sfx2.cur_ptr = kSfx2InstrListPtrs[((*p).sfx2.cur_sound_index >> 1) as usize];
    (*p).sfx2.cur_sound = a;
    match kSfx2Conf[(a as i32 - 1i32) as usize] as i32 {
        0 => {
            (*p).sfx2.voices_to_setup = 1u8;
            (*p).sfx2.priority = 0u8;
        }
        1 => {
            set_fields! {[(*p).sfx2], 1u8; voices_to_setup, priority}
        }
        2 => {
            (*p).sfx2.voices_to_setup = 2u8;
            (*p).sfx2.priority = 0u8;
        }
        3 => {
            (*p).sfx2.voices_to_setup = 2u8;
            (*p).sfx2.priority = 1u8;
        }
        _ => {}
    };
}
static kSfx3InstrListPtrs: [u16; 47] = [
    0x4eed, 0x4ef5, 0x4f00, 0x4f4b, 0x4f5b, 0x4f6b, 0x4f73, 0x4f89, 0x4f9f, 0x4fb6, 0x4fe1, 0x4ff7,
    0x4fff, 0x500c, 0x5047, 0x50a8, 0x50c4, 0x50d1, 0x50d4, 0x50f0, 0x511a, 0x5130, 0x5142, 0x5175,
    0x5178, 0x5188, 0x518b, 0x51b9, 0x51c1, 0x51d4, 0x51f0, 0x51fd, 0x5200, 0x5208, 0x5229, 0x5231,
    0x5239, 0x5241, 0x524e, 0x5256, 0x5277, 0x5293, 0x52a5, 0x52b0, 0x52c6, 0x52f1, 0x530b,
];
static kSfx3Conf: [u8; 47] = [
    0, 1, 2, 3, 3, 2, 4, 4, 2, 5, 3, 2, 2, 4, 3, 2, 2, 5, 2, 4, 4, 2, 2, 2, 4, 2, 4, 2, 2, 2, 2, 5,
    5, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 4, 2, 4, 2,
];
pub unsafe fn Sfx3_HandleCmdFromSnes(p: *mut SpcPlayer) {
    let a: u8 = (*p).input_ports[3i32 as usize];
    (*p).input_ports[3i32 as usize] = 255u8;
    if a as i32 == 255i32
        || a as i32 == 0i32
        || a as i32 != 1i32
            && ((*p).sfx3.some_value as i32 == 2i32
                || a as i32 != 2i32 && (*p).sfx3.priority as i32 != 0)
    {
        if (*p).sfx3.cur_sound != 0 {
            Sfx3_Process(p);
        }
        return;
    }
    if (*p).sfx3.cur_sound != 0 {
        (*p).sfx3.enabled_voices = 0u8;
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx3,
            (&raw mut (*p).sfx_chans_3 as *mut SfxChan).offset(0),
        );
        Sfx_Chan_Reset(
            p,
            &raw mut (*p).sfx3,
            (&raw mut (*p).sfx_chans_3 as *mut SfxChan).offset(1),
        );
    }
    let mut i: i32 = 0i32;
    while i != 2i32 {
        (*p).sfx_chans_3[i as usize].legato = 0u8;
        i += 1;
    }
    (*p).sfx3.cur_sound_index = ((a as i32 - 1i32) * 2i32) as u8;
    (*p).sfx3.cur_ptr = kSfx3InstrListPtrs[((*p).sfx3.cur_sound_index >> 1) as usize];
    (*p).sfx3.cur_sound = a;
    if (*p).sfx3.cur_sound as i32 >= 0xfei32 {
        Sfx3_Process(p);
        return;
    }
    match kSfx3Conf[(a as i32 - 1i32) as usize] as i32 {
        0 => {
            (*p).sfx3.voices_to_setup = 1u8;
            (*p).sfx3.some_value = 0u8;
            (*p).sfx3.priority = 1u8;
        }
        1 => {
            (*p).sfx3.voices_to_setup = 1u8;
            (*p).sfx3.some_value = 2u8;
        }
        2 => {
            (*p).sfx3.voices_to_setup = 1u8;
            (*p).sfx3.priority = 0u8;
        }
        3 => {
            (*p).sfx3.voices_to_setup = 2u8;
            (*p).sfx3.priority = 0u8;
        }
        4 => {
            (*p).sfx3.voices_to_setup = 2u8;
            (*p).sfx3.priority = 1u8;
        }
        5 => {
            set_fields! {[(*p).sfx3], 1u8; voices_to_setup, priority}
        }
        _ => {}
    };
}
pub unsafe fn SfxChan_GetByte(p: *mut SpcPlayer, s: *mut SfxChan) -> u8 {
    let fresh31 = (*s).ptr_idx;
    (*s).ptr_idx = (*s).ptr_idx.wrapping_add(1);
    (*p).ram[((*s).ptr as i32 + fresh31 as i32) as usize]
}
pub unsafe fn Sfx_RunChan(p: *mut SpcPlayer, sr: *mut SfxRegs, s: *mut SfxChan) {
    if (*s).disable as i32 == 0xffi32 {
        return;
    }
    (*s).instr_timer = (*s).instr_timer.wrapping_sub(1);
    if (*s).instr_timer as i32 == 0i32 {
        if (*s).legato == 0 {
            set_fields! {[(*s)], 0u8; enable_pitch_slide, subnote_delta, target_note}
            if (*s).release_flag as i32 != 0xffi32 {
                (*p).key_OFF = ((*p).key_OFF | (*s).voice_bitset);
                (*s).release_timer = 2u8;
                (*s).instr_timer = 1u8;
                (*s).release_flag = 0xffu8;
            }
            (*s).release_timer = (*s).release_timer.wrapping_sub(1);
            if (*s).release_timer != 0 {
                return;
            }
            (*s).release_flag = 0u8;
            (*p).cur_chan_bit = ((*p).cur_chan_bit & (*s).channel_mask);
            (*p).reg_NON = ((*p).reg_NON & (*s).channel_mask);
        }
        let mut a: u8 = 0;
        loop {
            a = SfxChan_GetByte(p, s);
            if a as i32 == 0xf9i32 {
                (*s).adsr_settings = SfxChan_GetByte(p, s) as u16;
                (*s).adsr_settings =
                    ((*s).adsr_settings as i32 | (SfxChan_GetByte(p, s) as i32) << 8i32) as u16;
                (*s).ptr_idx = ((*s).ptr_idx as i32 + 2i32) as u8;
                (*s).update_adsr = 0xffu8;
            } else {
                if a as i32 == 0xf5i32 || a as i32 == 0xf8i32 {
                    (*s).enable_pitch_slide_legato =
                        (if a as i32 == 0xf5i32 { 0xf5i32 } else { 0i32 }) as u8;
                    (*s).subnote_delta = SfxChan_GetByte(p, s);
                    (*s).target_note = SfxChan_GetByte(p, s);
                    (*s).enable_pitch_slide = 0xffu8;
                    a = SfxChan_GetByte(p, s);
                }
                if a as i32 == 0xffi32 {
                    Sfx_Chan_Reset(p, sr, s);
                    return;
                }
                if a as i32 == 0xfei32 {
                    (*s).repeat_ctr = SfxChan_GetByte(p, s);
                    (*s).repeat_pt = (*s).ptr_idx;
                    a = SfxChan_GetByte(p, s);
                }
                if a as i32 == 0xfbi32 || a as i32 == 0xfdi32 {
                    if a as i32 != 0xfbi32 && {
                        (*s).repeat_ctr = (*s).repeat_ctr.wrapping_sub(1);
                        (*s).repeat_ctr as i32 == 0i32
                    } {
                        continue;
                    }
                    (*s).ptr_idx = (*s).repeat_pt;
                    a = SfxChan_GetByte(p, s);
                }
                if (a as i32 != 0xfci32) {
                    break;
                }
                (*p).reg_NON = ((*p).reg_NON | (*s).voice_bitset);
            }
        }
        let c = (&raw mut (*p).channel as *mut Channel).offset(((*s).voice_index >> 1) as isize);
        Channel_SetInstrument_NoSetId(p, c, a);
        (*c).final_volume = SfxChan_GetByte(p, s);
        (*c).pan_flag_with_phase_invert = 0u8;
        WriteVolumeToDsp(p, c, ((SfxChan_GetByte(p, s) as i32) << 8i32) as u16);
        a = SfxChan_GetByte(p, s);
        if a as i32 != 0xf6i32 {
            (*s).note = a;
            (*s).subnote = 0u8;
        }
        WritePitchInner(
            p,
            c,
            (((*s).note as i32) << 8i32 | (*s).subnote as i32) as u16,
        );
        (*s).instr_timer = SfxChan_GetByte(p, s);
        if (*s).update_adsr != 0 {
            Dsp_Write(
                p,
                ((*s).voice_index_mult8 as i32 | V0ADSR1 as i32) as u8,
                (*s).adsr_settings as u8,
            );
            Dsp_Write(
                p,
                ((*s).voice_index_mult8 as i32 | V0ADSR2 as i32) as u8,
                ((*s).adsr_settings >> 8) as u8,
            );
        }
        if (*s).legato == 0 {
            (*p).key_ON = ((*p).key_ON | (*s).voice_bitset);
        }
    }
    if (*s).enable_pitch_slide as i32 != 0xffi32 {
        return;
    }
    if (*s).enable_pitch_slide_legato != 0 {
        (*s).legato = 0xffu8;
    }
    if (*s).note as i32 >= (*s).target_note as i32 {
        let t: i32 = (*s).subnote as i32 - (*s).subnote_delta as i32;
        (*s).subnote = t as u8;
        if t & 256i32 != 0 && {
            (*s).note = (*s).note.wrapping_sub(1);
            (*s).note as i32 == (*s).target_note as i32
        } {
            (*s).legato = 0u8;
            (*s).enable_pitch_slide = (*s).legato;
        }
    } else {
        let t_0: i32 = (*s).subnote as i32 + (*s).subnote_delta as i32;
        (*s).subnote = t_0 as u8;
        if t_0 & 256i32 != 0 && {
            (*s).note = (*s).note.wrapping_add(1);
            (*s).note as i32 == (*s).target_note as i32
        } {
            (*s).legato = 0u8;
            (*s).enable_pitch_slide = (*s).legato;
        }
    }
    WritePitchInner(
        p,
        (&raw mut (*p).channel as *mut Channel).offset(((*s).voice_index >> 1) as isize),
        (((*s).note as i32) << 8i32 | (*s).subnote as i32) as u16,
    );
}
pub unsafe fn Sfx1_Process(p: *mut SpcPlayer) {
    if (*p).sfx1.init_flag as i32 != 0xffi32 {
        Sfx1_Init(p);
        let mut i: i32 = 0i32;
        while i < 4i32 {
            let s = (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(i as isize);
            (*s).ptr = *((&raw mut (*p).ram as *mut u8)
                .offset(((*p).sfx1.cur_ptr as i32 + i * 2i32) as isize)
                as *mut u16);
            (*s).voice_index_mult8 = ((*s).voice_index as i32 * 8i32) as u8;
            (*s).ptr_idx = 0u8;
            (*s).instr_timer = 1u8;
            i += 1;
        }
    }
    let mut i_0: i32 = 0i32;
    while i_0 < 4i32 {
        Sfx_RunChan(
            p,
            &raw mut (*p).sfx1,
            (&raw mut (*p).sfx_chans_1 as *mut SfxChan).offset(i_0 as isize),
        );
        i_0 += 1;
    }
}
pub unsafe fn Sfx2_Process(p: *mut SpcPlayer) {
    if (*p).sfx2.init_flag as i32 != 0xffi32 {
        Sfx2_Init(p);
        let mut i: i32 = 0i32;
        while i < 2i32 {
            let s = (&raw mut (*p).sfx_chans_2 as *mut SfxChan).offset(i as isize);
            (*s).ptr = *((&raw mut (*p).ram as *mut u8)
                .offset(((*p).sfx2.cur_ptr as i32 + i * 2i32) as isize)
                as *mut u16);
            (*s).voice_index_mult8 = ((*s).voice_index as i32 * 8i32) as u8;
            (*s).ptr_idx = 0u8;
            (*s).instr_timer = 1u8;
            i += 1;
        }
    }
    let mut i_0: i32 = 0i32;
    while i_0 < 2i32 {
        Sfx_RunChan(
            p,
            &raw mut (*p).sfx2,
            (&raw mut (*p).sfx_chans_2 as *mut SfxChan).offset(i_0 as isize),
        );
        i_0 += 1;
    }
}
pub unsafe fn Sfx3_Process(p: *mut SpcPlayer) {
    if (*p).sfx3.init_flag as i32 != 0xffi32 {
        Sfx3_Init(p);
        let mut i: i32 = 0i32;
        while i < 2i32 {
            let s = (&raw mut (*p).sfx_chans_3 as *mut SfxChan).offset(i as isize);
            (*s).ptr = *((&raw mut (*p).ram as *mut u8)
                .offset(((*p).sfx3.cur_ptr as i32 + i * 2i32) as isize)
                as *mut u16);
            (*s).voice_index_mult8 = ((*s).voice_index as i32 * 8i32) as u8;
            (*s).ptr_idx = 0u8;
            (*s).instr_timer = 1u8;
            i += 1;
        }
    }
    let mut i_0: i32 = 0i32;
    while i_0 < 2i32 {
        Sfx_RunChan(
            p,
            &raw mut (*p).sfx3,
            (&raw mut (*p).sfx_chans_3 as *mut SfxChan).offset(i_0 as isize),
        );
        i_0 += 1;
    }
}
pub unsafe fn Sfx_Init(
    p: *mut SpcPlayer,
    sr: *mut SfxRegs,
    channels: *mut SfxChan,
    which_sfx: i32,
) {
    let num: i32 = if which_sfx == 0i32 { 4i32 } else { 2i32 };
    static kSomeAddrs: [[u16; 3]; 3] = [
        [0x3a6, 0x3aa, 0x3af],
        [0x446, 0x448, 0x44b],
        [0x47e, 0x480, 0x483],
    ];
    (*sr).voice_id = 9u8;
    (*sr).voices_remaining = (*p).is_chan_on;
    (*sr).init_flag = 0xffu8;
    (*sr).chan_idx_x2 = 0u8;
    (*sr).chan_idx = (*sr).chan_idx_x2;
    let mut i: i32 = 0i32;
    while i < num {
        set_fields! {[(*channels.offset(i as isize))], 0u8; voice_bitset, voice_index}
        set_fields! {[(*channels.offset(i as isize))], 0xffu8; channel_mask, disable}
        i += 1;
    }
    loop {
        (*sr).voice_id = (*sr).voice_id.wrapping_sub(1);
        if ((*sr).voice_id == 0) {
            break;
        }
        let t: i32 = (*sr).voices_remaining as i32;
        (*sr).voices_remaining = (((*sr).voices_remaining as i32) << 1i32) as u8;
        if t & 0x80i32 == 0 {
            if (*sr).voices_to_setup == 0 {
                return;
            }
            (*sr).voices_to_setup = (*sr).voices_to_setup.wrapping_sub(1);
            let c = channels.offset((*sr).chan_idx as isize);
            (*c).disable = 0u8;
            (*sr).chan_voice_bitset_ptr = (kSomeAddrs[which_sfx as usize][0i32 as usize] as i32
                + (*sr).chan_idx as i32) as u16;
            (*sr).chan_voice_mask_ptr = (kSomeAddrs[which_sfx as usize][1i32 as usize] as i32
                + (*sr).chan_idx as i32) as u16;
            (*sr).chan_voice_index_ptr = (kSomeAddrs[which_sfx as usize][2i32 as usize] as i32
                + (*sr).chan_idx as i32) as u16;
            (*sr).chan_idx = ((*sr).chan_idx as i32 + 1i32) as u8;
            (*sr).chan_idx_x2 = ((*sr).chan_idx_x2 as i32 + 2i32) as u8;
            (*sr).voice_index = (((*sr).voice_id as i32 - 1i32) * 2i32) as u8;
            (*c).voice_index = (*sr).voice_index;
            let t_0: i32 = (*sr).voice_id as i32 - 1i32;
            (*c).volume = ((*p).channel[t_0 as usize].final_volume as i32
                | ((*p).channel[t_0 as usize].pan_flag_with_phase_invert as i32) << 8i32)
                as u8;
            let m: i32 = (1i32) << t_0;
            (*p).is_chan_on = ((*p).is_chan_on as i32 | m) as u8;
            (*p).cur_chan_bit = ((*p).cur_chan_bit as i32 & !m) as u8;
            (*p).reg_EON = ((*p).reg_EON as i32 & !m) as u8;
            (*sr).enabled_voices = ((*sr).enabled_voices as i32 | m) as u8;
            (*c).voice_bitset = m as u8;
            (*c).channel_mask = !m as u8;
        }
    }
}
crate::forward_calls! { pub unsafe fn (p: *mut SpcPlayer);
    Sfx1_Init => Sfx_Init(
        p,
        &raw mut (*p).sfx1,
        &raw mut (*p).sfx_chans_1 as *mut SfxChan,
        0i32,
    );
    Sfx2_Init => Sfx_Init(
        p,
        &raw mut (*p).sfx2,
        &raw mut (*p).sfx_chans_2 as *mut SfxChan,
        1i32,
    );
    Sfx3_Init => Sfx_Init(
        p,
        &raw mut (*p).sfx3,
        &raw mut (*p).sfx_chans_3 as *mut SfxChan,
        2i32,
    );
}
pub unsafe fn SpcPlayer_Create() -> *mut SpcPlayer {
    let p = malloc(size_of::<SpcPlayer>() as size_t) as *mut SpcPlayer;
    memset(p as *mut c_void, 0i32, size_of::<SpcPlayer>() as size_t);
    (*p).dsp = dsp_init(&raw mut (*p).ram as *mut u8);
    (*p).reg_write_history = null_mut::<DspRegWriteHistory>();
    p
}
pub unsafe fn SpcPlayer_Initialize(p: *mut SpcPlayer) {
    Vector_Reset_Spc(p);
    Spc_Loop_Part1(p);
}
pub unsafe fn SpcPlayer_CopyVariablesToRam(p: *mut SpcPlayer) {
    let mut c = &raw mut (*p).channel as *mut Channel;
    let mut i: i32 = 0i32;
    while i < 8i32 {
        let mut m = (&raw const kChannel_Maps as *const MemMap).offset(0);
        while m
            != (&raw const kChannel_Maps as *const MemMap).offset(
                (size_of::<[MemMap; 47]>() as usize).wrapping_div(size_of::<MemMap>() as usize)
                    as isize,
            )
        {
            memcpy(
                (&raw mut (*p).ram as *mut u8)
                    .offset((((*m).org_off as i32 & 0x7fffi32) + i * 2i32) as isize)
                    as *mut c_void,
                (c as *mut u8).offset((*m).off as i32 as isize) as *const c_void,
                (if (*m).org_off & 0x8000 != 0 {
                    2i32
                } else {
                    1i32
                }) as size_t,
            );
            m = m.offset(1);
        }
        i += 1;
        c = c.offset(1);
    }
    let mut m_0 = (&raw const kSpcPlayer_Maps as *const MemMapSized).offset(0);
    while m_0
        != (&raw const kSpcPlayer_Maps as *const MemMapSized).offset(
            (size_of::<[MemMapSized; 283]>() as usize)
                .wrapping_div(size_of::<MemMapSized>() as usize) as isize,
        )
    {
        memcpy(
            (&raw mut (*p).ram as *mut u8).offset((*m_0).org_off as isize) as *mut c_void,
            (p as *mut u8).offset((*m_0).off as i32 as isize) as *const c_void,
            (*m_0).size as size_t,
        );
        m_0 = m_0.offset(1);
    }
}
pub unsafe fn SpcPlayer_CopyVariablesFromRam(p: *mut SpcPlayer) {
    let mut c = &raw mut (*p).channel as *mut Channel;
    let mut i: i32 = 0i32;
    while i < 8i32 {
        let mut m = (&raw const kChannel_Maps as *const MemMap).offset(0);
        while m
            != (&raw const kChannel_Maps as *const MemMap).offset(
                (size_of::<[MemMap; 47]>() as usize).wrapping_div(size_of::<MemMap>() as usize)
                    as isize,
            )
        {
            memcpy(
                (c as *mut u8).offset((*m).off as i32 as isize) as *mut c_void,
                (&raw mut (*p).ram as *mut u8)
                    .offset((((*m).org_off as i32 & 0x7fffi32) + i * 2i32) as isize)
                    as *const c_void,
                (if (*m).org_off & 0x8000 != 0 {
                    2i32
                } else {
                    1i32
                }) as size_t,
            );
            m = m.offset(1);
        }
        i += 1;
        c = c.offset(1);
    }
    let mut m_0 = (&raw const kSpcPlayer_Maps as *const MemMapSized).offset(0);
    while m_0
        != (&raw const kSpcPlayer_Maps as *const MemMapSized).offset(
            (size_of::<[MemMapSized; 283]>() as usize)
                .wrapping_div(size_of::<MemMapSized>() as usize) as isize,
        )
    {
        memcpy(
            (p as *mut u8).offset((*m_0).off as i32 as isize) as *mut c_void,
            (&raw mut (*p).ram as *mut u8).offset((*m_0).org_off as isize) as *const c_void,
            (*m_0).size as size_t,
        );
        m_0 = m_0.offset(1);
    }
    let mut i_0: i32 = 0i32;
    while i_0 < 8i32 {
        (*p).channel[i_0 as usize].index = i_0 as u8;
        i_0 += 1;
    }
}
pub unsafe fn SpcPlayer_GenerateSamples(p: *mut SpcPlayer) {
    crate::native_assertion! {
        '_c2rust_label: (*p).timer_cycles as i32 <= 64i32;
        b"p->timer_cycles <= 64\0", b"src/spc_player.c\0", 1550u32,
        b"void SpcPlayer_GenerateSamples(SpcPlayer *)\0"
    };
    crate::native_assertion! {
        '_c2rust_label_0: (*(*p).dsp).sampleOffset as i32 <= 534i32;
        b"p->dsp->sampleOffset <= 534\0", b"src/spc_player.c\0", 1552u32,
        b"void SpcPlayer_GenerateSamples(SpcPlayer *)\0"
    };
    loop {
        if (*p).timer_cycles as i32 >= 64i32 {
            Spc_Loop_Part2(p, ((*p).timer_cycles >> 6));
            Spc_Loop_Part1(p);
            (*p).timer_cycles = ((*p).timer_cycles & 63);
        }
        let mut n: i32 = 534i32 - (*(*p).dsp).sampleOffset as i32;
        if n > 64i32 - (*p).timer_cycles as i32 {
            n = 64i32 - (*p).timer_cycles as i32;
        }
        (*p).timer_cycles = ((*p).timer_cycles as i32 + n) as u8;
        let mut i: i32 = 0i32;
        while i < n {
            dsp_cycle((*p).dsp);
            i += 1;
        }
        if (*(*p).dsp).sampleOffset as i32 == 534i32 {
            break;
        }
    }
}
pub unsafe fn SpcPlayer_Upload(p: *mut SpcPlayer, mut data: *const u8) {
    Dsp_Write(p, EVOLL as i32 as u8, 0u8);
    Dsp_Write(p, EVOLR as i32 as u8, 0u8);
    Dsp_Write(p, KOF as i32 as u8, 0xffu8);
    loop {
        let mut numbytes: i32 = *(data as *mut u16) as i32;
        if numbytes == 0i32 {
            break;
        }
        let mut target: i32 = *(data.offset(2) as *mut u16) as i32;
        data = data.offset(4);
        loop {
            let fresh32 = data;
            data = data.offset(1);
            let fresh33 = target;
            target = target + 1;
            (*p).ram[(fresh33 & 0xffffi32) as usize] = *fresh32;
            numbytes -= 1;
            if (numbytes == 0) {
                break;
            }
        }
    }
    (*p).port_to_snes[0i32 as usize] = 0u8;
    (*p).input_ports[3i32 as usize] = 255u8;
    (*p).input_ports[2i32 as usize] = (*p).input_ports[3i32 as usize];
    (*p).input_ports[1i32 as usize] = (*p).input_ports[2i32 as usize];
    (*p).input_ports[0i32 as usize] = (*p).input_ports[1i32 as usize];
    (*p).music_ptr_toplevel = *((&raw mut (*p).ram as *mut u8).offset(0x581e) as *mut u16);
    (*p).counter_sf0c = 2u16;
    (*p).key_OFF = ((*p).key_OFF | !((*p).is_chan_on));
}
