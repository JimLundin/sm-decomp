//! Prepared global Samus producers on the one current Session.
//! Immutable admission contains content only, never a second player or display.
#![forbid(unsafe_code)]
use crate::external_inputs::ExternalInputs;
use crate::geometry::Point;
use crate::persistent_session::PersistentSession;
use crate::scene::owned_samus::{self, DrawingSources};
use crate::scene::records::{ObjectContext, ObjectSelection};
use crate::scene::samus_body::{Body, Half};
use crate::scene::samus_content::{PreparedBytes, Registration, Role, RoleLease, ValidatedContent};
use crate::scene::samus_position_tiles::TileRequest;
use crate::scene::state::{DisplayError, DisplayPhase};
use crate::session::Session;
use crate::session_owners::{Presentation, SessionStartRequirements};
use std::sync::Arc;

/// Held by the exclusive prepared host binding. Reopen admits a new binding.
pub struct PreparedSamusContent {
    roles: ValidatedContent,
    required: Arc<SessionStartRequirements>,
    world: Arc<crate::room_entry::world::WorldBinding>,
}
impl PreparedSamusContent {
    /// Admission uses the current initialized owner and its independent contract.
    /// It never initializes player state, advances a timer or sends an upload.
    pub fn prepare(owner: &PersistentSession) -> Result<Self, DisplayError> {
        let display = owner.gameplay.presentation.owner()?;
        let required = owner.required.clone();
        let metadata = owner.gameplay.control.metadata()?;
        metadata.samus_content.validate(display.media())?;
        metadata
            .samus_content
            .validate_expected(&required.display.samus_content)?;
        let mut requests = required.roles.requests.iter();
        let roles = ValidatedContent::construct(owner.roles.clone(), &required.roles, |request| {
            request.validate(
                requests
                    .next()
                    .ok_or(DisplayError::Incomplete)?
                    .request
                    .pointer,
                display.media(),
            )
        })?;
        // The normal echo path reads the independently loaded registration.
        roles.acquire_prepared(Role::PositionFallback, Registration::Loaded, 0)?;
        let result = Self {
            roles,
            required,
            world: owner.gameplay.world.clone(),
        };
        result.check_identity(&owner.gameplay)?;
        Ok(result)
    }
    pub fn check(&self, owner: &PersistentSession) -> Result<(), DisplayError> {
        if !Arc::ptr_eq(&self.required, &owner.required)
            || !self.roles.matches_description(&owner.roles)
        {
            return Err(DisplayError::Identity);
        }
        self.check_session(&owner.gameplay)
    }
    /// The parent host checks `check` before exposing split Session borrows.
    /// Every query thereafter uses this Session's actual player/display fields.
    pub fn check_session(&self, session: &Session) -> Result<(), DisplayError> {
        let display = session.presentation.owner()?;
        if display.state.phase == DisplayPhase::Failed {
            return Err(DisplayError::Poisoned);
        }
        if display.state.phase != DisplayPhase::Producing {
            return Err(DisplayError::Phase);
        }
        self.check_identity(session)
    }
    fn check_identity(&self, session: &Session) -> Result<(), DisplayError> {
        if !Arc::ptr_eq(&self.world, &session.world)
            || session.control.metadata()?.samus_content.as_ref()
                != self.required.display.samus_content.as_ref()
        {
            return Err(DisplayError::Identity);
        }
        Ok(())
    }
}
struct Sources<'a>(&'a PreparedSamusContent);
impl DrawingSources for Sources<'_> {
    fn prepared_role(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> Result<Option<RoleLease<PreparedBytes>>, DisplayError> {
        self.0
            .roles
            .acquire_prepared(role, registration, displacement)
            .map(Some)
    }
    fn role_base(
        &self,
        role: Role,
        registration: Registration,
    ) -> Result<Option<usize>, DisplayError> {
        self.0
            .roles
            .description()
            .roles
            .get(&(role, registration))
            .map(|role| Some(role.mapped_base))
            .ok_or(DisplayError::Incomplete)
    }
    fn request(&self, half: Half, pointer: u16) -> Result<Option<TileRequest>, DisplayError> {
        self.0.roles.request(half, pointer).map(Some)
    }
    fn spritemaps(&self) -> Result<Option<Arc<crate::scene::samus::SamusCatalog>>, DisplayError> {
        Ok(Some(self.0.required.display.samus_content.clone()))
    }
    fn diagnostic(&mut self, _: u32) -> Result<(), DisplayError> {
        // The original low-window diagnostic can mutate the current owner.
        // No prepared implementation of that exceptional external edge exists.
        Err(DisplayError::Source)
    }
}

