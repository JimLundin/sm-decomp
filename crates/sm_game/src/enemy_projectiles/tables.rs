//! Projectile content tables retained byte-for-byte.
#![forbid(unsafe_code)]
pub static CROCOMIRE_SPIKE_WALL_PIECES_TAB2: [u16; 18] = [
    0, 0, 0xff0, 0xee0, 0xcc0, 0xaa0, 0x880, 0x660, 0x440, 0x220, 0xff0, 0xee0, 0xcc0, 0xaa0,
    0x880, 0x660, 0x440, 0x220,
];
pub static CROCOMIRE_SPIKE_WALL_PIECES_TAB1: [u16; 18] = [
    0, 0, 0xff00, 0xee00, 0xcc00, 0xaa00, 0x8800, 0x6600, 0x4400, 0x2200, 0xff00, 0xee00, 0xcc00,
    0xaa00, 0x8800, 0x6600, 0x4400, 0x2200,
];
pub static CROCOMIRE_SPIKE_WALL_PIECES_TAB3: [i8; 36] = [
    0, 0, 0, 0, 4, 0, 4, 0, 3, 0, 3, 0, 2, 0, 2, 0, 1, 0, 1, 0, 6, 0, 5, 0, 4, 0, 3, 0, 2, 0, 2, 0,
    1, 0, 1, 0,
];
pub static K_EPROJ_INIT_9634_XVEL: [i16; 4] = [-0x200i16, -0x1f0i16, -0x1bci16, -0x16ai16];
pub static K_EPROJ_INIT_9634_YVEL: [i16; 4] = [0, 0x88, 0xfc, 0x16a];
pub static BYTE_8698_B4: [u8; 16] = [
    0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0xf0, 0xe0, 0xd0, 0xc0, 0xb0, 0xa0, 0x90, 0x80,
];
pub static BYTE_8698_F7: [u8; 9] = [0x30, 0x44, 0x58, 0x6c, 0x80, 0x94, 0xa8, 0xbc, 0xd0];
pub static BYTE_869979: [u8; 8] = [0, 0x20, 0x40, 0x60, 0x80, 0xa0, 0xc0, 0xe0];
pub static WORD_869_A3_E: [u16; 3] = [0xfd00, 0xfe00, 0xff00];
pub static WORD_86_A2_D6: [i16; 6] = [64, 72, 80, -64i16, -72i16, -80i16];
pub static K_EPROJ_INIT_BOMB_TORIZO_EXPLOSIVE_SWIPE_TAB0: [i16; 11] = [
    -30i16, -40i16, -47i16, -31i16, -21i16, -1i16, -28i16, -43i16, -48i16, -31i16, -21i16,
];
pub static K_EPROJ_INIT_BOMB_TORIZO_EXPLOSIVE_SWIPE_TAB1: [i16; 11] = [
    -52i16, -28i16, -11i16, 9, 21, 20, -52i16, -27i16, -10i16, 9, 20,
];
pub static K_EPROJ_INIT_BOMB_TORIZO_STATUE_BREAKING_XPOS: [i16; 16] = [
    8, 0x18, -8i16, 8, 0x18, -8i16, 8, 0x18, 8, -8i16, 0x18, 8, -8i16, 0x18, 8, -8i16,
];
pub static K_EPROJ_INIT_BOMB_TORIZO_STATUE_BREAKING_YPOS: [i16; 8] =
    [-8i16, -8i16, 8, 8, 8, 0x18, 0x18, 0x18];
pub static K_EPROJ_INIT_BOMB_TORIZO_STATUE_BREAKING_YVEL: [i16; 8] =
    [256, 256, 256, 256, 256, 256, 256, 256];
pub static K_EPROJ_INIT_BOMB_TORIZO_STATUE_BREAKING_F: [i16; 8] = [16, 16, 16, 16, 16, 16, 16, 16];
pub static K_EPROJ_INIT_BOMB_TORIZO_LOW_HEALTH_EXPLODE_X: [i16; 6] = [0, 12, -12i16, 0, 16, -16i16];
pub static K_EPROJ_INIT_BOMB_TORIZO_LOW_HEALTH_EXPLODE_Y: [i16; 6] =
    [-8i16, -8i16, -8i16, -20i16, -20i16, -20i16];
pub static WORD_86_B205: [i16; 2] = [-0x1ei16, 0x1e];
pub static WORD_86_B65_B: [i16; 20] = [
    -16i16, 16, -96i16, -64i16, -128i16, -32i16, -96i16, 64, -128i16, 32, 16, 16, 96, -64i16, 112,
    -64i16, 128, -64i16, 144, -64i16,
];
pub static WORD_86_B6_B1: [i16; 4] = [-64i16, 512, 64, 512];
pub static K_EPROJ_INIT_TOURIAN_STATUE_EYE_GLOW_X: [u16; 4] = [0x84, 0x7a, 0x9e, 0x68];
pub static K_EPROJ_INIT_TOURIAN_STATUE_EYE_GLOW_Y: [u16; 4] = [0x90, 0x51, 0x80, 0x72];
pub static K_EPROJ_INIT_TOURIAN_STATUE_EYE_GLOW_COLORS: [u16; 16] = [
    0x6bff, 0x33b, 0x216, 0x173, 0x7f5f, 0x7c1f, 0x5816, 0x300c, 0x7f5a, 0x7ec0, 0x6de0, 0x54e0,
    0x6bfa, 0x3be0, 0x2680, 0x1580,
];
pub static K_EPROJ_INIT_SHAKTOOL_ATTACK_MIDDLE_BACK_CIRCLE_X: [i16; 8] =
    [0, 12, 16, 12, 0, -12i16, -16i16, -12i16];
