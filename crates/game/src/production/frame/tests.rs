use super::*;
use data::dma::BusAddress;

fn region(offset: u16, bytes: Vec<u8>) -> ContentRegion {
    ContentRegion {
        start: BusAddress { bank: 1, offset },
        bytes,
    }
}

#[test]
fn ready_buffers_survive_restore_without_repeating_completion() {
    let mut content = TransferContent::new(vec![region(0, vec![2, 3])]).unwrap();
    let made = Materialization {
        buffer: Buffer::HudTilemap,
        region: region(2, vec![4, 5]),
    };
    content.complete(made.clone()).unwrap();
    let bytes = canonical::to_bytes(&content);
    drop(content);
    let mut restored: TransferContent = canonical::from_bytes(&bytes).unwrap();
    assert_eq!(restored.validate(), Ok(()));
    let before = restored.clone();
    assert_eq!(
        restored.complete(made),
        Err(ContentFault::AlreadyCompleted(Buffer::HudTilemap))
    );
    assert_eq!(restored, before);
    assert_eq!(
        restored.regions(),
        vec![region(0, vec![2, 3]), region(2, vec![4, 5])]
    );
}

#[test]
fn overlapping_producer_rejects_without_publishing_or_consuming_ready() {
    let mut content = TransferContent::new(vec![region(0, vec![2, 3])]).unwrap();
    let before = content.clone();
    assert_eq!(
        content.complete(Materialization {
            buffer: Buffer::SpriteDrawList,
            region: region(1, vec![4, 5])
        }),
        Err(ContentFault::Overlap)
    );
    assert_eq!(content, before);
    assert_eq!(
        content.complete(Materialization {
            buffer: Buffer::SpriteDrawList,
            region: region(2, vec![4, 5])
        }),
        Ok(())
    );
}

#[test]
fn physical_publications_restore_complete_stream_and_exactly_once_delivery() {
    use crate::production::timing::{Event, EventKind};
    use data::timing::MasterTicks;
    let first = Event {
        at: MasterTicks(3),
        kind: EventKind::ControllerStart,
    };
    let second = Event {
        at: MasterTicks(3),
        kind: EventKind::ControllerEnd,
    };
    let mut publications = Publications::default();
    publications.record(vec![first.clone()]).unwrap();
    assert_eq!(publications.deliver().unwrap(), vec![first.clone()]);
    publications.record(vec![second.clone()]).unwrap();
    let bytes = canonical::to_bytes(&publications);
    drop(publications);
    let mut restored: Publications = canonical::from_bytes(&bytes).unwrap();
    assert_eq!(restored.deliver().unwrap(), vec![second.clone()]);
    assert_eq!(restored.deliver().unwrap().len(), 0);
    assert_eq!(restored.events, vec![first, second]);
    let before = restored.clone();
    assert_eq!(
        restored.record(vec![Event {
            at: MasterTicks(2),
            kind: EventKind::ControllerCommitted
        }]),
        Err(PublicationFault::Order)
    );
    assert_eq!(restored, before);
}
