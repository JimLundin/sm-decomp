//! Actual prepared player-frame binding on the parent's exclusive actor host.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::PreparedActorState;
use crate::persistent_session::PersistentSession;
use crate::player_drawing::FrameHost;
use crate::player_projectiles::cannon::Tiles;
use crate::scene::samus_body::Body;
use crate::session_access::SessionAccess;

/// Remaining finite edges require real current-owner implementations.
/// Cannon sinks require supplied producer/variants and source-transfer identities;
/// the physical upload must remain pending until its consumer exists.
pub trait FramePorts {
    fn cannon_tile(
        &mut self,
        owner: &mut PersistentSession,
        x: u16,
        y: u16,
        body_y: u16,
        character: u16,
    );
    fn invalid_draw_mode(&mut self, owner: &mut PersistentSession);
}
impl FrameHost for PreparedActorState<'_> {
    fn owned_frame_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        self.guarded_actor_call(|host| {
            host.refresh_inputs();
            host.samus_content
                .check(host.owner)
                .expect("prepared Samus binding");
            let result = action(host);
            host.refresh_inputs();
            result
        })
    }
    fn refresh_player_observations(&mut self) {
        self.refresh_inputs();
    }
    fn advance_atmosphere(&mut self) {
        crate::scene::prepared_samus::advance_atmosphere(self.owner, &self.samus_content)
            .expect("prepared atmosphere");
    }
    fn draw_body(&mut self, mode: Body) {
        crate::scene::prepared_samus::draw_body(self.owner, &self.samus_content, mode)
            .expect("prepared Samus body");
    }
    fn draw_echoes(&mut self) {
        crate::scene::prepared_samus::draw_echoes(self.owner, &self.samus_content)
            .expect("prepared Samus echoes");
    }
    fn draw_crash_echoes(&mut self, slot: u16) {
        crate::scene::prepared_samus::draw_crash_echoes(self.owner, &self.samus_content, slot)
            .expect("prepared Samus crash echoes");
    }
    fn cannon_tile(&mut self, x: u16, y: u16, body_y: u16, character: u16) {
        self.aliases
            .cannon_tile(self.owner, x, y, body_y, character);
        self.refresh_inputs();
    }
    fn queue_cannon_tiles(&mut self, tiles: Tiles) {
        self.guarded_actor_call(|host| host.prepared_cannon_tiles(tiles));
        self.refresh_inputs();
    }
    fn draw_flare(&mut self) {
        self.owned_frame_call(crate::player_projectiles::feedback::flare);
    }
    fn draw_grapple_flare(&mut self) {
        self.owned_frame_call(crate::player_projectiles::feedback::grapple_flare);
    }
    fn restore_grapple_start_positions(&mut self) {
        self.guarded_actor_call(crate::player_effects::grapple::restore_start_positions);
        self.refresh_inputs();
    }
    fn update_grapple_tiles(&mut self) {
        self.guarded_actor_call(Self::prepared_grapple_tiles);
        self.refresh_inputs();
    }
    fn draw_grapple_rope(&mut self) {
        self.owned_frame_call(crate::grapple_rope::draw);
    }
    fn draw_projectiles(&mut self) {
        self.owned_frame_call(crate::player_projectiles::presentation::draw_projectiles);
    }
    fn finish_player_frame(&mut self) {
        self.guarded_actor_call(Self::prepared_finish_frame);
        self.refresh_inputs();
    }
    fn invalid_draw_mode(&mut self) {
        self.aliases.invalid_draw_mode(self.owner);
    }
}

