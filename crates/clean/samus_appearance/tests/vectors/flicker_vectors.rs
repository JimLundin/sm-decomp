//! Checks body placement admission at ordinary drawing opportunities, without body art.

use samus_appearance::OrdinaryBodyDraw;
use serde::Deserialize;
use types::Parity;

use super::{behaviour, read_file};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FlickerVector {
    id: String,
    source: String,
    pub(super) entries: Vec<String>,
    protection_while_drawn: u16,
    push_time: u16,
    timed_special_colour: bool,
    parity: Parity,
    body_drawn: bool,
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report the number of supplied protection flicker vectors executed"
)]
fn protection_flicker_vectors() {
    let (file, _) = read_file();
    for vector in &file.protection_flicker {
        for entry in &vector.entries {
            behaviour(entry);
        }
        let drawing = OrdinaryBodyDraw {
            protected_while_drawn: vector.protection_while_drawn > 0,
            push_time_left: vector.push_time > 0,
            timed_special_colour: vector.timed_special_colour,
            finished_frame_parity: vector.parity,
        };
        assert_eq!(
            drawing.body_drawn(),
            vector.body_drawn,
            "{} ({}): body_drawn for {drawing:?}",
            vector.id,
            vector.source
        );
    }
    println!(
        "{} protection flicker vectors passed",
        file.protection_flicker.len()
    );
}
