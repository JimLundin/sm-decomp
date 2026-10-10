use port::{Wram, WRAM_SIZE};
#[path = "../src/audio.rs"]
mod audio;

#[test]
fn music_presence_scans_delay_even_when_ring_positions_are_equal() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x637, 0x8000);
    assert_eq!(audio::dispatch(&mut ram, "HasQueuedMusic", 0, 0), Ok(1));
    ram.set_word(0x637, 0);
    ram.set_word(0x619, 5);
    assert_eq!(audio::dispatch(&mut ram, "HasQueuedMusic", 0, 0), Ok(0));
}

#[test]
fn delayed_music_clamps_delay_and_only_fixed_delay_respects_full_ring() {
    let mut bytes = vec![0xa5; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_word(0x998, 39);
    ram.set_word(0x639, 14);
    ram.set_word(0x63b, 0);
    let before = ram.word(0x627);
    assert_eq!(audio::dispatch(&mut ram, "QueueMusic_Delayed8", 0x8100, 0), Ok(0));
    assert_eq!(ram.word(0x627), before);
    assert_eq!(ram.word(0x639), 14);
    assert_eq!(audio::dispatch(&mut ram, "QueueMusic_DelayedY", 0x8100, 1), Ok(0));
    assert_eq!(ram.word(0x627), 0x8100);
    assert_eq!(ram.word(0x637), 8);
    assert_eq!(ram.word(0x639), 0);
    ram.set_word(0x998, 40);
    assert_eq!(audio::dispatch(&mut ram, "QueueMusic_DelayedY", 7, 0xffff), Ok(0));
    assert_eq!(ram.word(0x619), 0xa5a5);
}

#[test]
fn sfx_wrappers_truncate_sound_to_a_byte_and_clear_the_next_slot() {
    for channel in 0..3 {
        let mut bytes = vec![0x5a; WRAM_SIZE];
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(0x998, 0);
        ram.set_word(0x5f5, 0);
        ram.set_word(0x592, 0x7fff);
        ram.set_byte(0x643 + channel, 14);
        ram.set_byte(0x646 + channel, 15);
        let symbol = format!("QueueSfx{}_Max3", channel + 1);
        assert_eq!(audio::dispatch(&mut ram, &symbol, 0x1234, 0), Ok(0));
        assert_eq!(ram.byte(0x653 + channel), 3);
        assert_eq!(ram.byte(0x656 + channel * 16 + 15), 0x34);
        assert_eq!(ram.byte(0x646 + channel), 0);
        assert_eq!(ram.byte(0x656 + channel * 16), 0);
    }
}

#[test]
fn full_sfx_ring_replaces_only_with_lower_sound_priority_without_advancing() {
    let mut bytes = vec![0; WRAM_SIZE];
    let mut ram = Wram::new(&mut bytes).unwrap();
    ram.set_byte(0x646, 15);
    ram.set_byte(0x665, 9);
    assert_eq!(audio::dispatch(&mut ram, "QueueSfx1_Internal", 0x0310, 0), Ok(0));
    assert_eq!(ram.byte(0x665), 3);
    assert_eq!(ram.byte(0x646), 15);
    assert_eq!(audio::dispatch(&mut ram, "QueueSfx1_Internal", 0x0910, 0), Ok(0));
    assert_eq!(ram.byte(0x665), 3);
    assert_eq!(audio::dispatch(&mut ram, "QueueSfx1_Max15", 1, 0), Ok(0));
    assert_eq!(ram.byte(0x665), 3);
    assert_eq!(ram.byte(0x653), 15);
}

#[test]
fn inhibited_sfx_still_records_quota_without_changing_ring() {
    for (offset, gate) in [(0x5f5, 1), (0x998, 40), (0x592, 0x8000)] {
        let mut bytes = vec![0; WRAM_SIZE];
        let mut ram = Wram::new(&mut bytes).unwrap();
        ram.set_word(offset, gate);
        ram.set_byte(0x656, 0x55);
        assert_eq!(audio::dispatch(&mut ram, "QueueSfx1_Max9", 0x33, 0), Ok(0));
        assert_eq!(ram.byte(0x653), 9);
        assert_eq!(ram.byte(0x646), 0);
        assert_eq!(ram.byte(0x656), 0x55);
    }
}

#[test]
fn invalid_queue_cursors_are_rejected_without_unchecked_c_or_memory_writes() {
    let mut bytes = vec![0; WRAM_SIZE];
    bytes[0x639] = 15;
    let before = bytes.clone();
    assert_eq!(
        audio::dispatch(&mut Wram::new(&mut bytes).unwrap(), "QueueMusic_DelayedY", 1, 8),
        Err("invalid music ring cursor")
    );
    assert_eq!(bytes, before);
    bytes[0x646] = 16;
    let before = bytes.clone();
    assert_eq!(
        audio::dispatch(&mut Wram::new(&mut bytes).unwrap(), "QueueSfx1_Max9", 1, 0),
        Err("invalid sound ring cursor")
    );
    assert_eq!(bytes, before);
}
