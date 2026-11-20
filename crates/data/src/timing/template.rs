//! Byte-free physical templates resolved from actual current publications.

use super::{
    Cycle, DeviceEffect, DeviceWrite, Exposure, ExposureKind, Profile, PublicationPort, Run, Sample,
};
use canonical::Canonical;

/// Invalid physical template or incompatible current producer publications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateFault {
    /// Empty, zero-count or incompatible physical opportunity shape.
    InvalidTemplate,
    /// Missing/extra publication or incompatible port/byte/configuration.
    Publication,
}
/// Physical effect without a publication value or semantic dispatcher.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateEffect {
    /// No device effect.
    None,
    /// Sample the actual current hardware result.
    Sample(Sample),
    /// Current producer must supply a validated write to this port.
    Publish(PublicationPort),
}
/// Repeated equivalent physical opportunities with unresolved publications.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemplateRun {
    /// Nonzero number of equivalent opportunities.
    pub count: u32,
    /// Physical bus cycle class.
    pub cycle: Cycle,
    /// Charging and arithmetic edge ordering.
    pub kind: ExposureKind,
    /// Sample eligible interrupt transitions before this opportunity.
    pub poll_before: bool,
    /// Physical result or unresolved current publication.
    pub effect: TemplateEffect,
}
/// Canonical byte-free physical projection retained by its current semantic owner.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Template {
    /// Nonempty finite equivalent opportunities in physical order.
    pub runs: Vec<TemplateRun>,
}
impl TemplateRun {
    fn validate(&self) -> Result<(), TemplateFault> {
        if self.count == 0
            || (matches!(
                self.kind,
                ExposureKind::Internal | ExposureKind::PendingInternal
            ) && self.cycle != Cycle::Six)
        {
            return Err(TemplateFault::InvalidTemplate);
        }
        match self.effect {
            TemplateEffect::None => Ok(()),
            TemplateEffect::Sample(_) if self.kind == ExposureKind::Read => Ok(()),
            TemplateEffect::Publish(port) if self.kind == ExposureKind::Write => port
                .output_class()
                .map(|_| ())
                .map_err(|_| TemplateFault::InvalidTemplate),
            TemplateEffect::Sample(_) | TemplateEffect::Publish(_) => {
                Err(TemplateFault::InvalidTemplate)
            }
        }
    }
    const fn resolved(&self, effect: DeviceEffect) -> Run {
        Run {
            count: self.count,
            exposure: Exposure {
                cycle: self.cycle,
                kind: self.kind,
                poll_before: self.poll_before,
                effect,
            },
        }
    }
}
impl Template {
    /// Validate the complete physical shape after decoding a saved template.
    /// # Errors
    /// Rejects empty, zero-count or incompatible physical opportunities.
    pub fn validate(&self) -> Result<(), TemplateFault> {
        if self.runs.is_empty() {
            return Err(TemplateFault::InvalidTemplate);
        }
        self.runs.iter().try_for_each(TemplateRun::validate)
    }
    /// Bind a stable template from already produced and admitted current owner writes.
    ///
    /// Supply exactly one write per publication occurrence in physical order.
    /// This conversion executes no owner transition, callback, scheduling or charge.
    /// Rejection returns no partial work and preserves the template and inputs.
    /// # Errors
    /// Rejects invalid templates, missing/extra writes and port/configuration mismatches.
    pub fn bind(&self, publications: &[DeviceWrite]) -> Result<Profile, TemplateFault> {
        self.validate()?;
        let mut writes = publications.iter();
        let mut runs = Vec::new();
        for run in &self.runs {
            match run.effect {
                TemplateEffect::None => runs.push(run.resolved(DeviceEffect::None)),
                TemplateEffect::Sample(device) => {
                    runs.push(run.resolved(DeviceEffect::Sample(device)));
                }
                TemplateEffect::Publish(port) => {
                    for _ in 0..run.count {
                        let write = writes.next().ok_or(TemplateFault::Publication)?;
                        if write.port != port {
                            return Err(TemplateFault::Publication);
                        }
                        write.validate().map_err(|_| TemplateFault::Publication)?;
                        let mut resolved = run.resolved(DeviceEffect::Publish(write.clone()));
                        resolved.count = 1;
                        runs.push(resolved);
                    }
                }
            }
        }
        if writes.next().is_some() {
            return Err(TemplateFault::Publication);
        }
        Ok(Profile { runs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_internal_preserves_independent_poll_choice() {
        let template = Template {
            runs: vec![TemplateRun {
                count: 1,
                cycle: Cycle::Six,
                kind: ExposureKind::PendingInternal,
                poll_before: false,
                effect: TemplateEffect::None,
            }],
        };
        let bound = template.bind(&[]).unwrap();
        assert_eq!(
            bound.runs.first().unwrap().exposure.kind,
            ExposureKind::PendingInternal
        );
        assert!(!bound.runs.first().unwrap().exposure.poll_before);
    }
    #[test]
    fn saved_template_binds_current_writes_and_rejects_invalid_restored_shape() {
        let template = Template {
            runs: vec![TemplateRun {
                count: 2,
                cycle: Cycle::Six,
                kind: ExposureKind::Write,
                poll_before: false,
                effect: TemplateEffect::Publish(PublicationPort(0x4202)),
            }],
        };
        let restored: Template = canonical::from_bytes(&canonical::to_bytes(&template)).unwrap();
        let writes = [
            DeviceWrite::from_cpu_port(0x4202, 3).unwrap(),
            DeviceWrite::from_cpu_port(0x4202, 5).unwrap(),
        ];
        assert_eq!(restored.bind(&writes), template.bind(&writes));
        let bound = restored.bind(&writes).unwrap();
        assert_eq!(
            bound.runs.first().unwrap().exposure.effect,
            DeviceEffect::Publish(writes[0].clone())
        );
        assert_eq!(
            bound.runs.last().unwrap().exposure.effect,
            DeviceEffect::Publish(writes[1].clone())
        );
        let invalid = Template {
            runs: vec![TemplateRun {
                count: 0,
                ..template.runs[0]
            }],
        };
        let invalid_restored: Template =
            canonical::from_bytes(&canonical::to_bytes(&invalid)).unwrap();
        assert!(
            invalid_restored
                .bind(&writes)
                .is_err_and(|fault| fault == TemplateFault::InvalidTemplate)
        );
    }
}
