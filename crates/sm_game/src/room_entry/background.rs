//! Background command execution over retained live captures.
#![forbid(unsafe_code)]
use super::{
    content::{
        AssetSource, BackgroundCursor, BackgroundWidth, BackgroundWord, LoadMode, VramDestination,
    },
    effects::{
        BufferClear, DeviceByte, DeviceWord, EntryHost, ExternalField, InstallBuffer,
        StagingSource, VramSlot,
    },
};
fn clear_fx(h: &mut impl EntryHost, mode: &LoadMode) {
    h.clear_buffer(match mode {
        LoadMode::Initial => BufferClear::InitialFx,
        LoadMode::Door => BufferClear::DoorFx,
    });
}
fn clear_bg2(h: &mut impl EntryHost, mode: &LoadMode) {
    h.clear_buffer(match mode {
        LoadMode::Initial => BufferClear::InitialBg2,
        LoadMode::Door => BufferClear::DoorBg2,
    });
}
fn transfer(
    h: &mut impl EntryHost,
    cursor: BackgroundCursor,
    mode: &LoadMode,
) -> Option<BackgroundCursor> {
    let data = h.capture_background(&cursor, mode.clone());
    match mode {
        LoadMode::Initial => {
            h.device_vram_address(data.vram());
            h.device_word(DeviceWord::Dma1Mode, 0x1801);
            h.device_word(
                DeviceWord::Dma1SourceLow,
                data.word(BackgroundWord::SourceLow),
            );
            h.device_word(
                DeviceWord::Dma1SourceBank,
                data.word(BackgroundWord::SourceBankWord),
            );
            h.device_word(DeviceWord::Dma1Count, data.word(BackgroundWord::ByteCount));
            h.device_byte(DeviceByte::VramIncrement, 0x80);
            h.device_byte(DeviceByte::DmaEnable, 2);
        }
        LoadMode::Door => {
            let destination = data.vram();
            let source = data.source();
            let bytes = data.word(BackgroundWord::ByteCount);
            h.copy_vram(destination, source, bytes);
        }
    }
    cursor.advance(BackgroundWidth::Transfer, &h.session().world)
}
pub fn load(h: &mut impl EntryHost, mode: LoadMode) {
    clear_fx(h, &mode);
    match mode {
        LoadMode::Initial => {
            if h.read_external(ExternalField::FxTilemap) != 0 {
                let source = h.read_external(ExternalField::FxTilemap);
                h.device_word(DeviceWord::Dma1SourceLow, source);
                h.device_vram_address(VramDestination::Slot(VramSlot::FxTilemap));
                h.device_word(DeviceWord::Dma1Mode, 0x1801);
                h.device_word(DeviceWord::Dma1SourceBank, 0x8a);
                h.device_word(DeviceWord::Dma1Count, 0x840);
                h.device_byte(DeviceByte::VramIncrement, 0x80);
                h.device_byte(DeviceByte::DmaEnable, 2);
            }
        }
        LoadMode::Door => {
            let source = h.read_external(ExternalField::FxTilemap);
            if source != 0 {
                h.copy_vram(
                    VramDestination::Slot(VramSlot::FxTilemap),
                    AssetSource::FxTilemap(source),
                    2112,
                );
            }
        }
    }
    let state = h.session().room.entry.state.clone();
    let mut next = h.selected_background(&state, mode.clone());
    while let Some(cursor) = next {
        if cursor.is_end() {
            return;
        }
        let command = h
            .capture_background(&cursor, mode.clone())
            .word(BackgroundWord::Opcode);
        if command >> 1 == 0 {
            return;
        }
        // Operand zero is still executed; only a returned next-command zero ends.
        let operand = cursor.advance(BackgroundWidth::Opcode, &h.session().world);
        next = match command >> 1 {
            1 => transfer(h, operand.expect("transfer operands"), &mode),
            2 => {
                let operand = operand.expect("decompression operands");
                let data = h.capture_background(&operand, mode.clone());
                let source = data.source();
                let destination = data.destination();
                h.decompress(source, InstallBuffer::Background(destination));
                operand.advance(BackgroundWidth::Decompress, &h.session().world)
            }
            3 => {
                clear_fx(h, &mode);
                operand
            }
            4 => {
                let next = transfer(h, operand.expect("transfer operands"), &mode);
                h.write_external(ExternalField::BackgroundCopyByte, 2);
                next
            }
            5 => {
                clear_bg2(h, &mode);
                operand
            }
            6 => {
                match mode {
                    LoadMode::Initial => clear_bg2(h, &mode),
                    LoadMode::Door => {
                        h.clear_buffer(BufferClear::DoorDualBg);
                        h.copy_vram(
                            VramDestination::Slot(VramSlot::BackgroundFirst),
                            AssetSource::Staging(StagingSource::Background),
                            0x1000,
                        );
                        h.copy_vram(
                            VramDestination::Slot(VramSlot::BackgroundSecond),
                            AssetSource::Staging(StagingSource::Background),
                            0x1000,
                        );
                    }
                }
                operand
            }
            7 => {
                let operand = operand.expect("match operands");
                // Match always uses initial/native acquisition, including door loads.
                let exit = h
                    .capture_background(&operand, LoadMode::Initial)
                    .match_exit();
                if h.session().room.entry.exit == exit {
                    let transfer_at = operand
                        .advance(BackgroundWidth::Opcode, &h.session().world)
                        .expect("matching background command has transfer operands");
                    transfer(h, transfer_at, &mode)
                } else {
                    operand.advance(BackgroundWidth::MatchAndTransfer, &h.session().world)
                }
            }
            _ => panic!("invalid room background command: {command:#x}"),
        };
    }
}
