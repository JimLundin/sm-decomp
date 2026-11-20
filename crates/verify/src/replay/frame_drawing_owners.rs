//! Checked origin decoding for concrete borrowed drawing ports.

use game::production::frame::drawing_owners::{
    ActorDrawing, Cannon, Collections, Membership, Origins, Remaining, SceneDrawing, SpeedEchoes,
};
use serde::Deserialize;

/// Invalid pinned projection or unavailable actual owner provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A source identity or complete alias join differs.
    Source,
    /// The origin cannot establish the current unchanged owner.
    Origin,
    /// An active member requires another independently qualified port.
    Unsupported,
}

#[derive(Deserialize)]
struct Proof {
    format: String,
    expected_final_oam_or_tile_buffer_used_as_input: bool,
    normal_draw_identity_binding: String,
    current_premise: Vec<Premise>,
}

#[derive(Deserialize)]
struct Premise {
    name: String,
    private_wram_offset: usize,
    quantity_width: usize,
    source_use_index: usize,
    origin_equal_at_use: bool,
    current_source_encoding: u16,
}

#[derive(Deserialize)]
struct Requirements {
    format: String,
    source_projection_only: bool,
    generic_callbacks_allowed: bool,
    #[serde(flatten)]
    origins: CurrentOrigins,
    scalar: Vec<Scalar>,
    collection: Vec<Collection>,
}

#[derive(serde::Deserialize)]
struct CurrentOrigins {
    initial_owner_decoding_permitted: bool,
    actual_owner_updates_required: bool,
}

#[derive(Deserialize)]
struct Scalar {
    name: String,
    source_use_index: usize,
    private_wram_offset: usize,
    quantity_width: usize,
    actual_current_owner_required: bool,
    expected_encoding_is_not_runtime_initializer: bool,
}

#[derive(Deserialize)]
struct Collection {
    view: String,
    identity: usize,
    source_use_index: usize,
    private_wram_offset: usize,
    quantity_width: usize,
    initial_native_empty_is_not_runtime_proof: bool,
}

