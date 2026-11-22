//! The complete scoped opportunity and the external surrounding drawing stores.

mod checkpoint;

use super::{AfterActors, Origin, advance, carry, draw, follow, missing, place};
use crate::production::{ProductionOutputs, Unsupported, gameplay_sounds::GameplaySounds};
use crate::{
    production::{MovementContent, MovementInputs, ProductionFault, weapons::ShotDrawingOutputs},
    state::video::{NmiBatch, OAM_HIGH_BYTES, OAM_SPRITES, Oam, SpriteCount},
    weapons_projectiles_bridge::PowerContent,
};
use canonical::Canonical;
use data::assets::samus::key::Dress;
use types::{Parity, Pose};
use weapons_projectiles::{FlickerPhase, Inheritance, ScreenOrigin};

/// Fixed, admitted data reused for each advancing and restored opportunity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Independently admitted opening context; it supplies immutable values on restoration.
    pub admitted: Origin,
    /// Imported pose information.
    pub movement: MovementContent,
    /// The independently admitted nonchanging costume.
    pub dress: Dress,
    /// Imported ordinary power beam and muzzle content.
    pub power: PowerContent,
}

/// The current external room geometry and Camera setting.
#[derive(Clone, Copy)]
pub struct Environment<'a> {
    /// Checked terrain and reached contact world.
    pub world: &'a dyn types::World,
    /// Retained room consumed by Camera.
    pub camera_room: &'a world_camera::Room,
    /// Current Camera lead setting.
    pub camera_lead: world_camera::LeadSetting,
}

/// Combination of admitted content and current external dependencies.
pub(in crate::production) struct Context<'a> {
    /// Imported pose information.
    pub movement: &'a MovementContent,
    /// The independently admitted nonchanging costume.
    pub dress: Dress,
    /// Imported ordinary power beam and muzzle content.
    pub power: &'a PowerContent,
    /// External room and Camera setting.
    pub environment: Environment<'a>,
}

impl Content {
    pub(in crate::production) const fn at<'a>(
        &'a self,
        environment: Environment<'a>,
    ) -> Context<'a> {
        Context {
            movement: &self.movement,
            dress: self.dress,
            power: &self.power,
            environment,
        }
    }
}

/// One store by an enclosing producer before the live projectile insertion.
#[derive(Clone, Copy, Debug)]
pub struct DrawStore {
    /// Byte offset in the OAM low table, high table and little-endian byte cursor.
    pub offset: usize,
    /// Qualified operation at this occurrence.
    pub operation: DrawOp,
}

/// Actual site semantics for a surrounding producer's OAM write.
#[derive(Clone, Copy, Debug)]
pub enum DrawOp {
    /// Replace the addressed byte.
    Set(u8),
    /// OR the addressed byte with these bits.
    Or(u8),
}

/// Current causal inputs supplied independently of the selected owners.
#[derive(Clone, Copy)]
pub struct Inputs<'a> {
    /// Physical pad presented to Controller before PRE.
    pub sampled: u16,
    /// Presentation phase supplied by the enclosing drawing caller.
    pub phase: FlickerPhase,
    /// Ordered external stores performed before the power beam's insertion.
    pub before: &'a [DrawStore],
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
enum Audio {
    BeforeOffers(GameplaySounds),
    BeforeService(GameplaySounds),
}

impl Audio {
    const fn sounds(&self) -> &GameplaySounds {
        match self {
            Self::BeforeOffers(sounds) | Self::BeforeService(sounds) => sounds,
        }
    }
}

/// Complete current selected owners, held between ordinary opportunities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Continuation {
    origin: Origin,
    player: samus_movement::Samus,
    camera: world_camera::Camera,
    stimuli: MovementInputs,
    batch: NmiBatch,
    audio: Option<Audio>,
}

/// Carried facts read by W1 before POST consumes the current motion interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ingress {
    /// Previous pose eligible to supply the fallback aim.
    pub prior_pose: Pose,
    /// Actual travelled distances and Camera fraction.
    pub inheritance: Inheritance,
    /// View origin before the current Camera follow.
    pub view: ScreenOrigin,
}

impl Ingress {
    const fn read(origin: &Origin) -> Self {
        Self {
            prior_pose: origin.prior_pose,
            inheritance: origin.weapon.inheritance,
            view: origin.weapon.camera,
        }
    }
}

/// Actual publications and attributed drawings at their consumed boundaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    /// Actual feeding owner facts read by W1.
    pub ingress: Ingress,
    /// Before POST, after interleaved contact and door runs.
    pub actors: AfterActors,
    /// Newly emitted Movement receipts.
    pub movement: samus_movement::Outcome,
    /// Presentation-time Camera.
    pub camera: world_camera::Camera,
    /// Actual OAM backing at the power encoder's entrance.
    pub pre_power: Oam,
    /// The exact insertion and finalised carried backing.
    pub drawing: ShotDrawingOutputs,
    /// Actual power encoder's return cursor, in sprites.
    pub after_power: u16,
}

impl Continuation {
    /// Assemble the initially admitted owners at the same current opportunity.
    #[must_use]
    pub const fn new(
        origin: Origin,
        player: samus_movement::Samus,
        camera: world_camera::Camera,
        stimuli: MovementInputs,
        batch: NmiBatch,
    ) -> Self {
        Self {
            origin,
            player,
            camera,
            stimuli,
            batch,
            audio: None,
        }
    }

    /// Attach the three admitted sound senders to this producer path.
    #[must_use]
    pub fn with_audio(mut self, audio: GameplaySounds) -> Self {
        self.audio = Some(Audio::BeforeOffers(audio));
        self
    }

