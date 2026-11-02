//! Exclusive prepared actor entry and the retained Windows resource boundary.
//! This port does not dispatch actor gameplay through Game or maintain a RAM mirror.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use sm_game::{
    actors::owned_execution::{
        ActorAllocations, ImportedActorAllocation, OpenError, PreparedActorState,
        ProjectileRecordSource,
    },
    persistent_session::PersistentSession,
    scene::{
        actor_presentation::Diagnostics,
        records::{ObjectContext, ObjectSelection},
        state::DisplayError,
    },
    session::Session,
};
use std::{rc::Rc, sync::Arc};

pub(crate) struct PreparedActorPorts<'g> {
    pub(crate) game: &'g mut Game,
    publications: Arc<sm_game::session_owners::SamusPublications>,
}
impl PreparedActorPorts<'_> {
    /// Only the finite native resource/effect seams use this compatibility boundary.
    /// The supplied canonical owner remains the same allocation throughout the call.
    pub(crate) fn physical_effect<R>(
        &mut self,
        owner: &mut PersistentSession,
        operation: impl FnOnce(&mut Game) -> R,
    ) -> R {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.game
                .prepared_actor_physical_effect(&mut owner.gameplay, operation)
        }));
        owner.observations =
            sm_game::session_encoding::inputs_from_session(&owner.gameplay, &|at| {
                self.game.ram().read_byte(at)
            });
        match outcome {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }
    pub(super) fn refresh_allocations(
        &mut self,
        _owner: &PersistentSession,
        allocations: &mut ActorAllocations,
    ) {
        // Callers invoke this only after the original diagnostic and capture.
        allocations.replace_sources(
            self.game.loaded_rom.clone(),
            self.game.capture_native_rom_binding(),
            Rc::new(self.game.enemy_speeds.clone()),
            Rc::new(self.game.records.clone()),
        );
    }
    pub(super) fn physical_slope_byte(
        &mut self,
        owner: &mut PersistentSession,
        block: u16,
        lane: u8,
    ) -> u8 {
        let address = 0x7f6402u32 + u32::from(block) + u32::from(lane);
        self.physical_effect(owner, |game| {
            crate::game::session_runtime::prepared_actor_bus_byte(address, game)
        })
    }
    pub(super) fn physical_music_effect(
        &mut self,
        owner: &mut PersistentSession,
        effect: sm_game::music_requests::MusicEffect,
        meaning: Option<&sm_game::session_owners::QueuedMusicMeaning>,
    ) -> Result<(), DisplayError> {
        let selector = match effect {
            sm_game::music_requests::MusicEffect::UploadBank => Some(sm_data::OriginalMusicBank(
                owner.gameplay.control.requests()?.music.entry.to_le_bytes()[0],
            )),
            sm_game::music_requests::MusicEffect::WriteTrack(_) => None,
        };
        self.physical_effect(owner, |game| {
            crate::game::session_runtime::prepared_music_effect(selector, effect, meaning, game)
        });
        Ok(())
    }
}
impl ImportedActorAllocation for PreparedActorPorts<'_> {
    fn presentation_diagnostics(&mut self) -> &mut dyn Diagnostics {
        self
    }
    fn dialogue_controller_word(&mut self, owner: &mut PersistentSession, register: u16) -> u16 {
        self.physical_effect(owner, |game| {
            crate::game::session_runtime::prepared_dialogue_controller_word(register, game)
        })
    }
    fn dialogue_request_reset(&mut self, owner: &mut PersistentSession) {
        self.physical_effect(
            owner,
            crate::game::session_runtime::prepared_dialogue_request_reset,
        );
    }
    fn low_window_projectile_record(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut ActorAllocations,
        source: ProjectileRecordSource,
        bank: u8,
        address: u16,
    ) -> sm_data::cartridge::RomRecord {
        let absolute = (u32::from(bank) << 16) | u32::from(address);
        let record = match source {
            ProjectileRecordSource::PlayerLoaded => self.physical_effect(owner, |game| {
                crate::game::session_runtime::prepared_loaded_actor_record(bank, address, game)
            }),
            ProjectileRecordSource::PlayerNative | ProjectileRecordSource::EnemyNative => {
                self.physical_effect(owner, |game| {
                    crate::game::session_runtime::invalid_window(game, absolute)
                });
                sm_data::cartridge::RomRecord {
                    image: self.game.capture_native_rom(),
                    base: usize::try_from(sm_data::cartridge::lorom_offset(absolute))
                        .expect("native record offset"),
                }
            }
            ProjectileRecordSource::EnemyLoaded => {
                self.physical_effect(owner, |game| {
                    crate::game::session_runtime::invalid_window(game, absolute)
                });
                sm_data::cartridge::RomRecord {
                    image: self
                        .game
                        .loaded_rom
                        .clone()
                        .expect("loaded projectile allocation"),
                    base: usize::try_from(sm_data::cartridge::lorom_offset(absolute))
                        .expect("loaded record offset"),
                }
            }
        };
        self.refresh_allocations(owner, allocations);
        record
    }
    fn report_invalid_decompression_address(
        &mut self,
        owner: &mut PersistentSession,
        address: u32,
    ) {
        self.game
            .report_owned_invalid_decompression_address(owner, address);
    }
    fn actor_alias_byte(&self, _state: &Session, at: usize) -> u8 {
        // Exceptional full/odd/native addresses resolve the actual imported bus allocation.
        self.game.ram().read_byte(at)
    }
    fn set_actor_alias_byte(&mut self, _state: &mut Session, at: usize, value: u8) {
        self.game.ram().write_byte(at, value);
    }
    fn mechanism_diagnostic(
        &mut self,
        owner: &mut PersistentSession,
        diagnostic: sm_game::room_mechanisms::host::Diagnostic,
    ) -> u8 {
        self.physical_effect(owner, |game| {
            crate::game::room_mechanisms::prepared_actor_diagnostic(diagnostic, game)
        })
    }
    fn low_window_native_actor_record(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut ActorAllocations,
        bank: u8,
        address: u16,
    ) -> sm_data::cartridge::RomRecord {
        let record = self.physical_effect(owner, |game| {
            crate::game::session_runtime::native_record(bank, address, game)
        });
        self.refresh_allocations(owner, allocations);
        record
    }
    fn low_window_actor_record(
        &mut self,
        owner: &mut PersistentSession,
        allocations: &mut ActorAllocations,
        bank: u8,
        address: u16,
    ) -> sm_data::cartridge::RomRecord {
        let record = self.physical_effect(owner, |game| {
            crate::game::session_runtime::prepared_loaded_actor_record(bank, address, game)
        });
        self.refresh_allocations(owner, allocations);
        record
    }
}
impl sm_game::actors::remaining_owned::bindings::Ports for PreparedActorPorts<'_> {
    fn kraid_flags_observer(&mut self, owner: &mut PersistentSession, step: u8) {
        self.physical_effect(owner, |game| {
            crate::game::hosts::observe_kraid_flags(step, game)
        });
    }
    fn remaining_actor_diagnostic(
        &mut self,
        owner: &mut PersistentSession,
        diagnostic: sm_game::actors::remaining_owned::bindings::Diagnostic,
    ) {
        self.physical_effect(owner, |game| {
            crate::game::session_runtime::prepared_remaining_actor_diagnostic(diagnostic, game)
        });
    }
}
impl Diagnostics for PreparedActorPorts<'_> {
    fn invalid_sprite(
        &mut self,
        session: &mut Session,
        source: sm_game::scene::actor_presentation::SpriteAllocation,
        bank: u8,
        address: u16,
    ) -> Result<(), DisplayError> {
        // Native and loaded sprite diagnostics have distinct observer prefixes.
        self.game.prepared_actor_physical_effect(session, |game| {
            crate::game::session_runtime::prepared_sprite_diagnostic(source, bank, address, game)
        });
        Ok(())
    }
    fn loaded_image(&self) -> sm_data::cartridge::RomImage {
        self.game
            .loaded_rom
            .clone()
            .expect("prepared sprite loaded allocation")
    }
    fn native_image(&self) -> sm_data::cartridge::RomImage {
        self.game.capture_native_rom()
    }
    fn alias_byte(&self, _session: &Session, at: usize) -> Result<u8, DisplayError> {
        if at >= 0x20000 {
            return Err(DisplayError::Source);
        }
        Ok(self.game.ram().read_byte(at))
    }
    fn write_alias_byte(
        &mut self,
        _session: &mut Session,
        at: usize,
        value: u8,
    ) -> Result<(), DisplayError> {
        if at >= 0x20000 {
            return Err(DisplayError::Source);
        }
        self.game.ram().write_byte(at, value);
        Ok(())
    }
    fn object_context(
        &mut self,
        owner: &sm_game::scene::commit::OwnedDisplay,
    ) -> Result<ObjectContext, DisplayError> {
        let state = &owner.state;
        Ok(ObjectContext {
            selection: ObjectSelection {
                atlases: *self
                    .publications
                    .object_atlases
                    .get(&state.controls.object_layout)
                    .ok_or(DisplayError::Incomplete)?,
                size_set: state.controls.object_layout >> 5,
                first_order: self.publications.object_first_order,
                interlaced: state.raster.interlaced,
                even_frame: state.raster.even_frame,
            },
            palette: owner.image().palette,
        })
    }
    fn draw_echo_sprite(
        &mut self,
        session: &mut Session,
        observations: &sm_game::external_inputs::ExternalInputs,
        content: &sm_game::scene::prepared_samus::PreparedSamusContent,
        echo: usize,
        index: u16,
        position: sm_game::geometry::Point,
    ) -> Result<(), DisplayError> {
        sm_game::scene::prepared_samus::draw_echo_sprite_session(
            session,
            observations,
            content,
            echo,
            index,
            position,
        )
    }
}

