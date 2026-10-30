//! How an actor dies and leaves its item drop, as the compatibility runtime
//! still implements it.
#![forbid(unsafe_code)]
use crate::enemy_projectiles::drops::DropSource;

pub trait Deaths {
    /// Kill the current actor with death `animation`.
    fn normal_death(&mut self, animation: u16);
    /// Scatter the item drop for `source` from actor `index`.
    fn scatter_enemy_drop(&mut self, source: DropSource, index: u16);
}
