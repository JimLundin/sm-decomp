//! Independent callback decoding and exact projection of actual physical events.

use data::timing::MasterTicks;

/// One callback in the independently declared physical output namespace.
#[derive(canonical::Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Datum {
    pub at: MasterTicks,
    pub class: u8,
    pub register: u32,
    pub value: u8,
}

/// An observation cannot be represented in the declared callback namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionFault {
    Format,
    Class,
    HardwareIdentity,
    Port,
}

/// Decode the independently captured completed endpoint, without prescribing runtime work.
pub fn decode_endpoint(text: &str) -> Result<MasterTicks, ProjectionFault> {
    let table = text
        .parse::<toml::Table>()
        .map_err(|_| ProjectionFault::Format)?;
    let ticks = table
        .get("CPU_ticks")
        .and_then(toml::Value::as_integer)
        .ok_or(ProjectionFault::Format)?;
    Ok(MasterTicks(
        u64::try_from(ticks).map_err(|_| ProjectionFault::Format)?,
    ))
}

/// Decode frozen expected callbacks; source-order coordinates are navigation metadata.
/// File order, actual timestamps and complete callback identities remain unchanged.
pub fn decode_callbacks(bytes: &[u8]) -> Result<Vec<Datum>, ProjectionFault> {
    let (chunks, remainder) = bytes.as_chunks::<18>();
    if bytes.is_empty() || !remainder.is_empty() {
        return Err(ProjectionFault::Format);
    }
    chunks
        .iter()
        .map(|record| {
            let &[
                _,
                _,
                _,
                _,
                t0,
                t1,
                t2,
                t3,
                t4,
                t5,
                t6,
                t7,
                class,
                r0,
                r1,
                r2,
                r3,
                value,
            ] = record;
            if !(1..=12).contains(&class) {
                return Err(ProjectionFault::Class);
            }
            let register = u32::from_le_bytes([r0, r1, r2, r3]);
            if class >= 7 && (register != 0 || value != 0) {
                return Err(ProjectionFault::HardwareIdentity);
            }
            Ok(Datum {
                at: MasterTicks(u64::from_le_bytes([t0, t1, t2, t3, t4, t5, t6, t7])),
                class,
                register,
                value,
            })
        })
        .collect()
}

/// Project only callbacks declared by the independent physical output namespace.
/// Device samples and input commits remain retained internal events.
pub fn project(events: &[game::production::timing::Event]) -> Result<Vec<Datum>, ProjectionFault> {
    use game::production::timing::EventKind;
    let mut callbacks = Vec::new();
    for event in events {
        let (class, register, value) = match &event.kind {
            EventKind::Publication(write) => (
                write
                    .port
                    .output_class()
                    .map_err(|_| ProjectionFault::Port)?,
                u32::from(write.port.0),
                write.value,
            ),
            EventKind::Payload(payload) => {
                let port = payload.target.publication_port();
                (
                    port.output_class().map_err(|_| ProjectionFault::Port)?,
                    u32::from(port.0),
                    payload.value,
                )
            }
            EventKind::ControllerStart => (9, 0, 0),
            EventKind::ControllerEnd => (10, 0, 0),
            EventKind::LineService(_) => (11, 0, 0),
            EventKind::BulkService(_) => (12, 0, 0),
            EventKind::Interrupt(data::timing::Interrupt::Nmi) => (7, 0, 0),
            EventKind::Interrupt(data::timing::Interrupt::Irq) => (8, 0, 0),
            EventKind::Sample(_, _) | EventKind::ControllerCommitted => {
                continue;
            }
        };
        callbacks.push(Datum {
            at: event.at,
            class,
            register,
            value,
        });
    }
    Ok(callbacks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use data::timing::{ChannelSlot, Interrupt, Sample};
    use game::production::timing::{Event, EventKind, SampleValue};

    #[test]
    fn declared_callbacks_keep_order_and_interrupt_dispatch_without_internal_samples() {
        let events = vec![
            Event {
                at: MasterTicks(9),
                kind: EventKind::ControllerStart,
            },
            Event {
                at: MasterTicks(9),
                kind: EventKind::Sample(Sample::ControllerLow, SampleValue::Byte(0)),
            },
            Event {
                at: MasterTicks(9),
                kind: EventKind::BulkService(ChannelSlot::Two),
            },
            Event {
                at: MasterTicks(10),
                kind: EventKind::ControllerEnd,
            },
            Event {
                at: MasterTicks(11),
                kind: EventKind::ControllerCommitted,
            },
            Event {
                at: MasterTicks(12),
                kind: EventKind::Interrupt(Interrupt::Nmi),
            },
        ];
        let callbacks = project(&events).unwrap();
        assert_eq!(
            callbacks
                .iter()
                .map(|datum| datum.class)
                .collect::<Vec<_>>(),
            [9, 12, 10, 7]
        );
        assert_eq!(
            callbacks.iter().map(|datum| datum.at.0).collect::<Vec<_>>(),
            [9, 9, 10, 12]
        );
        assert!(
            callbacks
                .iter()
                .all(|datum| datum.register == 0 && datum.value == 0)
        );
    }

    #[test]
    #[ignore = "requires independent frozen ordered callback artifact"]
    fn frozen_callbacks_decode_and_restore_without_discarding_callbacks() {
        let path = std::env::var_os("SM_FRAME_OUTPUT_CALLBACKS").expect("frozen callback artifact");
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(
            crate::lock::Sha256::of(&bytes).to_string(),
            "c6f15c1b068065651a0791b13815078fc501f71e69ce575f4f65e7906a8c082b"
        );
        let callbacks = decode_callbacks(&bytes).unwrap();
        assert_eq!(callbacks.len(), 2295);
        let restored: Vec<Datum> = canonical::from_bytes(&canonical::to_bytes(&callbacks)).unwrap();
        assert!(restored.iter().eq(&callbacks));
        assert!(decode_callbacks(bytes.get(..bytes.len().saturating_sub(1)).unwrap()).is_err());
        let mut navigation = bytes;
        navigation
            .get_mut(..4)
            .unwrap()
            .copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_callbacks(&navigation).unwrap().iter().eq(&callbacks));
    }

    #[test]
    #[ignore = "requires pinned completed endpoint capture receipt"]
    fn completed_endpoint_uses_captured_cpu_ticks_and_rejects_invalid_units() {
        let path =
            std::env::var_os("SM_FRAME_ENDPOINT_RECEIPT").expect("captured endpoint receipt");
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(
            crate::lock::Sha256::of(&bytes).to_string(),
            "4d98979fde42a60d851743bfc6e1b7b8f1eb6cfe75a7d5048bd7a268a6d36f4b"
        );
        let text = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(decode_endpoint(text), Ok(MasterTicks(359_922)));
        for invalid in [
            "CPU_ticks = -1",
            "CPU_ticks = '359922'",
            "audio_cycles = 17178",
        ] {
            assert_eq!(decode_endpoint(invalid), Err(ProjectionFault::Format));
        }
    }
}
