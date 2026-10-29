//! Representation tests prepared for the isolated carrier; NOT EXECUTED.
use crate::game::player_transitions::state::*;
use crate::game::{
    room_entry::world::WorldBinding, session::Session, session_encoding as encoding,
};

#[test]
fn every_encoded_word_roundtrips_without_normalization() {
    macro_rules! check {
        ($($ty:ty),+ $(,)?) => {
            for raw in 0..=u16::MAX {
                $(assert_eq!(<$ty>::from_raw(raw).to_raw(), raw);)+
            }
            $(assert_eq!(<$ty>::default().to_raw(), 0);)+
        };
    }
    check!(
        PoseRequestWord,
        PoseId,
        MomentumAction,
        InterruptionAction,
        HurtAction,
        AnimationSkip,
        BombJump,
        LandingBounce,
        MovementMode,
        InputMode,
        FrameAlphaMode,
        FrameGammaMode
    );
    for raw in 0..=u16::MAX {
        let view = PoseRequestWord::from_raw(raw).view();
        if raw & 0x8000 == 0 {
            assert_eq!(view, RequestView::Pending(PoseId::from_raw(raw)));
        } else {
            assert_eq!(view, RequestView::Inactive { encoded: raw });
        }
    }
}

#[test]
fn construction_and_explicit_preparation_reset_are_distinct() {
    let mut state = PlayerTransitions::default();
    assert_eq!(
        state.ordinary.view(),
        RequestView::Pending(PoseId::from_raw(0))
    );
    assert_eq!(state.interrupted.to_raw(), 0);
    assert_eq!(state.transitional.to_raw(), 0);
    assert_eq!(state.input_mode, InputMode::Unknown(0));
    assert_eq!(state.frame_alpha, FrameAlphaMode::Unknown(0));
    assert_eq!(state.frame_gamma, FrameGammaMode::Unknown(0));
    assert_eq!(MovementMode::default(), MovementMode::Unknown(0));
    state.ordinary = PoseRequestWord::from_raw(0xffff);
    state.interrupted = PoseRequestWord::from_raw(0xffff);
    state.transitional = PoseRequestWord::from_raw(0xffff);
    assert_eq!(
        state.ordinary.view(),
        RequestView::Inactive { encoded: 0xffff }
    );
    assert_eq!(state.interrupted.to_raw(), 0xffff);
    assert_eq!(state.transitional.to_raw(), 0xffff);
}

const TYPED: [usize; 13] = [
    0xa28, 0xa2a, 0xa2c, 0xa2e, 0xa30, 0xa32, 0xa9a, 0xa58, 0xa42, 0xa56, 0xa60, 0xb20, 0xa5a,
];

#[test]
fn typed_decode_visit_and_partial_writes_share_one_live_word() {
    for raw in [
        0u16, 1, 0x7fff, 0x8000, 0x8001, 0xfffe, 0xffff, 0xe913, 0xa337,
    ] {
        let mut bytes = vec![0; 0x20000];
        for at in TYPED {
            bytes[at..at + 2].copy_from_slice(&raw.to_le_bytes());
        }
        let mut session = encoding::decode(
            &|at| bytes[at],
            bytes.len(),
            WorldBinding::canonical_legacy(),
        );
        let mut visits = [0; 13];
        encoding::visit(&session, |at, word| {
            if let Some(index) = TYPED.iter().position(|base| *base == at) {
                visits[index] += 1;
                assert_eq!(word, raw);
            }
        });
        assert_eq!(visits, [1; 13]);
        for at in TYPED {
            // Change high byte, then low byte: second write must retain live high.
            assert!(encoding::write_projected_byte(&mut session, at + 1, 0x37));
            assert_eq!(encoding::projected_byte(&session, at), Some(raw as u8));
            assert!(encoding::write_projected_byte(&mut session, at, 0xe9));
            assert_eq!(encoding::projected_byte(&session, at + 1), Some(0x37));
            assert_eq!(encoding::projected_byte(&session, at), Some(0xe9));
        }
        encoding::encode(&session, |at, span| {
            bytes[at..at + span.len()].copy_from_slice(span)
        });
        for at in TYPED {
            assert_eq!(encoding::read(&|at| bytes[at], at), 0x37e9);
        }
    }
}

#[test]
fn newly_owned_union_has_exactly_sixty_four_unique_projected_bytes() {
    let mut session = Session::with_world(WorldBinding::canonical_legacy());
    let scalar = [
        0xa18, 0xa42, 0xa52, 0xa56, 0xa60, 0xa82, 0xa9c, 0xa9e, 0xaa0, 0xaa2, 0xad2, 0xb1a, 0xb20,
        0xb22, 0xb2a, 0xb38,
    ];
    let mut addresses = std::collections::BTreeSet::new();
    for base in scalar {
        addresses.extend([base, base + 1]);
    }
    addresses.extend(0xad4..0xaf4);
    assert_eq!(addresses.len(), 64);
    for at in addresses {
        let value = (at as u8).wrapping_mul(17);
        assert!(encoding::write_projected_byte(&mut session, at, value));
        assert_eq!(encoding::projected_byte(&session, at), Some(value));
    }
}
