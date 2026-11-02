//! Concrete player ports on the current owner; physical resources stay on Game.
#![forbid(unsafe_code)]
use super::prepared_execution::PreparedActorPorts;
use sm_data::cartridge::RomRecord;
use sm_game::actors::owned_execution::{ImportedActorAllocation, PlayerPorts};
use sm_game::persistent_session::PersistentSession;
use sm_game::player_effects::save;
use sm_game::scene::state::WordOrder;

impl PreparedActorPorts<'_> {
    fn player_byte(&mut self, owner: &mut PersistentSession, at: usize, value: u8) {
        if !sm_game::session_encoding::write_projected_byte(&mut owner.gameplay, at, value) {
            self.set_actor_alias_byte(&mut owner.gameplay, at, value);
        }
    }
    fn player_native_word(&mut self, owner: &mut PersistentSession, at: usize, value: u16) {
        // The old native slice checks the whole extent before either publication.
        assert!(at.checked_add(2).is_some_and(|end| end <= 0x20000));
        for (offset, value) in value.to_ne_bytes().into_iter().enumerate() {
            self.player_byte(owner, at + offset, value);
        }
    }
    fn player_spawn_hdma(
        &mut self,
        owner: &mut PersistentSession,
        bank: u8,
        read: impl Fn(usize) -> u16,
    ) {
        use sm_game::hdma::{ObjectWord as Field, Word};
        let Some(index) = (0..sm_game::hdma::OBJECTS).find(|&index| {
            owner.gameplay.hdma_word(Word::Object {
                index,
                field: Field::Channels,
            }) == 0
        }) else {
            return;
        };
        let store = |owner: &mut PersistentSession, field, value| {
            owner
                .gameplay
                .store_hdma_word(Word::Object { index, field }, value, WordOrder::Little)
        };
        store(owner, Field::PreInstruction, 0x84b8);
        store(owner, Field::PreInstructionBank, 0x88);
        // Original definition reads the instruction list after both setup stores.
        store(owner, Field::InstructionList, read(1));
        store(owner, Field::InstructionTimer, 1);
        for field in [Field::Timer, Field::A, Field::B, Field::C, Field::D] {
            store(owner, field, 0);
        }
        store(owner, Field::Channels, 4 << index);
        let channel = 0x20 + 0x10 * u16::try_from(index).expect("HDMA slot");
        store(owner, Field::BankSlot, (u16::from(bank) << 8) | channel);
        let control = read(0).to_le_bytes();
        // Actual platform DMA registers are not owned gameplay; retain their seam.
        self.physical_effect(owner, |game| {
            sm_game::menus::host::MenuRuntime::write_dma_register(
                game,
                0x4300 + channel,
                control[0],
            );
            sm_game::menus::host::MenuRuntime::write_dma_register(
                game,
                0x4301 + channel,
                control[1],
            );
        });
    }
}

