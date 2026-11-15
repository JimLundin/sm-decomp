//! Independently observed exact door instruction identities and fixed return layouts.

use super::{Address, Layout, Leaf, Return, StackBytes};

pub(super) const LEAVES: &[Leaf] = &[
    Leaf {
        instructions: &[Address(0x82_E526), Address(0x82_E529)],
        layout: Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    },
    Leaf {
        instructions: &[
            Address(0x82_DA21),
            Address(0x82_DA24),
            Address(0x82_DA27),
            Address(0x82_DA2A),
            Address(0x82_DA2C),
            Address(0x82_DA2D),
            Address(0x82_DA30),
            Address(0x82_DA31),
            Address(0x82_DA34),
            Address(0x82_DA37),
            Address(0x82_DA3A),
            Address(0x82_DA3D),
            Address(0x82_DA3E),
            Address(0x82_DA3F),
            Address(0x82_DA42),
            Address(0x82_DA44),
            Address(0x82_DA47),
        ],
        layout: Layout {
            saved: StackBytes(1),
            returning: Return::Near,
        },
    },
    Leaf {
        instructions: &[Address(0x82_DA4C)],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Near,
        },
    },
    Leaf {
        instructions: &[Address(0x82_DA4D)],
        layout: Layout {
            saved: StackBytes(6),
            returning: Return::Near,
        },
    },
    Leaf {
        instructions: &[
            Address(0x82_DA4E),
            Address(0x82_DA50),
            Address(0x82_DA53),
            Address(0x82_DA54),
            Address(0x82_DA56),
            Address(0x82_DA59),
            Address(0x82_DA5A),
            Address(0x82_DA5C),
            Address(0x82_DA5F),
            Address(0x82_DA61),
            Address(0x82_DA63),
            Address(0x82_DA64),
            Address(0x82_DA65),
            Address(0x82_DA66),
            Address(0x82_DA67),
            Address(0x82_DA6A),
            Address(0x82_DA6B),
            Address(0x82_DA6D),
            Address(0x82_DA6E),
            Address(0x82_DA6F),
            Address(0x82_DA70),
            Address(0x82_DA71),
            Address(0x82_DA74),
            Address(0x82_DA75),
            Address(0x82_DA77),
            Address(0x82_DA7A),
            Address(0x82_DA7B),
            Address(0x82_DA7C),
            Address(0x82_DA7D),
            Address(0x82_DA7E),
            Address(0x82_DA7F),
            Address(0x82_DA81),
            Address(0x82_DA83),
            Address(0x82_DA85),
            Address(0x82_DA86),
            Address(0x82_DA87),
            Address(0x82_DA88),
            Address(0x82_DA8B),
            Address(0x82_DA8C),
            Address(0x82_DA8E),
            Address(0x82_DA8F),
            Address(0x82_DA90),
            Address(0x82_DA91),
            Address(0x82_DA94),
            Address(0x82_DA95),
            Address(0x82_DA97),
            Address(0x82_DA9A),
            Address(0x82_DA9B),
            Address(0x82_DA9C),
            Address(0x82_DA9D),
            Address(0x82_DA9F),
            Address(0x82_DAA1),
        ],
        layout: Layout {
            saved: StackBytes(8),
            returning: Return::Near,
        },
    },
    Leaf {
        instructions: &[
            Address(0x82_DAA5),
            Address(0x82_DAA6),
            Address(0x82_DAA9),
            Address(0x82_DAAB),
            Address(0x82_DAAC),
            Address(0x82_DAAD),
            Address(0x82_DAAE),
            Address(0x82_DAB1),
            Address(0x82_DAB3),
            Address(0x82_DAB4),
            Address(0x82_DAB5),
        ],
        layout: Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    },
    Leaf {
        instructions: &[
            Address(0x82_DAB6),
            Address(0x82_DAB7),
            Address(0x82_DAB9),
            Address(0x82_DABA),
            Address(0x82_DABB),
            Address(0x82_DABD),
            Address(0x82_DABF),
            Address(0x82_DAC1),
            Address(0x82_DAC4),
            Address(0x82_DAC5),
            Address(0x82_DAC6),
            Address(0x82_DAC9),
            Address(0x82_DACD),
            Address(0x82_DACF),
            Address(0x82_DAD2),
            Address(0x82_DAD4),
            Address(0x82_DAD5),
            Address(0x82_DAD9),
            Address(0x82_DADB),
            Address(0x82_DADC),
            Address(0x82_DADD),
            Address(0x82_DADE),
            Address(0x82_DADF),
            Address(0x82_DAE3),
            Address(0x82_DAE5),
            Address(0x82_DAE7),
            Address(0x82_DAEA),
            Address(0x82_DAEB),
            Address(0x82_DAED),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Near,
        },
    },
    Leaf {
        instructions: &[
            Address(0x82_DAEE),
            Address(0x82_DAEF),
            Address(0x82_DAF0),
            Address(0x82_DAF2),
            Address(0x82_DAF3),
            Address(0x82_DAF6),
        ],
        layout: Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    },
    Leaf {
        instructions: &[Address(0x82_E06B), Address(0x82_E06E)],
        layout: Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    },
    Leaf {
        instructions: &[Address(0x82_EA8E)],
        layout: Layout {
            saved: StackBytes(0),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0xA0_8A7F),
            Address(0xA0_8A83),
            Address(0xA0_8A84),
            Address(0xA0_8A85),
            Address(0xA0_8A86),
            Address(0xA0_8A87),
            Address(0xA0_8A89),
            Address(0xA0_8A8C),
            Address(0xA0_8A90),
            Address(0xA0_8A93),
            Address(0xA0_8A95),
            Address(0xA0_8A98),
            Address(0xA0_8A9B),
            Address(0xA0_8A9C),
            Address(0xA0_8A9D),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0xA0_8AA9),
            Address(0xA0_8AAD),
            Address(0xA0_8AAE),
            Address(0xA0_8AAF),
            Address(0xA0_8AB0),
            Address(0xA0_8AB2),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B129),
            Address(0x80_B12B),
            Address(0x80_B12E),
            Address(0x80_B12F),
            Address(0x80_B131),
            Address(0x80_B134),
            Address(0x80_B136),
        ],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B140),
            Address(0x80_B142),
            Address(0x80_B144),
            Address(0x80_B146),
            Address(0x80_B148),
            Address(0x80_B149),
            Address(0x80_B14A),
            Address(0x80_B14B),
            Address(0x80_B14D),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B14E),
            Address(0x80_B150),
            Address(0x80_B152),
            Address(0x80_B153),
        ],
        layout: Layout {
            saved: StackBytes(3),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B154),
            Address(0x80_B156),
            Address(0x80_B159),
            Address(0x80_B15A),
            Address(0x80_B15C),
            Address(0x80_B15F),
            Address(0x80_B161),
        ],
        layout: Layout {
            saved: StackBytes(5),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[Address(0x80_B164)],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B165),
            Address(0x80_B167),
            Address(0x80_B168),
            Address(0x80_B16A),
            Address(0x80_B16C),
            Address(0x80_B16D),
            Address(0x80_B16E),
        ],
        layout: Layout {
            saved: StackBytes(3),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B16F),
            Address(0x80_B171),
            Address(0x80_B173),
            Address(0x80_B176),
            Address(0x80_B178),
            Address(0x80_B17A),
            Address(0x80_B17C),
            Address(0x80_B17E),
            Address(0x80_B180),
            Address(0x80_B182),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B183),
            Address(0x80_B185),
            Address(0x80_B188),
            Address(0x80_B189),
            Address(0x80_B18B),
            Address(0x80_B18E),
            Address(0x80_B190),
        ],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B191),
            Address(0x80_B193),
            Address(0x80_B194),
            Address(0x80_B195),
            Address(0x80_B197),
            Address(0x80_B199),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B19A),
            Address(0x80_B19C),
            Address(0x80_B19F),
            Address(0x80_B1A0),
            Address(0x80_B1A2),
            Address(0x80_B1A5),
            Address(0x80_B1A7),
        ],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B1A8),
            Address(0x80_B1AA),
            Address(0x80_B1AB),
            Address(0x80_B1AC),
            Address(0x80_B1AE),
            Address(0x80_B1B1),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B1B2),
            Address(0x80_B1B4),
            Address(0x80_B1B7),
            Address(0x80_B1B8),
            Address(0x80_B1BA),
            Address(0x80_B1BD),
            Address(0x80_B1BF),
        ],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[Address(0x80_B1C0), Address(0x80_B1C2)],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B1C3),
            Address(0x80_B1C5),
            Address(0x80_B1C8),
            Address(0x80_B1C9),
            Address(0x80_B1CB),
            Address(0x80_B1CE),
            Address(0x80_B1D0),
        ],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B1D1),
            Address(0x80_B1D3),
            Address(0x80_B1D5),
            Address(0x80_B1D7),
            Address(0x80_B1D8),
            Address(0x80_B1D9),
            Address(0x80_B1DB),
            Address(0x80_B1DD),
            Address(0x80_B1DF),
            Address(0x80_B1E0),
            Address(0x80_B1E1),
            Address(0x80_B1E3),
            Address(0x80_B1E6),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B1E7),
            Address(0x80_B1E9),
            Address(0x80_B1EC),
            Address(0x80_B1ED),
            Address(0x80_B1EF),
            Address(0x80_B1F2),
            Address(0x80_B1F4),
        ],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B1F5),
            Address(0x80_B1F7),
            Address(0x80_B1F8),
            Address(0x80_B1F9),
            Address(0x80_B1FA),
            Address(0x80_B1FC),
            Address(0x80_B1FF),
            Address(0x80_B201),
            Address(0x80_B203),
            Address(0x80_B205),
            Address(0x80_B207),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B208),
            Address(0x80_B20A),
            Address(0x80_B20D),
            Address(0x80_B20E),
            Address(0x80_B210),
            Address(0x80_B213),
            Address(0x80_B215),
        ],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[Address(0x80_B216), Address(0x80_B218)],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B219),
            Address(0x80_B21B),
            Address(0x80_B21E),
            Address(0x80_B21F),
            Address(0x80_B221),
            Address(0x80_B224),
            Address(0x80_B226),
        ],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[Address(0x80_B227), Address(0x80_B229), Address(0x80_B22B)],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[Address(0x80_B22C)],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B22D),
            Address(0x80_B22F),
            Address(0x80_B231),
            Address(0x80_B232),
            Address(0x80_B234),
        ],
        layout: Layout {
            saved: StackBytes(6),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B235),
            Address(0x80_B237),
            Address(0x80_B239),
            Address(0x80_B23B),
            Address(0x80_B23D),
            Address(0x80_B23E),
        ],
        layout: Layout {
            saved: StackBytes(4),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B23F),
            Address(0x80_B240),
            Address(0x80_B242),
            Address(0x80_B245),
            Address(0x80_B247),
            Address(0x80_B249),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
    Leaf {
        instructions: &[
            Address(0x80_B137),
            Address(0x80_B139),
            Address(0x80_B13B),
            Address(0x80_B13D),
        ],
        layout: Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    },
];

pub(super) const CALLERS: &[(Address, Layout)] = &[
    (
        Address(0x82_D96B),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_DA37),
        Layout {
            saved: StackBytes(1),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_DA5F),
        Layout {
            saved: StackBytes(8),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_DA7A),
        Layout {
            saved: StackBytes(8),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_DA9A),
        Layout {
            saved: StackBytes(8),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E292),
        Layout {
            saved: StackBytes(2),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E2DE),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E387),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E435),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E45A),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E464),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E4D8),
        Layout {
            saved: StackBytes(1),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E4DC),
        Layout {
            saved: StackBytes(1),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E4ED),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E544),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E582),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_E755),
        Layout {
            saved: StackBytes(0),
            returning: Return::Near,
        },
    ),
    (
        Address(0x82_EAA3),
        Layout {
            saved: StackBytes(0),
            returning: Return::Far,
        },
    ),
    (
        Address(0x82_EB20),
        Layout {
            saved: StackBytes(0),
            returning: Return::Far,
        },
    ),
    (
        Address(0xA0_8A5C),
        Layout {
            saved: StackBytes(2),
            returning: Return::Far,
        },
    ),
];
