//! Independently qualified current owner receipt codec evidence.
#[path = "../src/replay/frame_borrowed_receipts.rs"]
mod frame_borrowed_receipts;
use frame_borrowed_receipts::BorrowedReceiptFault;
use game::production::frame::borrowed_receipts::{
    FrameContinuationAfterNmi, MusicCompletionReceipt, MusicCountdown, MusicQueue, MusicRequest,
    MusicServiceReceipt, SoftwareRequest,
};

#[test]
fn incomplete_publication_origin_is_rejected() {
    for length in [0, 1461, 131_071, 131_073] {
        assert!(
            frame_borrowed_receipts::publication(&vec![0; length])
                .is_err_and(|fault| fault == BorrowedReceiptFault::Length)
        );
        assert!(
            frame_borrowed_receipts::music(&vec![0; length])
                .is_err_and(|fault| fault == BorrowedReceiptFault::Length)
        );
        assert!(
            frame_borrowed_receipts::qualified_music_completion(&vec![0; length], "")
                .is_err_and(|fault| fault == BorrowedReceiptFault::Length)
        );
    }
}

#[test]
#[ignore = "requires independently qualified origin and external Music opportunity"]
fn qualified_external_music_completion_retains_actual_dependency() {
    let bytes = std::fs::read(std::env::var_os("SM_FRAME_ORIGIN_WRAM").unwrap()).unwrap();
    let projection =
        std::fs::read_to_string(std::env::var_os("SM_MUSIC_COMPLETION").unwrap()).unwrap();
    let completion =
        frame_borrowed_receipts::qualified_music_completion(&bytes, &projection).unwrap();
    assert!(
        completion
            .dependency
            .eq(&frame_borrowed_receipts::music(&bytes).unwrap())
    );
    assert_eq!(completion.terminal.countdown, MusicCountdown::Ready);
    assert_eq!(completion.terminal.queue, MusicQueue::Empty);
    assert_eq!(completion.request, MusicRequest::None);
    let restored: MusicCompletionReceipt =
        canonical::from_bytes(&canonical::to_bytes(&completion)).unwrap();
    assert!(restored.eq(&completion));
    for at in [1599, 1600, 1595, 1596, 1593, 1594] {
        let mut changed = bytes.clone();
        changed[at] ^= 1;
        assert!(
            frame_borrowed_receipts::qualified_music_completion(&changed, &projection)
                .is_err_and(|fault| fault == BorrowedReceiptFault::UnsupportedMusic)
        );
    }
    assert!(
        frame_borrowed_receipts::qualified_music_completion(&bytes, "")
            .is_err_and(|fault| fault == BorrowedReceiptFault::Format)
    );
    for (from, to) in [
        ("output_request = \"None\"", "output_request = \"Cue\""),
        (
            "physical_reply_observations = 0",
            "physical_reply_observations = 1",
        ),
        (
            "consume_once_at_terminal_completion = true",
            "consume_once_at_terminal_completion = false",
        ),
        (
            "expected_callback_authority = false",
            "expected_callback_authority = true",
        ),
    ] {
        let changed = projection.replace(from, to);
        assert!(changed.ne(&projection));
        assert!(
            frame_borrowed_receipts::qualified_music_completion(&bytes, &changed)
                .is_err_and(|fault| fault == BorrowedReceiptFault::UnsupportedCompletion)
        );
    }
}

#[test]
#[ignore = "requires independently qualified complete origin WRAM"]
fn actual_music_disposition_requires_ready_countdown_and_empty_queue() {
    let bytes = std::fs::read(std::env::var_os("SM_FRAME_ORIGIN_WRAM").unwrap()).unwrap();
    let receipt = frame_borrowed_receipts::music(&bytes).unwrap();
    assert_eq!(receipt.countdown, MusicCountdown::Ready);
    assert_eq!(receipt.queue, MusicQueue::Empty);
    let restored: MusicServiceReceipt =
        canonical::from_bytes(&canonical::to_bytes(&receipt)).unwrap();
    assert!(restored.eq(&receipt));
    for at in [1599, 1600, 1595, 1596, 1593, 1594] {
        let mut changed = bytes.clone();
        changed[at] ^= 1;
        assert!(
            frame_borrowed_receipts::music(&changed)
                .is_err_and(|fault| fault == BorrowedReceiptFault::UnsupportedMusic)
        );
    }
    let mut moved_empty_queue = bytes;
    moved_empty_queue[1593] ^= 1;
    moved_empty_queue[1595] ^= 1;
    assert!(
        frame_borrowed_receipts::music(&moved_empty_queue)
            .unwrap()
            .eq(&receipt)
    );
}

#[test]
#[ignore = "requires independently qualified complete origin WRAM"]
fn actual_publication_acknowledgment_comes_from_software_owner() {
    let bytes = std::fs::read(std::env::var_os("SM_FRAME_ORIGIN_WRAM").unwrap()).unwrap();
    let receipt = frame_borrowed_receipts::publication(&bytes).unwrap();
    assert_eq!(
        receipt.publication.prior_request,
        SoftwareRequest::Acknowledged
    );
    let restored: FrameContinuationAfterNmi =
        canonical::from_bytes(&canonical::to_bytes(&receipt)).unwrap();
    assert!(restored.eq(&receipt));
    let mut high_byte = bytes.clone();
    high_byte[1461] ^= 1;
    assert!(
        frame_borrowed_receipts::publication(&high_byte)
            .unwrap()
            .eq(&receipt)
    );
    for request in [1, 128, 255] {
        let mut requested = bytes.clone();
        requested[1460] = request;
        assert!(
            frame_borrowed_receipts::publication(&requested)
                .is_err_and(|fault| fault == BorrowedReceiptFault::UnsupportedRequest)
        );
    }
}
