//! ID37 supplements frozen0..36 with actual VRAM back-reference positioning.
use crate::room_background_support::ram_bytes;
use crate::{
    game_state::Game, rom::RomImage, room_background_cases::Operation, room_background_fixture::*,
    room_background_support::*,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(crate) fn capture(run: fn(Operation, &mut Game) -> Option<u8>) -> Option<Vec<u8>> {
    let result = catch_unwind(AssertUnwindSafe(|| unsafe {
        let mut game = initialized();
        let image = RomImage::zeroed(0x400000);
        // Two literal bytes followed by an absolute two-byte reference to the
        // decoder's starting byte offset (5000 for the actual CRE VRAM load).
        bytes(
            &image,
            0xb98000,
            &[0x01, 0x91, 0x92, 0x81, 0x00, 0x00, 0xff],
        );
        literal(&image, 0x808000, &[0xa1, 0xa2]);
        literal(&image, 0x809000, &[0xb1, 0xb2]);
        game.loaded_rom = Some(image.clone());
        game.ram().copy_in(0x7c3, &[0x00, 0x80, 0x80]);
        game.ram().copy_in(0x7c6, &[0x00, 0x90, 0x80]);
        put(&mut game, 0xe16, 0xabcd);

        let mut expected_ram = ram_bytes(&game);
        exp(&mut expected_ram, 0xe16, 0);
        expected_ram[0xc200..0xc202].copy_from_slice(&[0xb1, 0xb2]);
        let mut expected_vram = (&(*(*game.g_snes).ppu).vram).to_vec();
        expected_vram[0x2800] = 0x9291;
        expected_vram[0x2801] = 0x9291;
        expected_vram[0] = 0xa2a1;

        let mut writes = Vec::new();
        word(&mut writes, 0x2115, 0x80);
        word(&mut writes, 0x2116, 0x2800);
        writes.extend([(0x2118, 0x91), (0x2119, 0x92)]);
        // Actual ReadPpuByte(5000), then back-reference output byte at5002.
        word(&mut writes, 0x2116, 0x2800);
        word(&mut writes, 0x2116, 0x2801);
        writes.push((0x2118, 0x91));
        // Actual ReadPpuByte(5001), then back-reference output byte at5003.
        word(&mut writes, 0x2116, 0x2800);
        word(&mut writes, 0x2116, 0x2801);
        writes.push((0x2119, 0x92));
        word(&mut writes, 0x2116, 0);
        writes.extend([(0x2118, 0xa1), (0x2119, 0xa2)]);
        assert_eq!(writes.len(), 20);

        let mut output = b"RMBG1".to_vec();
        output.extend_from_slice(&37u32.to_le_bytes());
        record(&mut output, 0, &snapshot(&mut game));
        crate::room_background_observer::begin(
            37,
            writes,
            Vec::new(),
            image.clone(),
            image,
            Vec::new(),
        );
        let actual = catch_unwind(AssertUnwindSafe(|| run(Operation::Tiles, &mut game)));
        let trace = crate::room_background_observer::finish();
        let returned = actual.expect("actual initial tiles call failed");
        assert_eq!(returned, None, "actual unit return");
        record(&mut output, 7, &[0]);

        assert_eq!(&ram_bytes(&game), &expected_ram[..], "ID37 full RAM");
        assert_eq!(
            &(&(*(*game.g_snes).ppu).vram)[..],
            &expected_vram[..],
            "ID37 full VRAM"
        );
        assert!(!game.g_fail);
        assert_eq!(game.decompress_src, 0x809004);
        record(&mut output, 8, &snapshot(&mut game));
        record(&mut output, 9, &trace);
        output
    }));
    match result {
        Ok(bytes) => Some(bytes),
        Err(_) => {
            eprintln!("background offset case37 failed");
            None
        }
    }
}
