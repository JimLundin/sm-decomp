use super::*;
use crate::rom::Rom;

#[test]
fn the_tables_decode_shape_by_shape() {
    let squares = [0x00, 0x00, 0x80, 0x80, 0x00, 0x80, 0x00, 0x80];
    let mut surfaces = vec![0; 3 * COLUMNS];
    surfaces[2 * COLUMNS] = 0x30;
    surfaces[2 * COLUMNS + 1] = 0x0f;
    let factors = [0, 0, 0x00, 0x01, 0, 0, 0x00, 0x01, 0x00, 0x10, 0xc0, 0x00];
    let slopes = Slopes::decode(&squares, &surfaces, &factors);
    let Some(Shape::Square(half)) = slopes.shape(0) else {
        panic!("shape 0 is square");
    };
    assert!(!half.solid_at(false, false) && half.solid_at(true, true));
    let Some(Shape::Square(width)) = slopes.shape(1) else {
        panic!("shape 1 is square");
    };
    assert!(width.solid_at(true, false) && !width.solid_at(false, true));
    assert_eq!(slopes.shape(2), None, "the square table is short");
    assert_eq!(slopes.shape(5), None, "the surface table is short");
    let short = Slopes::decode(&[], &surfaces, &factors);
    let surface = short.surfaces[2];
    assert_eq!((surface.top(0), surface.top(1)), (0x10, 0x0f), "five bits");
    assert_eq!(surface.factor, 0x00c0);
}

fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

#[test]
fn the_rom_holds_the_bank_log_shapes() {
    let Some(cartridge) = rom() else {
        return;
    };
    let slopes = Slopes::load(cartridge.lorom()).unwrap();
    let square = |shape| match slopes.shape(shape) {
        Some(Shape::Square(square)) => square.solid,
        other => panic!("shape {shape} is {other:?}"),
    };
    let surface = |shape| match slopes.shape(shape) {
        Some(Shape::Surface(surface)) => surface,
        other => panic!("shape {shape} is {other:?}"),
    };
    // $94:8E54: half height, half width, quarter, three-quarters, whole.
    assert_eq!(square(0), [false, false, true, true]);
    assert_eq!(square(1), [false, true, false, true]);
    assert_eq!(square(2), [false, false, false, true]);
    assert_eq!(square(3), [false, true, true, true]);
    assert_eq!(square(4), [true; 4]);
    // $94:8B2B row 12h, the triangle: 10h down to 1; row 13h, the rectangle: all 0.
    let triangle = surface(0x12);
    assert_eq!(
        triangle.tops,
        core::array::from_fn(|column| 0x10 - u8::try_from(column).unwrap())
    );
    assert_eq!(triangle.factor, 0x00c0);
    let rectangle = surface(0x13);
    assert_eq!(rectangle.tops, [0; COLUMNS]);
    assert_eq!(rectangle.factor, 0x0100);
    // Row 1Bh, the upper half-width triangle, whose 14h survives the five-bit mask.
    assert_eq!(surface(0x1b).top(0), 0x14);
    assert_eq!(surface(0x1f).factor, 0x0050);
    assert_eq!(slopes.shape(0x20), None);
}