pub static K_EPROJ_INIT_SHAKTOOL_ATTACK_MIDDLE_BACK_CIRCLE_Y: [i16; 8] =
    [-16i16, -12i16, 0, 12, 16, 12, 0, -12i16];
pub static K_EPROJ_INIT_MOTHER_BRAIN_ROOM_TURRETS_X: [i16; 12] = [
    0x398, 0x348, 0x328, 0x2d8, 0x288, 0x268, 0x218, 0x1c8, 0x1a8, 0x158, 0x108, 0xe8,
];
pub static EPROJ_INIT_MOTHER_BRAIN_ROOM_TURRETS_Y: [i16; 12] = [
    0x30, 0x40, 0x40, 0x30, 0x40, 0x40, 0x30, 0x40, 0x40, 0x30, 0x40, 0x40,
];
pub static K_EPROJ_MOTHER_BRAIN_ROOM_TURRET_BULLETS_X: [i16; 8] =
    [-17i16, -12i16, 0, 12, 17, 12, 0, -12i16];
pub static K_EPROJ_MOTHER_BRAIN_ROOM_TURRET_BULLETS_Y: [i16; 8] =
    [-9i16, 3, 7, 3, -9i16, -19i16, -21i16, -19i16];
pub static K_EPROJ_MOTHER_BRAIN_ROOM_TURRET_BULLETS_XVEL: [i16; 8] =
    [-704i16, -498i16, 0, 498, 704, 498, 0, -498i16];
pub static K_EPROJ_MOTHER_BRAIN_ROOM_TURRET_BULLETS_YVEL: [i16; 8] =
    [0, 498, 704, 498, 0, -498i16, -704i16, -498i16];
pub static K_MOTHER_BRAINS_BOMB_YACCEL: [i16; 10] =
    [7, 0x10, 0x20, 0x40, 0x70, 0xb0, 0xf0, 0x130, 0x170, 0];
pub static K_EPROJ_INIT_MOTHER_BRAINS_DROOL: [i16; 12] =
    [6, 0x14, 0xe, 0x12, 8, 0x17, 0xa, 0x13, 0xb, 0x19, 0xc, 0x12];
pub static K_EPROJ_INIT_MOTHER_BRAIN_GLASS_SHATTERING_SHARD_X: [i16; 3] = [8, -40i16, -16i16];
pub static K_EPROJ_INIT_MOTHER_BRAIN_GLASS_SHATTERING_SHARD_Y: [i16; 3] = [32, 32, 32];
pub static WORD_86_D082: u16 = 0xe0;
pub static WORD_86_D21_A: [i16; 8] = [0, 32, 32, 32, 0, -32i16, -32i16, -32i16];
pub static WORD_86_D22_A: [i16; 8] = [-32i16, -32i16, 0, 32, 32, 32, 0, -32i16];
pub static K_EPROJ_INIT_N00B_TUBE_SHARDS_X: [i16; 10] = [
    -56i16, -64i16, -20i16, -40i16, -64i16, -48i16, -24i16, -40i16, 0, -8i16,
];
pub static K_EPROJ_INIT_N00B_TUBE_SHARDS_Y: [i16; 10] =
    [8, -12i16, -26i16, -24i16, -32i16, 28, 16, -8i16, -24i16, 16];
pub static K_EPROJ_INIT_N00B_TUBE_SHARDS_XVEL: [i16; 10] = [
    -384i16, -384i16, -160i16, -288i16, -288i16, -320i16, -96i16, -352i16, 0, -64i16,
];
pub static K_EPROJ_INIT_N00B_TUBE_SHARDS_YVEL: [i16; 10] = [
    320, -256i16, -416i16, -288i16, -288i16, 448, 576, -96i16, -288i16, 384,
];
pub static K_EPROJ_INIT_N00B_TUBE_RELEASED_AIR_BUBBLES_X: [u16; 6] = [40, 80, 104, 120, 152, 184];
pub static K_EPROJ_INIT_N00B_TUBE_RELEASED_AIR_BUBBLES_Y: [u16; 6] = [80, 72, 84, 32, 64, 84];
pub static WORD_86_DCB9: [i16; 5] = [-64i16, -56i16, -48i16, -40i16, -32i16];
pub static K_EPROJ_INIT_SPORE_SPAWNERS_X: [i16; 4] = [0x20, 0x60, 0xa0, 0xe0];
