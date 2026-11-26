//! Prepare the room's working colours and publish them when NMI accepts the frame.

use crate::state::video::Colour;

impl super::State {
    pub(super) fn prepare_room_palette(&mut self, publishes: bool) {
        let video = &mut self.room_video.video;
        video.batch.cgram = self.palette.screen.map(|colour| Colour(colour.bits()));
        if publishes {
            video.upload_cgram();
        }
    }
}