fn pin(text: &str, expected: &str) -> Result<(), Fault> {
    if crate::lock::Sha256::of(text.as_bytes())
        != crate::lock::Sha256::try_from(expected).map_err(|_| Fault::Source)?
    {
        return Err(Fault::Source);
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct HookOrigin {
    offset: usize,
    width: usize,
    origin_sha256: String,
}

pub(super) fn no_action_graphics_hook(
    wram: &[u8],
    source: &str,
    rom: &data::rom::Rom,
) -> Result<(), Fault> {
    pin(
        source,
        "30e82e309aa2e4023bddbe94b5f202286b24ed5591188fbafeaa55f4bfa4a9af",
    )?;
    let source: HookOrigin = toml::from_str(source).map_err(|_| Fault::Source)?;
    let origin =
        crate::lock::Sha256::try_from(source.origin_sha256.as_str()).map_err(|_| Fault::Source)?;
    if crate::lock::Sha256::of(wram) != origin || source.width != 3 {
        return Err(Fault::Origin);
    }
    let address = wram
        .get(
            source.offset
                ..source
                    .offset
                    .checked_add(source.width)
                    .ok_or(Fault::Source)?,
        )
        .ok_or(Fault::Origin)?;
    let address = data::rom::RomAddress::from_long(address.try_into().map_err(|_| Fault::Origin)?);
    // Return from the installed hook without any room-specific effect.
    if rom.lorom().read(address, 1).map_err(|_| Fault::Source)? != [0x6b] {
        return Err(Fault::Unsupported);
    }
    Ok(())
}

/// Decode sole borrowed initial owners, refusing source terminal collection inputs.
/// Actual S2 Projectiles/Sprites and Actor dispositions are joined later from completions.
/// No selected pose, playhead, current Camera or future output is imported here.
/// # Errors
/// Invalid source pins, incomplete aliases or active initial collection bodies.
pub fn origin(bytes: &[u8], proof: &str, requirements: &str) -> Result<Origins, Fault> {
    if crate::lock::Sha256::of(bytes)
        != crate::lock::Sha256::try_from(
            "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db",
        )
        .map_err(|_| Fault::Source)?
    {
        return Err(Fault::Origin);
    }
    pin(
        proof,
        "08e4fe71f9ccaefdd201cfd162b5d8fd175cfb2b679d9fad0570910556f76832",
    )?;
    pin(
        requirements,
        "5822b7fdc9755cb77cea2b58d709f91b0edb8ae44580566687c1e67b57db5889",
    )?;
    let proof: Proof = toml::from_str(proof).map_err(|_| Fault::Source)?;
    let requirements: Requirements = toml::from_str(requirements).map_err(|_| Fault::Source)?;
    if bytes.len() != 131_072
        || proof.format != "bounded-current-drawing-premises-v1"
        || proof.expected_final_oam_or_tile_buffer_used_as_input
        || proof.normal_draw_identity_binding != "ActualCurrentDrawingOpportunityIdentity"
        || requirements.format != "drawing-typed-owner-requirements-v1"
        || !requirements.source_projection_only
        || requirements.generic_callbacks_allowed
        || !requirements.origins.initial_owner_decoding_permitted
        || !requirements.origins.actual_owner_updates_required
        || proof.current_premise.len() != 253
        || requirements.scalar.len() != 40
        || requirements.collection.len() != 213
    {
        return Err(Fault::Source);
    }
    check_requirements(&proof, &requirements)?;
    let scalar = |name: &str| -> Result<u16, Fault> {
        let matches = proof
            .current_premise
            .iter()
            .filter(|premise| premise.name == name)
            .collect::<Vec<_>>();
        let first = matches.first().ok_or(Fault::Source)?;
        let value = initial(bytes, first)?;
        for premise in matches {
            if initial(bytes, premise)? != value {
                return Err(Fault::Origin);
            }
        }
        Ok(value)
    };
    Ok(Origins {
        actors: ActorDrawing {
            background_upload: scalar("EnemyBackgroundUploadRequest")? != 0,
            background_hook: scalar("EnemyGraphicsHookBank")? != 0,
            sprite_interactions_disabled: scalar("SpriteInteractionsDisabled")? != 0,
        },
        echoes: SpeedEchoes {
            merging: scalar("EchoMergeDisposition")? != 0,
            sound_playing: scalar("EchoSoundPlaying")? != 0,
        },
        scene: SceneDrawing {
            normal: {
                scalar("CurrentDrawingHandler")?;
                true
            },
            debug: scalar("CurrentDebugMode")? != 0,
            rotating: scalar("CeresRotationMode")? != 0,
        },
        collections: drawing_collections(bytes, &proof, &requirements, scalar)?,
        cannon: Cannon {
            charge_ticks: scalar("ChargeCounter")?,
            resume_charge_sound: scalar("ResumeChargeSound")? != 0,
            open: scalar("CurrentCannonOpen")? != 0,
            frame: scalar("CurrentCannonFrame")?,
            opening_closing: scalar("CannonOpeningClosing")? != 0,
        },
        frame_counter: scalar("CurrentFrameCounter")?,
        frozen: scalar("CurrentFreezeOwner")? != 0,
        invincibility: Remaining(scalar("CurrentInvincibilityTimer")?),
        knockback: Remaining(scalar("KnockbackTimer")?),
        shake: Remaining(scalar("ScreenShakeTimer")?),
        hyper_beam: scalar("HyperBeamMode")? != 0,
        timebomb: scalar("CurrentZebesTimebombEventByte")? != 0,
        power_bomb_radius: scalar("PowerBombRadius")?,
        previous_health: scalar("PreviousDrawHealth")?,
        projectile_population: scalar("ProjectilePopulation")?,
        jump_buttons: scalar("CurrentJumpBinding")?,
    })
}

fn check_requirements(proof: &Proof, requirements: &Requirements) -> Result<(), Fault> {
    for scalar in &requirements.scalar {
        if !scalar.actual_current_owner_required
            || !scalar.expected_encoding_is_not_runtime_initializer
        {
            return Err(Fault::Source);
        }
        matching(
            proof,
            &scalar.name,
            scalar.source_use_index,
            scalar.private_wram_offset,
            scalar.quantity_width,
        )?;
    }
    for collection in &requirements.collection {
        if !collection.initial_native_empty_is_not_runtime_proof {
            return Err(Fault::Source);
        }
        matching(
            proof,
            &collection.view,
            collection.source_use_index,
            collection.private_wram_offset,
            collection.quantity_width,
        )?;
    }
    Ok(())
}

fn drawing_collections(
    bytes: &[u8],
    proof: &Proof,
    requirements: &Requirements,
    scalar: impl Fn(&str) -> Result<u16, Fault>,
) -> Result<Collections, Fault> {
    let members = |view: &str, size: usize| -> Result<Vec<Membership>, Fault> {
        let mut result = Vec::with_capacity(size);
        for identity in 0..size {
            let parts = requirements
                .collection
                .iter()
                .filter(|part| part.view == view && part.identity == identity)
                .collect::<Vec<_>>();
            if parts.is_empty() {
                return Err(Fault::Source);
            }
            for part in parts {
                let premise = matching(
                    proof,
                    view,
                    part.source_use_index,
                    part.private_wram_offset,
                    part.quantity_width,
                )?;
                if initial(bytes, premise)? != 0 {
                    return Err(Fault::Unsupported);
                }
            }
            result.push(Membership::Absent);
        }
        Ok(result)
    };
    let enemy = members("EnemyProjectileSlot", 18)?;
    // Four source traversals are aliases of the sole enemy-projectile cells.
    for view in [
        "CollisionEnemyProjectileSlot",
        "HighPriorityEnemyProjectileSlot",
        "LowPriorityEnemyProjectileSlot",
    ] {
        if members(view, 18)? != enemy {
            return Err(Fault::Origin);
        }
        for identity in 0..18 {
            let first = requirements
                .collection
                .iter()
                .find(|part| part.view == "EnemyProjectileSlot" && part.identity == identity)
                .ok_or(Fault::Source)?;
            if requirements
                .collection
                .iter()
                .filter(|part| part.view == view && part.identity == identity)
                .any(|part| part.private_wram_offset != first.private_wram_offset)
            {
                return Err(Fault::Source);
            }
        }
    }
    members("LeftProjectileTrailTimer", 18)?;
    members("RightProjectileTrailTimer", 18)?;
    members("EnemyDrawingLayerQueue", 8)?;
    Ok(Collections {
        animated_tiles_enabled: scalar("AnimatedTilesEnabled")? != 0,
        enemy_projectiles_enabled: scalar("EnemyProjectilesEnabled")? != 0,
        plm_enabled: scalar("PlmEnabled")? != 0,
        enemy_projectiles: enemy.try_into().map_err(|_| Fault::Source)?,
        plm: members("PlmSlot", 40)?
            .try_into()
            .map_err(|_| Fault::Source)?,
        animated_tiles: members("AnimatedTileSlot", 6)?
            .try_into()
            .map_err(|_| Fault::Source)?,
        atmospheric: members("AtmosphericGraphicSlot", 4)?
            .try_into()
            .map_err(|_| Fault::Source)?,
        left_trails: [Remaining(0); 18],
        right_trails: [Remaining(0); 18],
        actor_layers: core::array::from_fn(|_| Vec::new()),
    })
}

fn matching<'a>(
    proof: &'a Proof,
    name: &str,
    opportunity: usize,
    offset: usize,
    width: usize,
) -> Result<&'a Premise, Fault> {
    let mut matching = proof.current_premise.iter().filter(|premise| {
        premise.name == name
            && premise.source_use_index == opportunity
            && premise.private_wram_offset == offset
            && premise.quantity_width == width
    });
    let premise = matching.next().ok_or(Fault::Source)?;
    if matching.next().is_some() {
        return Err(Fault::Source);
    }
    Ok(premise)
}

