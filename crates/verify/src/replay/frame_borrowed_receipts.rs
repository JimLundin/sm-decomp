//! One-way import of independently qualified current publication and sound owner facts.
use game::production::frame::borrowed_receipts::{
    FrameContinuationAfterNmi, MusicCompletionReceipt, MusicCountdown, MusicQueue, MusicRequest,
    MusicServiceReceipt, PublicationOrigin, SoftwareRequest,
};
use serde::Deserialize;

/// Invalid current owner memory or unsupported semantic disposition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorrowedReceiptFault {
    /// Malformed externally qualified receipt projection.
    Format,
    /// The origin must contain complete physical WRAM.
    Length,
    /// The current software request is outside the qualified acknowledgment arm.
    UnsupportedRequest,
    /// The current music service inputs are outside the qualified Ready/Empty arm.
    UnsupportedMusic,
    /// Unsupported terminal projection or external receipt consumption contract.
    UnsupportedCompletion,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct ContractFlag(bool);

#[derive(Deserialize)]
struct ExternalMusicCompletion {
    format: String,
    owner: String,
    required_countdown: String,
    required_queue: String,
    output_countdown: String,
    output_queue: String,
    output_request: String,
    physical_reply_observations: u32,
    physical_port_presentations: u32,
    read_until_actual_operation_completes: ContractFlag,
    require_actual_current_dependencies_revalidation: ContractFlag,
    consume_once_at_terminal_completion: ContractFlag,
    completion_identity_from_fixture: ContractFlag,
    expected_callback_authority: ContractFlag,
    adapter_implements_music_rule: ContractFlag,
}

fn word(bytes: &[u8], at: usize) -> Result<u16, BorrowedReceiptFault> {
    Ok(u16::from_le_bytes([
        bytes.get(at).copied().ok_or(BorrowedReceiptFault::Length)?,
        bytes
            .get(at.wrapping_add(1))
            .copied()
            .ok_or(BorrowedReceiptFault::Length)?,
    ]))
}

/// Import actual current sound owner dispositions from the qualified origin.
///
/// Source admission separately proves these inputs unchanged through their service reads.
/// This receipt performs no countdown transition, queue operation or physical reply read.
/// # Errors
/// Rejects incomplete memory and unqualified countdown or queue dispositions.
pub fn music(bytes: &[u8]) -> Result<MusicServiceReceipt, BorrowedReceiptFault> {
    if bytes.len() != 131_072 {
        return Err(BorrowedReceiptFault::Length);
    }
    if word(bytes, 1599)? != 0 || word(bytes, 1595)? != word(bytes, 1593)? {
        return Err(BorrowedReceiptFault::UnsupportedMusic);
    }
    Ok(MusicServiceReceipt {
        countdown: MusicCountdown::Ready,
        queue: MusicQueue::Empty,
    })
}

/// Import an independently qualified, externally borrowed Music completion.
///
/// The caller must admit the source opportunity's terminal Ready/Empty/no-request
/// projection independently of this origin. Origin integrity and no prior CPU writer
/// prove the complete current dependency equal to the owner's actual reads.
/// This imports observed unselected owner facts; it executes no countdown or queue rule.
/// Retain this receipt unread until the actual Music operation's full physical charge
/// completes, then revalidate its current dependency and consume it exactly once.
/// # Errors
/// Rejects incomplete origins, unqualified dependencies, malformed projections and
/// unsupported terminal facts or consumption contracts.
pub fn qualified_music_completion(
    current_origin: &[u8],
    external_projection: &str,
) -> Result<MusicCompletionReceipt, BorrowedReceiptFault> {
    let dependency = music(current_origin)?;
    let projection: ExternalMusicCompletion =
        toml::from_str(external_projection).map_err(|_| BorrowedReceiptFault::Format)?;
    if projection.format != "borrowed-music-completion-receipt-v2"
        || projection.owner != "Music"
        || projection.required_countdown != "Ready"
        || projection.required_queue != "Empty"
        || projection.output_countdown != "Ready"
        || projection.output_queue != "Empty"
        || projection.output_request != "None"
        || projection.physical_reply_observations != 0
        || projection.physical_port_presentations != 0
        || projection.read_until_actual_operation_completes.0
        || !projection
            .require_actual_current_dependencies_revalidation
            .0
        || !projection.consume_once_at_terminal_completion.0
        || projection.completion_identity_from_fixture.0
        || projection.expected_callback_authority.0
        || projection.adapter_implements_music_rule.0
    {
        return Err(BorrowedReceiptFault::UnsupportedCompletion);
    }
    Ok(MusicCompletionReceipt {
        dependency,
        terminal: MusicServiceReceipt {
            countdown: MusicCountdown::Ready,
            queue: MusicQueue::Empty,
        },
        request: MusicRequest::None,
    })
}

/// Import the actual software publication acknowledgment from current origin memory.
///
/// Admission separately proves origin integrity and invariance through the owner's read.
/// No hardware NMI state, elapsed time or future observation supplies acknowledgment.
/// # Errors
/// Rejects incomplete memory and an unqualified current software request.
pub fn publication(bytes: &[u8]) -> Result<FrameContinuationAfterNmi, BorrowedReceiptFault> {
    if bytes.len() != 131_072 {
        return Err(BorrowedReceiptFault::Length);
    }
    if bytes
        .get(1460)
        .copied()
        .ok_or(BorrowedReceiptFault::Length)?
        != 0
    {
        return Err(BorrowedReceiptFault::UnsupportedRequest);
    }
    Ok(FrameContinuationAfterNmi {
        publication: PublicationOrigin {
            prior_request: SoftwareRequest::Acknowledged,
        },
    })
}
