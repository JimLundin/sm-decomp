//! Pure cores of moved enemies, checked against the bank logs. Expected values are
//! worked out by hand from the routines' descriptions, not read back from the code.
use sm_game::actors::kihunter::swing;
use sm_game::actors::powamp::{DEFLATE, INFLATE, balloon_offset};
use sm_game::actors::yard::signed_speed;

/// $A8:C234 set powamp balloon Y position: A = (pointer - 4 - list) / 2, 0 from 6 on,
/// indexes the Y offsets $A8:C277 (inflate: -12, -16, -20) or $A8:C27D (deflate: -20,
/// -16, -12); pointers from $C191 on count into the deflate list.
#[test]
fn the_balloon_rises_as_it_inflates_and_sinks_as_it_deflates() {
    let negative = |offset: u16| offset.wrapping_neg();
    // Inflate ($A8:C183): entries at C183, C187, C18B, then the sleep at C18F.
    for (pointer, offset) in [(0xc187, 12), (0xc18b, 16), (0xc18f, 20)] {
        assert_eq!(balloon_offset(pointer), negative(offset), "{pointer:#x}");
    }
    // The list's own address gives A = -2, which is not below 6 unsigned: frame 0.
    assert_eq!(balloon_offset(INFLATE), negative(12));
    // Deflate ($A8:C191): entries at C191 and C195.
    for (pointer, offset) in [(0xc195, 20), (0xc199, 16), (0xc19d, 12)] {
        assert_eq!(balloon_offset(pointer), negative(offset), "{pointer:#x}");
    }
    assert_eq!(
        balloon_offset(DEFLATE + 16),
        negative(20),
        "past the third frame: frame 0"
    );
}

/// $A3:CD82 yard crawling velocity signs: 0000,0000 is positive and FFFF,0001 negative;
/// $A3:CE27 sets velocity = (speed ^ first) + second.
#[test]
fn yard_velocity_signs_keep_or_negate_the_speed() {
    assert_eq!(signed_speed(0x0120, [0, 0]), 0x0120);
    assert_eq!(signed_speed(0x0120, [0xffff, 1]), 0xfee0);
    assert_eq!(signed_speed(0, [0xffff, 1]), 0, "negating zero stays zero");
}

/// $A8:F3B8 ki-hunter swoop: the angular velocity (16.16) gains the angular
/// acceleration, its whole part capped (forwards at most the cap, backwards at least),
/// then the arc angle moves by the whole part; forwards the arc ends at 100h, backwards
/// once the angle is below 80h.
#[test]
fn the_swoop_accelerates_to_its_cap_and_ends_at_the_arc_end() {
    // Forwards from rest: 0.2000h a frame, so the angle does not move yet.
    assert_eq!(swing(0, 0x2000, 2, 0x80), (0x2000, 0x80, false));
    // 1.F000h + 0.2000h = 2.1000h passes a cap of 1: the whole part stops at 1.
    assert_eq!(swing(0x1_f000, 0x2000, 1, 0x80), (0x1_1000, 0x81, false));
    // A whole part equal to the cap is the cap.
    assert_eq!(swing(0x1_e000, 0x2000, 2, 0x80), (0x2_0000, 0x82, false));
    // Reaching angle 100h ends the forward arc.
    assert_eq!(swing(0x1_0000, 0, 2, 0xff), (0x1_0000, 0x100, true));

    // Backwards, -0.2000h a frame and a cap of -2: -0.2000h is above the cap.
    assert_eq!(
        swing(0, 0xffff_e000, 0xfffe, 0xff),
        (0xffff_e000, 0xfe, false)
    );
    // -2.0 - 0.2000h passes the cap: the whole part stops at -2.
    let (velocity, angle, finished) = swing(0xfffe_0000, 0xffff_e000, 0xfffe, 0x81);
    assert_eq!(velocity, 0xfffe_e000);
    assert_eq!(angle, 0x7f, "0x81 - 2");
    assert!(finished, "below 80h ends the backward arc");
}
