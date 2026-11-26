//! The delivery and the examination (`DeliveryBegins`, `PageFourBegins`, `ExaminationBegins`):
//! the press after page 3 brings up the delivery, its picture panning as the baby is carried
//! into view; the baby's signal brings page 4 back over it; the press after page 4 starts the
//! examination, the baby rising into view. The pan and the rise are `baby`'s, the dissolves
//! `dissolve`'s; page 4's end is a page's signal (`DiscoveryAndDeliveryPagesEnd`).

use core::num::Wrapping;

use types::intro::{Baby, BabyScript, IntroState, PageScript, Pan, ScriptPlace};
use types::{Pair, Px};

use crate::content::IntroContent;
use crate::dissolve::Crossing;
use crate::passage::{text_gives_way, text_returns};
use crate::report::{IntroEvent, Report, ScenePicture};

/// `DeliveryBegins`: the delivery picture's height, px.
const DELIVERY_HEIGHT: i16 = 8;
/// `DeliveryBegins`: where the delivery baby starts, px.
const DELIVERY_BABY_PLACE: Pair<Px> = Pair {
    x: Wrapping(84),
    y: Wrapping(139),
};
/// `ExaminationBegins`: the examination picture's height, px.
const EXAMINATION_HEIGHT: i16 = -24;
/// `ExaminationBegins`: where the examination baby starts, px.
const EXAMINATION_BABY_PLACE: Pair<Px> = Pair {
    x: Wrapping(112),
    y: Wrapping(111),
};

/// `DeliveryBegins`: the press after page 3 selects the delivery's picture at its scroll
/// (32, 8) and places the delivery baby at the start of its script, at (84, 139); the text gives
/// way to it (`text_gives_way`) with the examination's colour sets.
pub fn delivery_begins(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    report
        .events
        .push(IntroEvent::PictureSelected(ScenePicture::Delivery));
    state.pan = Pan::WIDEST;
    state.height = DELIVERY_HEIGHT;
    state.baby = Some(baby_at_start(BabyScript::Delivery, DELIVERY_BABY_PLACE));
    text_gives_way(state, Crossing::IntoDelivery, content, report);
}

/// `PageFourBegins`: the frame after the baby's signal, page 4 returns over the delivery
/// (`text_returns`).
pub fn page_four_begins(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    text_returns(
        state,
        Crossing::OutOfDelivery,
        PageScript::Four,
        content,
        report,
    );
}

/// `ExaminationBegins`: the press after page 4 selects the examination's picture at its scroll
/// (0, −24) and places the examination baby at the start of its script, at (112, 111); the text
/// gives way to it (`text_gives_way`), and `BabyRises` steps it from there.
pub fn examination_begins(state: &mut IntroState, content: &IntroContent, report: &mut Report) {
    report
        .events
        .push(IntroEvent::PictureSelected(ScenePicture::Examination));
    state.pan = Pan::AT_REST;
    state.height = EXAMINATION_HEIGHT;
    state.baby = Some(baby_at_start(
        BabyScript::Examination,
        EXAMINATION_BABY_PLACE,
    ));
    text_gives_way(state, Crossing::ToScene, content, report);
}

/// A baby at `place`, its script placed at its start.
const fn baby_at_start(script: BabyScript, place: Pair<Px>) -> Baby {
    Baby {
        place,
        script: ScriptPlace::start(script),
    }
}
