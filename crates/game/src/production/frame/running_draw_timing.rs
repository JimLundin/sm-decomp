//! Qualified physical appends for both maps of the live running picture.

use super::rendering_stage::{Content, NormalDrawingReceipt};
use crate::production::{ProductionFault, Unsupported};
use data::{
    spritemap::{SpriteSize, Spritemap},
    timing::{Profile, Template, TemplateEffect},
};

/// Settled work around the two actual picture maps for one running figure family.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawingPortions {
    setup: Profile,
    between: Profile,
    finish: Profile,
}

impl DrawingPortions {
    /// Admit the quiet work before, between and after the two map appends.
    ///
    /// # Errors
    /// Work contains unresolved effects or has an invalid physical profile.
    pub fn new(
        setup: &Template,
        between: &Template,
        finish: &Template,
    ) -> Result<Self, ProductionFault> {
        Ok(Self {
            setup: bind(setup)?,
            between: bind(between)?,
            finish: bind(finish)?,
        })
    }
}

/// Physical append branches shared by forward and down-forward running pictures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PieceProfiles {
    small: Profile,
    small_last: Profile,
    large: Profile,
    large_last: Profile,
}

impl PieceProfiles {
    /// Bind the four low-X piece appends, including each map's terminal append.
    ///
    /// # Errors
    /// Work contains unresolved effects or has an invalid physical profile.
    pub fn new(
        small: &Template,
        small_last: &Template,
        large: &Template,
        large_last: &Template,
    ) -> Result<Self, ProductionFault> {
        Ok(Self {
            small: bind(small)?,
            small_last: bind(small_last)?,
            large: bind(large)?,
            large_last: bind(large_last)?,
        })
    }

    fn append(
        &self,
        profile: &mut Profile,
        map: &Spritemap,
        origin_x: i32,
    ) -> Result<(), ProductionFault> {
        let (last, rest) = map.entries.split_last().ok_or_else(missing)?;
        for (entry, last) in rest
            .iter()
            .map(|entry| (entry, false))
            .chain(std::iter::once((last, true)))
        {
            u8::try_from(origin_x.wrapping_add(i32::from(entry.x))).map_err(|_| missing())?;
            let portion = match (entry.size, last) {
                (SpriteSize::Small, false) => &self.small,
                (SpriteSize::Small, true) => &self.small_last,
                (SpriteSize::Large, false) => &self.large,
                (SpriteSize::Large, true) => &self.large_last,
            };
            profile.runs.extend(portion.runs.iter().cloned());
        }
        Ok(())
    }
}

/// Whole running drawing work assembled from the actual two checked sprite maps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecurringProfiles {
    /// Work around the map loops for the live forward figure.
    pub forward: DrawingPortions,
    /// Work around the map loops for the live down-forward figure.
    pub down_forward: DrawingPortions,
    /// Physical loop paths for individual visible pieces in each map.
    pub pieces: PieceProfiles,
}

impl RecurringProfiles {
    pub(super) fn profile(
        &self,
        content: &Content,
        receipt: &NormalDrawingReceipt,
        mixed: Option<&Profile>,
    ) -> Result<Profile, ProductionFault> {
        use types::{Action, Aim, Body, Facing, Gun, Sprite};
        let figure = receipt.figure.0;
        let pose = figure.pose;
        if !receipt.premises.normal_visible
            || !receipt.premises.no_additional_selected_pieces
            || !receipt.premises.unselected_draws_empty
            || pose.action != Action::Running
            || pose.body != Body::Upright
            || pose.facing != Facing::Right
            || pose.gun != Gun::Stowed
            || figure.sprite != Sprite::Plain
        {
            return Err(missing());
        }
        let portions = if pose.aim == Aim::Forward {
            &self.forward
        } else if pose.aim == Aim::DownForward {
            &self.down_forward
        } else {
            return Err(missing());
        };
        let [top, bottom] = content.running_maps(receipt).map_err(|_| missing())?;
        if top.entries.len().checked_add(bottom.entries.len()) != Some(usize::from(receipt.pieces))
        {
            return Err(missing());
        }
        let [x, _] = receipt.centre;
        let origin_x = receipt.premises.horizontal_origin(x);
        let mut profile = match receipt.first_byte {
            0 => portions.setup.clone(),
            24 => splice_actor_layer(&portions.setup, mixed.ok_or_else(missing)?)?,
            _ => return Err(missing()),
        };
        self.pieces.append(&mut profile, top, origin_x)?;
        profile.runs.extend(portions.between.runs.iter().cloned());
        self.pieces.append(&mut profile, bottom, origin_x)?;
        profile.runs.extend(portions.finish.runs.iter().cloned());
        Ok(profile)
    }
}

fn bind(template: &Template) -> Result<Profile, ProductionFault> {
    if template
        .runs
        .iter()
        .any(|run| run.effect != TemplateEffect::None)
    {
        return Err(missing());
    }
    template.bind(&[]).map_err(|_| missing())
}

const fn missing() -> ProductionFault {
    use super::Dependency;
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::DrawingPreparation))
}

fn splice_actor_layer(setup: &Profile, actor: &Profile) -> Result<Profile, ProductionFault> {
    use data::timing::Run;
    const START: u32 = 1340;
    const END: u32 = 1354;

    let mut prefix = Vec::new();
    let mut suffix = Vec::new();
    let mut start: u32 = 0;
    for run in &setup.runs {
        let end = start.checked_add(run.count).ok_or_else(missing)?;
        let before = end.min(START).saturating_sub(start);
        let after = end.saturating_sub(start.max(END));
        if before != 0 {
            prefix.push(Run {
                count: before,
                exposure: run.exposure.clone(),
            });
        }
        if after != 0 {
            suffix.push(Run {
                count: after,
                exposure: run.exposure.clone(),
            });
        }
        start = end;
    }
    if prefix.is_empty() || suffix.is_empty() {
        return Err(missing());
    }
    prefix.extend(actor.runs.iter().cloned());
    prefix.extend(suffix);
    Ok(Profile { runs: prefix })
}