impl PreparedActorState<'_> {
    pub(crate) fn prepared_beam_tiles_and_palette(&mut self) {
        let beams = self.session().inventory.beams.active & 0xfff;
        self.prepared_cannon_tiles(Tiles::Beam(beams));
        crate::samus_palette::write_beam_palette(self, beams);
    }
    fn prepared_cannon_tiles(&mut self, tiles: Tiles) {
        let (size, destination) = match tiles {
            Tiles::Beam(_) => (256, 0x6300),
            Tiles::Cannon(_) => (32, 0x61f0),
        };
        let saved = self.session().queued_tilemaps.tail;
        self.prepared_queue_native_word(saved, size);
        // Keep source capture after size publication, including off-table aliases.
        let source = match tiles {
            Tiles::Beam(index) => crate::visual_host::Content::table(
                self,
                crate::visual_host::Source::Loaded,
                0x90c3b1,
            )
            .word(usize::from(index) * 2),
            Tiles::Cannon(pointer) => self
                .projectile_record(
                    crate::actors::owned_execution::ProjectileRecordSource::PlayerNative,
                    0x90,
                    pointer,
                )
                .word(0),
        };
        self.prepared_queue_native_word(saved.wrapping_add(2), source);
        self.prepared_queue_bank(saved.wrapping_add(4), 0x9a);
        self.prepared_queue_native_word(saved.wrapping_add(5), destination);
        self.prepared_queue_tail(saved.wrapping_add(7));
    }
    fn prepared_queue_native_word(&mut self, cursor: u16, value: u16) {
        let bytes = value.to_ne_bytes();
        for (offset, value) in bytes.into_iter().enumerate() {
            self.presentation
                .queue_byte(
                    &mut self.owner.gameplay,
                    usize::from(cursor) + offset,
                    value,
                    self.aliases.presentation_diagnostics(),
                )
                .expect("prepared player native queue");
            self.refresh_inputs();
        }
    }
    fn prepared_queue_bank(&mut self, cursor: u16, bank: u8) {
        self.presentation
            .queue_byte(
                &mut self.owner.gameplay,
                usize::from(cursor),
                bank,
                self.aliases.presentation_diagnostics(),
            )
            .expect("prepared player queue bank");
        self.refresh_inputs();
    }
    fn prepared_queue_tail(&mut self, tail: u16) {
        self.presentation
            .publish_queue_tail(&mut self.owner.gameplay, tail)
            .expect("prepared player queue tail");
        self.refresh_inputs();
    }
    fn prepared_grapple_tiles(&mut self) {
        let timer = self
            .session()
            .player
            .swing
            .start_animation_timer
            .wrapping_sub(1);
        self.session_mut().player.swing.start_animation_timer = timer;
        self.refresh_inputs();
        if self.session().player.swing.start_animation_timer & 0x8000 != 0 {
            self.session_mut().player.swing.start_animation_timer = 5;
            self.refresh_inputs();
            let tiles = self
                .session()
                .player
                .swing
                .start_animation_tiles
                .wrapping_add(512);
            self.session_mut().player.swing.start_animation_tiles = tiles;
            self.refresh_inputs();
            let tiles = self.session().player.swing.start_animation_tiles;
            if i16::from_le_bytes(tiles.to_le_bytes()).wrapping_sub(i16::from_le_bytes(
                crate::grapple_aim::point_animation(self).end.to_le_bytes(),
            )) >= 0
            {
                let first = crate::grapple_aim::point_animation(self).first;
                self.session_mut().player.swing.start_animation_tiles = first;
                self.refresh_inputs();
            }
        }
        let mut cursor = self.session().queued_tilemaps.tail;
        self.prepared_queue_native_word(cursor, 32);
        cursor = cursor.wrapping_add(2);
        let source = self.session().player.swing.start_animation_tiles;
        self.prepared_queue_native_word(cursor, source);
        cursor = cursor.wrapping_add(2);
        self.prepared_queue_bank(cursor, 0x9a);
        cursor = cursor.wrapping_add(1);
        self.prepared_queue_native_word(cursor, 0x6200);
        cursor = cursor.wrapping_add(2);
        self.prepared_queue_tail(cursor);
        let angle = self.session().player.grapple_angle.to_le_bytes()[1];
        let index = ((angle >> 1) & 0xfe) >> 1;
        self.prepared_queue_native_word(cursor, 128);
        cursor = cursor.wrapping_add(2);
        let source = crate::gameplay_tables::grapple_point_tile(self, isize::from(index));
        self.prepared_queue_native_word(cursor, source);
        cursor = cursor.wrapping_add(2);
        self.prepared_queue_bank(cursor, 0x9a);
        cursor = cursor.wrapping_add(1);
        self.prepared_queue_native_word(cursor, 0x6210);
        cursor = cursor.wrapping_add(2);
        self.prepared_queue_tail(cursor);
        if self.session().projectiles.controls.charge.wrapping_sub(120) & 0x8000 != 0 {
            let charge = self.session().projectiles.controls.charge.wrapping_add(1);
            self.session_mut().projectiles.controls.charge = charge;
            self.refresh_inputs();
        }
    }
    fn prepared_finish_frame(&mut self) {
        let inputs = self.inputs().clone();
        crate::player_transitions::input::finish_input(self.session_mut(), &inputs);
        self.write_current_slice(
            crate::ram::words::JOYPAD1_INPUT_SAMUSFILTER,
            &inputs.held.to_le_bytes(),
        );
        self.write_current_slice(
            crate::ram::words::JOYPAD1_NEWINPUT_SAMUSFILTER,
            &inputs.pressed.to_le_bytes(),
        );
        self.guarded_actor_call(crate::player_projectiles::feedback::movement_sound);
        if self.current_word(crate::ram::words::ENABLE_DEBUG) != 0 {
            if matches!(self.session().player.pose, 0 | 0x9b) {
                if self.current_word(crate::ram::words::JOYPAD2_LAST) & 0x30 == 0x30
                    && self.current_word(crate::ram::words::JOYPAD2_NEW_KEYS) & 0x80 != 0
                {
                    self.write_current_slice(
                        crate::ram::words::DEBUG_INVINCIBILITY,
                        &7u16.to_le_bytes(),
                    );
                }
            } else {
                if i16::from_le_bytes(
                    self.current_word(crate::ram::words::DEBUG_INVINCIBILITY)
                        .wrapping_sub(7)
                        .to_le_bytes(),
                ) >= 0
                {
                    return;
                }
                self.write_current_slice(
                    crate::ram::words::DEBUG_INVINCIBILITY,
                    &0u16.to_le_bytes(),
                );
            }
        }
        if self.guarded_actor_call(|host| crate::progress::event_happened(host, 0xe))
            && self.session().player.transitions.frame_gamma
                == crate::player_transitions::state::FrameGammaMode::DrawEscapeTimer
            && self.current_word(0x998) != 35
        {
            self.write_current_slice(0x998, &35u16.to_le_bytes());
        }
    }
}
