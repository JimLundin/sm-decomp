use super::*;

#[test]
fn a_speed_keeps_its_pixel_and_subpixel_words() {
    let speed = Speed::new(0x0002, 0xC000);
    assert_eq!(speed, Speed(0x0002_C000));
    assert_eq!((speed.pixels(), speed.subpixels()), (0x0002, 0xC000));
}