/// Guards the actual aggregate, preserving completed prefixes during unwinding.
struct Guard<'a> {
    session: &'a mut Session,
    armed: bool,
}
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if self.armed {
            if let Ok(display) = self.session.presentation.owner_mut() {
                display.state.phase = DisplayPhase::Failed;
            }
            self.session.actor_identity.retain_poison();
        }
    }
}
fn operate<T>(
    session: &mut Session,
    content: &PreparedSamusContent,
    action: impl FnOnce(&mut Session, &mut Sources<'_>) -> Result<T, DisplayError>,
) -> Result<T, DisplayError> {
    let mut guard = Guard {
        session,
        armed: true,
    };
    content.check_session(guard.session)?;
    let result = action(guard.session, &mut Sources(content));
    guard.armed = result.is_err();
    result
}

pub fn draw_body_session(
    session: &mut Session,
    observations: &ExternalInputs,
    content: &PreparedSamusContent,
    mode: Body,
) -> Result<(), DisplayError> {
    operate(session, content, |session, sources| {
        let Presentation::Owned {
            display,
            marker,
            samus_publications,
            ..
        } = &mut session.presentation
        else {
            return Err(DisplayError::Phase);
        };
        owned_samus::draw_body(
            display.owner_mut(),
            sources,
            &mut session.player,
            observations,
            samus_publications.clone(),
            marker,
            mode,
        )?;
        // This is a second fresh table/pose/frame query, after both tile requests.
        let byte = owned_samus::definition_byte(
            display.owner_mut(),
            sources,
            &mut session.player,
            observations,
            samus_publications.clone(),
        )?;
        session.spawn_scratch.argument = u16::from(byte);
        Ok(())
    })
}
pub fn bottom_visible_session(
    session: &mut Session,
    observations: &ExternalInputs,
    content: &PreparedSamusContent,
) -> Result<bool, DisplayError> {
    operate(session, content, |session, sources| {
        let Presentation::Owned {
            display,
            marker,
            samus_publications,
            ..
        } = &mut session.presentation
        else {
            return Err(DisplayError::Phase);
        };
        owned_samus::bottom_visible(
            display.owner_mut(),
            sources,
            &mut session.player,
            observations,
            samus_publications.clone(),
            marker,
        )
    })
}
fn sprite_session(
    session: &mut Session,
    observations: &ExternalInputs,
    content: &PreparedSamusContent,
    echo: Option<usize>,
    index: u16,
    position: Point,
) -> Result<(), DisplayError> {
    operate(session, content, |session, sources| {
        let publications = session.presentation.publications()?.clone();
        let producer = match echo {
            None => publications.body,
            Some(slot) => publications
                .echoes
                .get(slot)
                .copied()
                .flatten()
                .ok_or(DisplayError::Incomplete)?,
        };
        owned_samus::draw_sprite(
            session.presentation.owner_mut()?,
            sources,
            &mut session.player,
            observations,
            publications,
            owned_samus::Sprite {
                producer,
                index,
                position: (position.x, position.y),
            },
        )
    })
}
pub fn draw_sprite_session(
    session: &mut Session,
    observations: &ExternalInputs,
    content: &PreparedSamusContent,
    index: u16,
    position: Point,
) -> Result<(), DisplayError> {
    sprite_session(session, observations, content, None, index, position)
}
pub fn draw_echo_sprite_session(
    session: &mut Session,
    observations: &ExternalInputs,
    content: &PreparedSamusContent,
    slot: usize,
    index: u16,
    position: Point,
) -> Result<(), DisplayError> {
    sprite_session(session, observations, content, Some(slot), index, position)
}
pub fn draw_body(
    owner: &mut PersistentSession,
    content: &PreparedSamusContent,
    mode: Body,
) -> Result<(), DisplayError> {
    if let Err(error) = content.check(owner) {
        owner.poison();
        return Err(error);
    }
    draw_body_session(&mut owner.gameplay, &owner.observations, content, mode)
}
pub fn draw_sprite(
    owner: &mut PersistentSession,
    content: &PreparedSamusContent,
    index: u16,
    position: Point,
) -> Result<(), DisplayError> {
    if let Err(error) = content.check(owner) {
        owner.poison();
        return Err(error);
    }
    draw_sprite_session(
        &mut owner.gameplay,
        &owner.observations,
        content,
        index,
        position,
    )
}
pub fn draw_echo_sprite(
    owner: &mut PersistentSession,
    content: &PreparedSamusContent,
    slot: usize,
    index: u16,
    position: Point,
) -> Result<(), DisplayError> {
    if let Err(error) = content.check(owner) {
        owner.poison();
        return Err(error);
    }
    draw_echo_sprite_session(
        &mut owner.gameplay,
        &owner.observations,
        content,
        slot,
        index,
        position,
    )
}

struct AtmosphereInputs<'a> {
    observations: &'a ExternalInputs,
    publications: Arc<crate::session_owners::SamusPublications>,
    palette: crate::scene::records::MediaRef,
    raster: crate::scene::state::RasterState,
}
impl crate::scene::atmosphere::Inputs for AtmosphereInputs<'_> {
    fn camera_x(&mut self, _: &crate::scene::state::DisplayState) -> Result<u16, DisplayError> {
        Ok(self.observations.camera_x)
    }
    fn camera_y(&mut self, _: &crate::scene::state::DisplayState) -> Result<u16, DisplayError> {
        Ok(self.observations.camera_y)
    }
    fn fx_y(&mut self, _: &crate::scene::state::DisplayState) -> Result<u16, DisplayError> {
        Ok(self.observations.fx_y)
    }
    fn object_context(
        &mut self,
        controls: &crate::scene::state::DisplayControls,
        _: &crate::scene::state::EffectiveDisplayDescription,
    ) -> Result<ObjectContext, DisplayError> {
        Ok(ObjectContext {
            selection: ObjectSelection {
                atlases: *self
                    .publications
                    .object_atlases
                    .get(&controls.object_layout)
                    .ok_or(DisplayError::Incomplete)?,
                size_set: controls.object_layout >> 5,
                first_order: self.publications.object_first_order,
                interlaced: self.raster.interlaced,
                even_frame: self.raster.even_frame,
            },
            palette: self.palette,
        })
    }
}
pub fn draw_atmospheric_tile(
    owner: &mut PersistentSession,
    content: &PreparedSamusContent,
    slot: usize,
    kind: u16,
) -> Result<(), DisplayError> {
    if let Err(error) = content.check(owner) {
        owner.poison();
        return Err(error);
    }
    operate(&mut owner.gameplay, content, |session, _| {
        let publications = session.presentation.publications()?.clone();
        let display = session.presentation.owner_mut()?;
        let media = display.media().clone();
        let mut inputs = AtmosphereInputs {
            observations: &owner.observations,
            publications,
            palette: display.image().palette,
            raster: display.state.raster,
        };
        crate::scene::atmosphere::draw_tile_owned(
            &mut display.state,
            &media,
            &mut inputs,
            slot,
            kind,
        )
    })
}

