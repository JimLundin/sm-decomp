//! Small per-enemy tables in banks $A8 and $B3: instruction lists, AI
//! functions and glow colours, each indexed by a facing, kind or phase.

/// $A8:E380: Wrecked Ship orb instruction lists, by starting direction.
pub const ORB_INSTRUCTION_LISTS: u32 = 0xa8_e380;
pub const ORB_DIRECTIONS: usize = 4;

/// $A8:E682: Wrecked Ship spark instruction lists, by spark kind.
pub const SPARK_INSTRUCTION_LISTS: u32 = 0xa8_e682;
/// $A8:E688: Wrecked Ship spark AI functions, by spark kind.
pub const SPARK_FUNCTIONS: u32 = 0xa8_e688;
pub const SPARK_KINDS: usize = 3;

/// $A8:E7CC: the blue face block's glow, eight phases of four colours.
pub const FACE_BLOCK_GLOW_COLOURS: u32 = 0xa8_e7cc;
pub const FACE_BLOCK_GLOW_WORDS: usize = 8 * 4;

/// $A8:F3B0: Ki-Hunter wing instruction lists, two pairs of left and right wings.
pub const KI_HUNTER_WING_INSTRUCTION_LISTS: u32 = 0xa8_f3b0;
pub const KI_HUNTER_WING_LISTS: usize = 4;

/// $A8:C599: Maridia floater spike instruction lists.
pub const FLOATER_SPIKE_INSTRUCTION_LISTS: u32 = 0xa8_c599;
pub const FLOATER_SPIKE_LISTS: usize = 3;

/// $A8:90DA: morph ball eye instruction lists, by facing.
pub const EYE_INSTRUCTION_LISTS: u32 = 0xa8_90da;
pub const EYE_FACINGS: usize = 4;

/// $B3:882B: Brinstar pipe bug instruction lists, by animation phase.
pub const PIPE_BUG_INSTRUCTION_LISTS: u32 = 0xb3_882b;
/// $B3:8833: the same for pipe bugs whose first parameter is set.
pub const PIPE_BUG_ALTERNATE_INSTRUCTION_LISTS: u32 = 0xb3_8833;
pub const PIPE_BUG_PHASES: usize = 4;