fn initial(bytes: &[u8], premise: &Premise) -> Result<u16, Fault> {
    if !premise.origin_equal_at_use {
        return Err(Fault::Origin);
    }
    let raw = bytes
        .get(
            premise.private_wram_offset
                ..premise
                    .private_wram_offset
                    .checked_add(premise.quantity_width)
                    .ok_or(Fault::Origin)?,
        )
        .ok_or(Fault::Origin)?;
    let value = match raw {
        [byte] => u16::from(*byte),
        [low, high] => u16::from_le_bytes([*low, *high]),
        _ => return Err(Fault::Source),
    };
    if value != premise.current_source_encoding {
        return Err(Fault::Origin);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_or_replaced_origin_is_rejected_before_decoding() {
        assert!(matches!(origin(&[], "", ""), Err(Fault::Origin)));
        assert!(matches!(
            origin(&vec![0; 131_072], "", ""),
            Err(Fault::Origin)
        ));
    }

    #[test]
    #[ignore = "requires independently reviewed private drawing sources"]
    fn actual_source_packet_retains_concrete_initial_owners() {
        let read = |name| std::fs::read(std::env::var(name).unwrap()).unwrap();
        let bytes = read("SM_FRAME_CURRENT_ORIGIN");
        let proof = String::from_utf8(read("SM_FRAME_DRAWING_CURRENT_PROOF")).unwrap();
        let requirements = String::from_utf8(read("SM_FRAME_DRAWING_OWNER_REQUIREMENTS")).unwrap();
        let owners = origin(&bytes, &proof, &requirements).unwrap();
        assert!(
            owners
                .collections
                .enemy_projectiles
                .iter()
                .all(|member| *member == Membership::Absent)
        );
        let mut replaced = bytes;
        replaced[0] ^= 1;
        assert!(matches!(
            origin(&replaced, &proof, &requirements),
            Err(Fault::Origin)
        ));
    }
}