pub fn advance_atmosphere(
    owner: &mut PersistentSession,
    content: &PreparedSamusContent,
) -> Result<(), DisplayError> {
    if let Err(error) = content.check(owner) {
        owner.poison();
        return Err(error);
    }
    operate(&mut owner.gameplay, content, |session, _| {
        let publications = session.presentation.publications()?.clone();
        let display = session.presentation.owner_mut()?;
        let mut inputs = AtmosphereInputs {
            observations: &owner.observations,
            publications,
            palette: display.image().palette,
            raster: display.state.raster,
        };
        display.advance_atmosphere(&mut inputs)
    })
}

struct Echoes<'a> {
    session: &'a mut Session,
    observations: &'a ExternalInputs,
    content: &'a PreparedSamusContent,
}
impl crate::scene::player_echoes::Host for Echoes<'_> {
    type Error = DisplayError;
    fn session(&self) -> &Session {
        self.session
    }
    fn session_mut(&mut self) -> &mut Session {
        self.session
    }
    fn draw_echo_slot(&mut self, slot: u16) -> Result<(), DisplayError> {
        let index = usize::from(slot >> 1);
        // The native path captures Y before acquiring the loaded fallback table.
        let y = *self
            .session
            .projectiles
            .echoes
            .y
            .get(index)
            .ok_or(DisplayError::Alias {
                offset: usize::from(slot),
            })?;
        let table =
            self.content
                .roles
                .acquire_prepared(Role::PositionFallback, Registration::Loaded, 0)?;
        let height = table.byte(usize::from(self.session.player.pose) * 8 + 4)?;
        let y = y
            .wrapping_sub(u16::from(height))
            .wrapping_sub(self.observations.camera_y);
        if i16::from_le_bytes(y.to_le_bytes()) >= 0 && (y.wrapping_sub(248) & 0x8000) != 0 {
            let sprite = self.session.player.top_sprite;
            let x =
                self.session.projectiles.echoes.x[index].wrapping_sub(self.observations.camera_x);
            draw_echo_sprite_session(
                self.session,
                self.observations,
                self.content,
                index,
                sprite,
                Point { x, y },
            )?;
            if self.session.player.bottom_sprite != 0 {
                let sprite = self.session.player.bottom_sprite;
                let x = self.session.projectiles.echoes.x[index]
                    .wrapping_sub(self.observations.camera_x);
                draw_echo_sprite_session(
                    self.session,
                    self.observations,
                    self.content,
                    index,
                    sprite,
                    Point { x, y },
                )?;
            }
        }
        Ok(())
    }
    fn frame_counter(&self) -> u16 {
        self.observations.frame
    }
    fn sprite_index(&mut self, half: Half, pose: u16) -> Result<u16, DisplayError> {
        let frame = self.session.player.animation_clock.frame;
        let role = match half {
            Half::Top => Role::BodyTop,
            Half::Bottom => Role::BodyBottom,
        };
        let table = self
            .content
            .roles
            .acquire_prepared(role, Registration::Loaded, 0)?;
        Ok(frame.wrapping_add(table.native_word(usize::from(pose) * 2)?))
    }
    fn height(&mut self) -> Result<u8, DisplayError> {
        let table =
            self.content
                .roles
                .acquire_prepared(Role::PositionFallback, Registration::Native, 0)?;
        table.byte(usize::from(self.session.player.pose) * 8 + 4)
    }
    fn echo_x(&mut self, slot: u16) -> Result<u16, DisplayError> {
        self.session
            .projectiles
            .echoes
            .x
            .get(usize::from(slot >> 1))
            .copied()
            .ok_or(DisplayError::Alias {
                offset: usize::from(slot),
            })
    }
    fn echo_y(&mut self, slot: u16) -> Result<u16, DisplayError> {
        self.session
            .projectiles
            .echoes
            .y
            .get(usize::from(slot >> 1))
            .copied()
            .ok_or(DisplayError::Alias {
                offset: usize::from(slot),
            })
    }
    fn camera(&self) -> (u16, u16) {
        (self.observations.camera_x, self.observations.camera_y)
    }
    fn draw_sprite(&mut self, slot: u16, index: u16, x: u16, y: u16) -> Result<(), DisplayError> {
        draw_echo_sprite_session(
            self.session,
            self.observations,
            self.content,
            usize::from(slot >> 1),
            index,
            Point { x, y },
        )
    }
    fn bottom_visible(&mut self) -> Result<bool, DisplayError> {
        bottom_visible_session(self.session, self.observations, self.content)
    }
}
pub fn draw_echoes(
    owner: &mut PersistentSession,
    content: &PreparedSamusContent,
) -> Result<(), DisplayError> {
    if let Err(error) = content.check(owner) {
        owner.poison();
        return Err(error);
    }
    operate(&mut owner.gameplay, content, |session, _| {
        crate::scene::player_echoes::draw(&mut Echoes {
            session,
            observations: &owner.observations,
            content,
        })
    })
}
pub fn draw_crash_echoes(
    owner: &mut PersistentSession,
    content: &PreparedSamusContent,
    slot: u16,
) -> Result<(), DisplayError> {
    if let Err(error) = content.check(owner) {
        owner.poison();
        return Err(error);
    }
    operate(&mut owner.gameplay, content, |session, _| {
        crate::scene::player_echoes::crash(
            slot,
            &mut Echoes {
                session,
                observations: &owner.observations,
                content,
            },
        )
    })
}