impl Game {
    /// Open once at the actual actor caller; the same owner survives suspension.
    pub(crate) fn prepared_actor_call<R>(
        &mut self,
        operation: impl FnOnce(&mut PreparedActorState<'_>) -> R,
    ) -> Result<R, OpenError> {
        self.revoke_prepared_message_frame();
        let revision = self
            .world()
            .actors()
            .revision()
            .ok_or(OpenError::ActorDefinitions)?;
        let mut allocations = ActorAllocations::new(
            self.world(),
            self.loaded_rom.clone().ok_or(OpenError::ActorDefinitions)?,
            self.capture_native_rom_binding()
                .ok_or(OpenError::ActorDefinitions)?,
            Rc::new(self.enemy_speeds.clone()),
            Rc::new(self.records.clone()),
            revision,
        )?;
        let mut owner = self
            .lease_persistent_session()
            .map_err(|_| OpenError::Presentation)?
            .ok_or(OpenError::World)?;
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let content = owner.actor_content.clone().ok_or(OpenError::Presentation)?;
            let sm_game::session_owners::Presentation::Owned {
                samus_publications, ..
            } = &owner.gameplay.presentation
            else {
                return Err(OpenError::Presentation);
            };
            let publications = samus_publications.clone();
            let mut ports = PreparedActorPorts {
                game: self,
                publications,
            };
            let mut host = PreparedActorState::open(
                &mut owner,
                &mut allocations,
                &mut ports,
                content.binding(),
            )?;
            Ok(operation(&mut host))
        }));
        // Compatibility snapshots retain a valid typed prefix even when the
        // operation unwinds. Completion separately governs device publication
        // and resumption, as in Runtime::relinquish on the same owner.
        let eligible = Arc::ptr_eq(&owner.gameplay.world, self.world())
            && owner.gameplay.world.revision() == self.world().revision()
            && sm_game::fx_encoding::validate_control_ownership(&owner.gameplay).is_ok()
            && self.preflight_samus_runtime().is_ok();
        let completed = eligible
            && matches!(&outcome, Ok(Ok(_)))
            && owner.gameplay.presentation.owner().is_ok_and(|display| {
                display.state.phase != sm_game::scene::state::DisplayPhase::Failed
            });
        let publication = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if eligible {
                sm_game::session_encoding::encode(&owner.gameplay, |at, bytes| {
                    self.ram().copy_in(at, bytes)
                });
                crate::game::session_runtime::export_native_spawn(&owner.gameplay, self);
                if completed {
                    self.export_owned_hardware(&owner.gameplay);
                }
            }
        }));
        let epoch = self.compatibility_write_epoch();
        let returned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.return_persistent_session(owner, completed && publication.is_ok(), epoch)
        }));
        if returned.is_err() {
            self.fail_prepared_actor_return();
        }
        self.finalize_prepared_message_frame();
        match outcome {
            Err(payload) => std::panic::resume_unwind(payload),
            Ok(value) => match publication {
                Err(payload) => std::panic::resume_unwind(payload),
                Ok(()) => match returned {
                    Ok(()) => value,
                    Err(payload) => std::panic::resume_unwind(payload),
                },
            },
        }
    }
}
