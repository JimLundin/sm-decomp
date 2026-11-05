//! How an actor dies and leaves its item drop, as the compatibility runtime
//! still implements it.

pub trait Deaths {
    /// Kill the current actor with death `animation`.
    fn normal_death(&mut self, animation: u16);
}
