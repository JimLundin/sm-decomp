//! Current HUD drawing receipts and physical profiles, separate from native entry.

use canonical::Canonical;
use data::timing::{DeviceWrite, Profile, Template, TemplateEffect};

/// Actual software HUD drawing request.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawingRequest {
    /// Present the HUD overlay.
    Begin,
    /// Restore the current gameplay video settings.
    End,
}

/// Current separately requested interrupt disposition.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NextRequest {
    /// The qualified handler has no separate next request.
    None,
    /// An additional request requires an unqualified handler arm.
    Pending,
}

/// Complete current gameplay video mirror settings used by the end handler.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct VideoSettings {
    /// Background-three map layout encoding.
    pub background_three_map: u8,
    /// Current colour-window encoding.
    pub colour_window: u8,
    /// Current colour-math encoding.
    pub colour_math: u8,
    /// Current main-layer visibility encoding.
    pub main_layers: u8,
}

/// Imported beam target coordinates, each limited to nine bits.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawingTarget {
    /// Horizontal beam coordinate.
    pub horizontal: u16,
    /// Vertical beam coordinate.
    pub vertical: u16,
}

/// Immutable HUD overlay settings and its two target coordinates.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawingContent {
    /// HUD overlay settings imported independently of device callbacks.
    pub hud: VideoSettings,
    /// Target installed after the begin handler.
    pub after_begin: DrawingTarget,
    /// Target installed by the empty-request end handler.
    pub after_end: DrawingTarget,
}

/// Current owner facts required at a HUD drawing opportunity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawingInputs {
    /// Actual current software drawing request.
    pub request: DrawingRequest,
    /// Current separately requested interrupt.
    pub next_request: NextRequest,
    /// Complete current gameplay video settings.
    pub video: VideoSettings,
}

/// Independently admitted complete current dependency, retained until handler use.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawingReceipt {
    /// Exact current facts for which the handler arm is qualified.
    pub dependency: DrawingInputs,
}

/// Retained current drawing owner and once-only handler admissions.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct DrawingState {
    /// Actual current request and unchanged video settings.
    pub current: DrawingInputs,
    /// Independently admitted Begin dependency, consumed when entered.
    pub begin: Option<DrawingReceipt>,
    /// Independently admitted End dependency, consumed when entered.
    pub end: Option<DrawingReceipt>,
    /// Handler currently suspended or charging through native return.
    pub active: Option<DrawingRequest>,
    /// Independently borrowed final Begin schedule writer.
    pub after_begin: Option<DrawingRequest>,
    /// Independently borrowed final End schedule writer; unavailable until admitted.
    pub after_end: Option<DrawingRequest>,
    /// Next qualified recurring handler, empty after both have returned.
    pub recurring_due: Option<DrawingRequest>,
}

impl DrawingState {
    pub(super) fn begin_opportunity(&mut self) {
        self.begin = None;
        self.end = None;
        self.after_begin = None;
        self.after_end = None;
        self.recurring_due = (self.active.is_none()
            && self.current.request == DrawingRequest::Begin)
            .then_some(DrawingRequest::Begin);
    }

    pub(super) fn bind_recurring(
        &self,
        profiles: &DrawingProfiles,
    ) -> Result<Profile, DrawingFault> {
        if self.recurring_due != Some(self.current.request) {
            return Err(DrawingFault::Stale);
        }
        profiles.bind_current(self.current)
    }

    pub(super) fn complete_recurring(
        &mut self,
        request: DrawingRequest,
    ) -> Result<(), DrawingFault> {
        if self.recurring_due != Some(request) || self.current.request != request {
            return Err(DrawingFault::Stale);
        }
        match request {
            DrawingRequest::Begin => {
                self.current.request = DrawingRequest::End;
                self.recurring_due = Some(DrawingRequest::End);
            }
            DrawingRequest::End => {
                self.current.request = DrawingRequest::Begin;
                self.recurring_due = None;
            }
        }
        Ok(())
    }
}

/// Qualified byte-free drawing profiles with their current immutable content.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct DrawingProfiles {
    /// Complete begin-handler physical profile.
    pub begin: Template,
    /// Complete empty-request end-handler physical profile.
    pub end: Template,
    /// Independently checked imported settings and target coordinates.
    pub content: DrawingContent,
}

/// Invalid or stale drawing dependency or physical projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawingFault {
    /// A required current receipt is missing.
    Missing,
    /// The complete actual dependency differs from the admitted receipt.
    Stale,
    /// A separately requested interrupt arm is not qualified.
    NextRequest,
    /// Imported target coordinates exceed their physical representation.
    Target,
    /// The complete physical template does not match the owner publications.
    Profile,
}

impl DrawingProfiles {
    /// Resolve all handler writes from current owners before exposing any effects.
    /// Native entry, sampling, charging and request completion remain caller work.
    /// # Errors
    /// Rejects missing/stale receipts, other next-request arms and invalid profiles.
    pub fn bind(
        &self,
        current: DrawingInputs,
        receipt: Option<&DrawingReceipt>,
    ) -> Result<Profile, DrawingFault> {
        let receipt = receipt.ok_or(DrawingFault::Missing)?;
        if receipt.dependency != current {
            return Err(DrawingFault::Stale);
        }
        self.bind_current(current)
    }

    fn bind_current(&self, current: DrawingInputs) -> Result<Profile, DrawingFault> {
        if current.next_request != NextRequest::None {
            return Err(DrawingFault::NextRequest);
        }
        let (template, settings, target) = match current.request {
            DrawingRequest::Begin => (&self.begin, self.content.hud, self.content.after_begin),
            DrawingRequest::End => (&self.end, current.video, self.content.after_end),
        };
        if target.horizontal > 511 || target.vertical > 511 {
            return Err(DrawingFault::Target);
        }
        let [horizontal_low, horizontal_high] = target.horizontal.to_le_bytes();
        let [vertical_low, vertical_high] = target.vertical.to_le_bytes();
        let settings = match current.request {
            DrawingRequest::Begin => [
                settings.background_three_map,
                settings.colour_window,
                settings.colour_math,
                settings.main_layers,
            ],
            DrawingRequest::End => [
                settings.colour_window,
                settings.colour_math,
                settings.background_three_map,
                settings.main_layers,
            ],
        };
        let values = settings.into_iter().chain([
            vertical_low,
            vertical_high,
            horizontal_low,
            horizontal_high,
        ]);
        let ports = template.runs.iter().flat_map(|run| {
            let port = match run.effect {
                TemplateEffect::Publish(port) => Some(port),
                TemplateEffect::None | TemplateEffect::Sample(_) => None,
            };
            (0..run.count).filter_map(move |_| port)
        });
        let mut values = values;
        let writes = ports
            .map(|port| {
                let value = values.next().ok_or(DrawingFault::Profile)?;
                DeviceWrite::from_cpu_port(port.0, value).map_err(|_| DrawingFault::Profile)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if values.next().is_some() {
            return Err(DrawingFault::Profile);
        }
        template.bind(&writes).map_err(|_| DrawingFault::Profile)
    }
}