impl PlayerPorts for PreparedActorPorts<'_> {
    fn samus_sprite(&mut self, owner: &mut PersistentSession, index: u16, x: u16, y: u16) {
        let content = sm_game::scene::prepared_samus::PreparedSamusContent::prepare(owner)
            .expect("prepared Samus content");
        sm_game::scene::prepared_samus::draw_sprite(
            owner,
            &content,
            index,
            sm_game::geometry::Point { x, y },
        )
        .expect("prepared Samus sprite");
    }
    fn atmospheric_tile(&mut self, owner: &mut PersistentSession, slot: usize, kind: u16) {
        let content = sm_game::scene::prepared_samus::PreparedSamusContent::prepare(owner)
            .expect("prepared atmosphere content");
        sm_game::scene::prepared_samus::draw_atmospheric_tile(owner, &content, slot, kind)
            .expect("prepared atmosphere tile");
    }
    fn visual_upload(
        &mut self,
        owner: &mut PersistentSession,
        request: sm_game::visual_state::Upload,
    ) {
        // Synchronous DMA observes the current ordered producer prefix immediately.
        self.physical_effect(owner, |game| {
            crate::game::animated_tiles::native::upload(request, game)
        });
    }
    fn player_hdma(
        &mut self,
        owner: &mut PersistentSession,
        bank: u8,
        definition: sm_game::display::hdma_spawn::HdmaObjectDefinition,
    ) {
        self.player_spawn_hdma(owner, bank, |index| match index {
            0 => u16::from_ne_bytes([definition.dma_control, definition.target_register]),
            1 => definition.instruction_list,
            _ => unreachable!(),
        });
    }
    fn player_hdma_record(
        &mut self,
        owner: &mut PersistentSession,
        bank: u8,
        definition: RomRecord,
    ) {
        self.player_spawn_hdma(owner, bank, |index| definition.word(index * 2));
    }
    fn xray_window(&mut self, owner: &mut PersistentSession, row: u16, value: u16) {
        self.player_native_word(owner, 0x9800 + usize::from(row) * 2, value);
    }
    fn xray_tables(&mut self, owner: &mut PersistentSession) {
        self.player_native_word(owner, 0xa88, 1);
        self.player_byte(owner, 0xa8a, 0x98);
        self.player_native_word(owner, 0xa8b, 0);
        owner.gameplay.player.demo.previous_held = 0x98c8;
        self.player_native_word(owner, 0xa8e, 0x9098);
        self.player_byte(owner, 0xa90, 0x99);
        // The terminator is a little-endian slice, not a native-word store.
        for (offset, value) in 0u16.to_le_bytes().into_iter().enumerate() {
            self.player_byte(
                owner,
                sm_game::ram::words::XRAY_HDMA_TABLE_END + offset,
                value,
            );
        }
    }
    fn enable_hdma(&mut self, owner: &mut PersistentSession) {
        let value = owner.gameplay.hdma_word(sm_game::hdma::Word::Enabled) | 0x8000;
        owner
            .gameplay
            .store_hdma_word(sm_game::hdma::Word::Enabled, value, WordOrder::Little);
    }
    fn message_step(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut sm_game::actors::owned_execution::ActorAllocations,
        presentation: &sm_game::scene::actor_presentation::Binding,
        samus_content: &sm_game::scene::prepared_samus::PreparedSamusContent,
        request: u16,
    ) -> u8 {
        self.resume_actor_dialogue(owner, allocations, presentation, samus_content, request)
    }
    fn message_poll(&mut self, owner: &mut PersistentSession, request: u16) -> i32 {
        // Poll consumes an active answer and clears only its active index.
        let read = |at| {
            sm_game::session_encoding::projected_byte(&owner.gameplay, at)
                .unwrap_or_else(|| self.actor_alias_byte(&owner.gameplay, at))
        };
        let active = u16::from_le_bytes([read(0x1c1f), read(0x1c20)]);
        if request == active {
            let selection = u16::from_le_bytes([read(0x5f9), read(0x5fa)]);
            self.player_native_word(owner, 0x1c1f, 0);
            i32::from(selection)
        } else {
            // A live request differs from the captured active dialogue identity.
            for (offset, byte) in request.to_le_bytes().into_iter().enumerate() {
                self.player_byte(owner, 0xdc8 + offset, byte);
            }
            -1
        }
    }
    fn sram_native_word(&self, byte: usize) -> u16 {
        self.game.g_sram.word(byte)
    }
    fn set_sram_native_word(&mut self, byte: usize, word: u16) {
        self.game.g_sram.set_word(byte, word);
    }
    fn sram_snapshot(&self) -> Box<[u8]> {
        self.game
            .g_sram
            .bytes(0..crate::recording::BASE_SRAM_SIZE)
            .into_boxed_slice()
    }
    fn deliver_save(
        &mut self,
        _owner: &mut PersistentSession,
        request: save::SaveRequest,
    ) -> save::Delivery {
        // Prepared storage is an explicit host output, never an implicit legacy
        // path write or a durability acknowledgement without the actual consumer.
        save::Delivery::Pending(request)
    }
    fn retain_pending_save(&mut self, _owner: &mut PersistentSession, request: save::SaveRequest) {
        self.game.prepared_save_requests.push(request);
    }
}

impl sm_game::actors::prepared_player_frame::FramePorts for PreparedActorPorts<'_> {
    fn cannon_tile(
        &mut self,
        owner: &mut PersistentSession,
        x: u16,
        y: u16,
        body_y: u16,
        character: u16,
    ) {
        let content = owner
            .actor_content
            .as_ref()
            .expect("prepared actor content")
            .clone();
        content
            .binding()
            .cannon_tile(&mut owner.gameplay, x, y, body_y, character, self)
            .expect("prepared cannon publication");
    }
    fn invalid_draw_mode(&mut self, owner: &mut PersistentSession) {
        self.physical_effect(owner, |game| {
            crate::rtl::report_unreachable(game);
        });
    }
}