    /// The live sound owners carried with the producer, when admitted.
    #[must_use]
    pub const fn audio(&self) -> Option<&GameplaySounds> {
        match &self.audio {
            Some(audio) => Some(audio.sounds()),
            None => None,
        }
    }

    /// Stored OAM backing and cursor at the completed boundary.
    #[must_use]
    pub const fn batch(&self) -> &NmiBatch {
        &self.batch
    }

    /// Current pre-motion owner facts consumed by the next update.
    #[must_use]
    pub const fn before(&self) -> &Origin {
        &self.origin
    }

    pub(in crate::production) fn validate(
        &self,
        content: &Content,
    ) -> Result<(), crate::production::AdmissionFault> {
        use crate::production::AdmissionFault;
        content.movement.validate()?;
        if let Some(audio) = self.audio() {
            audio.validate()?;
        }
        content
            .power
            .validate_state(&self.origin.shots)
            .map_err(AdmissionFault::Weapons)?;
        // Restore reconstructs these W1 operands from the same retained owners.
        let muzzles = super::muzzles(
            self.player.pose,
            self.origin.prior_pose,
            content.dress,
            &content.power,
        )
        .map_err(|_| AdmissionFault::ShotOpportunity)?;
        let mut derived = self.origin.clone();
        super::refresh(&mut derived, &self.player, &self.camera, muzzles);
        derived.weapon.inheritance.camera_vertical_fraction.0 = self.camera.vertical_fraction().0;
        if !self.origin.same_immutable(&content.admitted)
            || self.origin != derived
            || self.player.pose != self.origin.pose
            || self.player.run.speed != self.origin.horizontal_speed
            || usize::from(self.batch.sprites.0) > OAM_SPRITES
        {
            return Err(AdmissionFault::ShotOpportunity);
        }
        Ok(())
    }

    /// Run the selected owners in order, and publish one complete next opportunity.
    ///
    /// # Errors
    /// An unsupported owner relation or surrounding draw store rejects the opportunity.
    pub(in crate::production) fn advance(
        &mut self,
        input: Inputs<'_>,
        context: &Context<'_>,
    ) -> Result<ProductionOutputs, ProductionFault> {
        let ingress = Ingress::read(&self.origin);
        let mut audio = match &self.audio {
            Some(Audio::BeforeOffers(audio)) => Some(audio.clone()),
            Some(Audio::BeforeService(_)) => {
                return Err(ProductionFault::Unsupported(Unsupported::SenderService));
            }
            None => None,
        };
        let (actors, offers) = advance(
            &self.origin,
            input.sampled,
            context.movement,
            context.power,
            audio.as_mut(),
        )?;
        let mut player = self.player;
        let movement = place(
            &actors,
            &mut player,
            self.stimuli,
            context.environment.world,
            context.movement,
            &samus_movement::Rules::default(),
        )?;
        let mut camera = self.camera;
        follow(
            &mut camera,
            &player,
            &movement,
            context.environment.camera_room,
            context.environment.camera_lead,
            &world_camera::Rules::REFERENCE,
        );
        let mut batch = self.batch.clone();
        apply_stores(&mut batch, input.before)?;
        let pre_power = batch.oam;
        let (drawing, after_power) = draw(&actors, &camera, input.phase, &batch, context.power)?;
        carry(
            &mut self.origin,
            &actors,
            &player,
            &movement,
            &camera,
            context.dress,
            context.power,
        )?;
        self.player = player;
        self.audio = audio.map(Audio::BeforeService);
        self.camera = camera;
        self.batch = drawing.batch.clone();
        self.stimuli.sprite_before_movement = movement.sprite;
        self.stimuli.parity = match self.stimuli.parity {
            Parity::Odd => Parity::Even,
            Parity::Even => Parity::Odd,
        };
        let shot = Box::new(Completion {
            ingress,
            actors,
            movement,
            camera,
            pre_power,
            drawing,
            after_power,
        });
        Ok(if self.audio.is_some() {
            ProductionOutputs::ShotOpportunityWithAudio { shot, offers }
        } else {
            ProductionOutputs::ShotOpportunity(shot)
        })
    }
}

fn apply_stores(batch: &mut NmiBatch, stores: &[DrawStore]) -> Result<(), ProductionFault> {
    const LOW_BYTES: usize = OAM_SPRITES * 4;
    const HIGH_END: usize = LOW_BYTES + OAM_HIGH_BYTES;
    let mut cursor = batch
        .sprites
        .0
        .checked_mul(4)
        .ok_or_else(missing)?
        .to_le_bytes();
    for store in stores {
        let byte = if store.offset < LOW_BYTES {
            let sprite = batch
                .oam
                .low
                .get_mut(store.offset / 4)
                .ok_or_else(missing)?;
            match store.offset % 4 {
                0 => &mut sprite.x,
                1 => &mut sprite.y,
                2 => &mut sprite.tile,
                _ => &mut sprite.attributes,
            }
        } else if store.offset < HIGH_END {
            batch
                .oam
                .high
                .get_mut(store.offset.saturating_sub(LOW_BYTES))
                .ok_or_else(missing)?
        } else {
            cursor
                .get_mut(store.offset.saturating_sub(HIGH_END))
                .ok_or_else(missing)?
        };
        match store.operation {
            DrawOp::Set(value) => *byte = value,
            DrawOp::Or(mask) => *byte |= mask,
        }
    }
    let cursor = usize::from(u16::from_le_bytes(cursor));
    if cursor > LOW_BYTES || !cursor.is_multiple_of(4) {
        return Err(missing());
    }
    batch.sprites = SpriteCount(u16::try_from(cursor / 4).map_err(|_| missing())?);
    Ok(())
}
