//! Complete native Ridley lifecycle orchestration from pinned snesrev bank A6.
//!
//! State, movement, scripts and rendering use canonical WRAM. Services whose
//! ownership lies outside this subsystem cross explicit synchronous host calls.
pub use crate::enemy_projectiles::ProjectileHost;
use crate::{Wram, audio, enemy, enemy_projectiles, observed_projectile_ownership};

#[path = "ridley_subsystem_admission.rs"]
mod admission;
#[path = "ridley_cutscene.rs"]
mod cutscene;
#[path = "ridley_subsystem_instructions.rs"]
mod instructions;
pub use instructions::ScriptReport;
#[path = "ridley_movement.rs"]
mod movement;
#[path = "ridley_rendering.rs"]
mod rendering;

pub type Result<T> = core::result::Result<T, &'static str>;

/// Native services retain their individual ownership outside Ridley's AI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    SpawnEnemy { bank: u8, population: u16 },
    EnemyDeath { slot: u16, animation: u16 },
    EnemyItemDrop { slot: u16 },
    NormalTouch,
    NormalShot,
    NormalPowerbomb,
    SamusCode { code: u16 },
}

/// The caller publishes canonical WRAM and all host words before each service.
/// A service error after acceptance terminates the Rust-owned invocation.
/// Hosts must preserve each native service's WRAM and host-word write footprint;
/// admission cannot validate a foreign host that mutates unrelated gameplay state.
pub trait Host {
    fn service(
        &mut self,
        ram: &mut Wram<'_>,
        rom: &[u8],
        projectile: &mut ProjectileHost,
        service: Service,
    ) -> Result<u32>;
    /// Exact register order: M7A low, M7A high, M7B factor, MPYM low/high reads.
    fn mode7_multiply(&mut self, sine: i16, factor: u8) -> Result<u16>;
    fn ordered_audio(&mut self, channel: u8, sound: u16, maximum: u8) -> Result<()>;
    fn ordered_music(&mut self, track: u16, delay: u16) -> Result<()>;
    fn ordered_projectile(&mut self, definition: u16, parameter: u16, slot: Option<u16>) -> Result<()>;
}

const ENEMY: usize = 0xf78;
const CURRENT: usize = 0xe54;
const TANGIBLE: u16 = 0x400;
const INVISIBLE: u16 = 0x100;
const DELETED: u16 = 0x200;

fn record(slot: u16) -> Result<usize> {
    if slot >= 0x800 || slot & 0x3f != 0 {
        Err("Ridley enemy record must be aligned below 0x800")
    } else {
        Ok(ENEMY + usize::from(slot))
    }
}
fn var_address(slot: u16, index: u16) -> Result<usize> {
    record(slot)?;
    let base = match index {
        0x00..=0x1f => 0x7800,
        0x20..=0x3f => 0x8000,
        0x40..=0x5f => 0x8800,
        _ => return Err("Ridley variable index exceeds the native record"),
    };
    Ok(base + usize::from(slot) + usize::from(index & 0x1f) * 2)
}
fn negative(value: u16) -> bool {
    value & 0x8000 != 0
}
fn abs(value: u16) -> u16 {
    if negative(value) { value.wrapping_neg() } else { value }
}
fn rom_offset(address: u32) -> usize {
    (((address >> 16) & 0x7f) * 0x8000 + (address & 0x7fff)) as usize
}
fn rom_byte(rom: &[u8], address: u32) -> Result<u8> {
    rom.get(rom_offset(address)).copied().ok_or("Ridley ROM read outside image")
}
fn rom_word(rom: &[u8], address: u32) -> Result<u16> {
    if address & 0xffff == 0xffff {
        return Err("Ridley ROM word crosses bank");
    }
    Ok(u16::from_le_bytes([rom_byte(rom, address)?, rom_byte(rom, address + 1)?]))
}

struct Context<'a, 'r, H> {
    ram: &'a mut Wram<'r>,
    rom: &'a [u8],
    projectile: &'a mut ProjectileHost,
    host: &'a mut H,
    calls: usize,
}
impl<H: Host> Context<'_, '_, H> {
    fn w(&self, at: usize) -> u16 {
        self.ram.word(at)
    }
    fn set(&mut self, at: usize, value: u16) {
        self.ram.set_word(at, value);
    }
    fn v(&self, index: u16) -> u16 {
        self.ram.word(var_address(0, index).expect("native Ridley variable"))
    }
    fn sv(&mut self, index: u16, value: u16) {
        self.ram.set_word(var_address(0, index).expect("native Ridley variable"), value);
    }
    fn t(&self, index: usize) -> u16 {
        self.ram.word(0x2000 + index * 2)
    }
    fn st(&mut self, index: usize, value: u16) {
        self.ram.set_word(0x2000 + index * 2, value);
    }
    fn a(&self) -> u16 {
        self.w(ENEMY + 0x30)
    }
    fn sa(&mut self, value: u16) {
        self.set(ENEMY + 0x30, value);
    }
    fn b(&self) -> u16 {
        self.w(ENEMY + 0x32)
    }
    fn sb(&mut self, value: u16) {
        self.set(ENEMY + 0x32, value);
    }
    fn c(&self) -> u16 {
        self.w(ENEMY + 0x34)
    }
    fn sc(&mut self, value: u16) {
        self.set(ENEMY + 0x34, value);
    }
    fn e(&self) -> u16 {
        self.w(ENEMY + 0x38)
    }
    fn se(&mut self, value: u16) {
        self.set(ENEMY + 0x38, value);
    }
    fn f(&self) -> u16 {
        self.w(ENEMY + 0x3a)
    }
    fn sf(&mut self, value: u16) {
        self.set(ENEMY + 0x3a, value);
    }
    fn decrement_f(&mut self) -> bool {
        let value = self.f().wrapping_sub(1);
        self.sf(value);
        negative(value)
    }
    fn rw(&self, address: u16) -> Result<u16> {
        rom_word(self.rom, 0xa60000 | u32::from(address))
    }
    fn table(&self, address: u16, index: u16) -> Result<u16> {
        self.rw(address
            .checked_add(index.checked_mul(2).ok_or("Ridley table index overflow")?)
            .ok_or("Ridley table extent")?)
    }
    fn animation(&mut self, address: u16, timer: u16) {
        self.set(ENEMY + 0x1a, address);
        self.set(ENEMY + 0x1c, timer);
        self.set(ENEMY + 0x18, 0);
    }
    fn service(&mut self, service: Service) -> Result<u32> {
        self.host.service(self.ram, self.rom, self.projectile, service)
    }
    fn sound(&mut self, channel: u8, sound: u16, maximum: u8) -> Result<()> {
        let channel = channel.checked_sub(1).filter(|channel| *channel < 3).ok_or("Ridley sound channel extent")?;
        audio::queue_sfx(self.ram, usize::from(channel), (sound << 8) | u16::from(maximum))?;
        self.host.ordered_audio(channel, sound, maximum)
    }
    fn music(&mut self, track: u16) -> Result<()> {
        audio::queue_music(self.ram, track, 8, true)?;
        self.host.ordered_music(track, 8)
    }
    fn angle(&mut self, x: u16, y: u16) -> Result<u8> {
        Ok(enemy::dispatch_rom(self.ram, self.rom, "CalculateAngleFromXY", i32::from(x), i32::from(y))? as u8)
    }
    fn component(&self, amplitude: u16, angle: u8) -> u16 {
        enemy_projectiles::beam_component(amplitude, angle)
    }
    fn random(&mut self) -> u16 {
        enemy_projectiles::random(self.ram)
    }
    fn spawn(&mut self, definition: u16, parameter: u16) -> Result<()> {
        let before = (0..36).step_by(2).map(|k| self.w(0x1997 + k)).collect::<Vec<_>>();
        if definition == 0x9642 {
            observed_projectile_ownership::dispatch_host(
                self.ram,
                self.rom,
                self.projectile,
                "SpawnEprojWithRoomGfx",
                i32::from(definition),
                i32::from(parameter),
            )?;
        } else {
            enemy_projectiles::spawn_room_with_host(self.ram, self.rom, definition, parameter, self.projectile)?;
        }
        let slot = before
            .iter()
            .enumerate()
            .find_map(|(i, before)| (*before == 0 && self.w(0x1997 + i * 2) != 0).then_some((i * 2) as u16));
        self.host.ordered_projectile(definition, parameter, slot)
    }
    fn damage(&mut self, amount: u16) -> Result<()> {
        let amount = if self.w(0x9a2) & 0x20 != 0 {
            amount >> 2
        } else if self.w(0x9a2) & 1 != 0 {
            amount >> 1
        } else {
            amount
        };
        if negative(amount) {
            return Err("Ridley signed damage would trigger native invalid interrupt");
        }
        if amount != 300 && self.w(0xa78) == 0 {
            let value = self.w(0x9c2).wrapping_sub(amount);
            self.set(0x9c2, if negative(value) { 0 } else { value });
        }
        Ok(())
    }
    fn knockback(&mut self, x: u16) {
        self.set(0x18a8, 96);
        self.set(0x18aa, 5);
        self.set(0xa54, u16::from(!negative(self.w(0xaf6).wrapping_sub(x))));
    }
    fn samus_rect(&self, x: u16, y: u16, width: u16, height: u16) -> bool {
        let dx = abs(self.w(0xaf6).wrapping_sub(x));
        let dy = abs(self.w(0xafa).wrapping_sub(y));
        (dx < self.w(0xafe) || dx.wrapping_sub(self.w(0xafe)) < width)
            && (dy < self.w(0xb00) || dy.wrapping_sub(self.w(0xb00)) < height)
    }
    fn outside_rect(&self, slot: u16, x: u16, y: u16, width: u16, height: u16) -> Result<bool> {
        let e = record(slot)?;
        Ok(abs(x.wrapping_sub(self.w(e + 2))) >= self.w(e + 0xa).wrapping_add(width).wrapping_add(1)
            || abs(y.wrapping_sub(self.w(e + 6))) >= self.w(e + 0xc).wrapping_add(height).wrapping_add(1))
    }
    fn boss_bit(&mut self) -> Result<()> {
        let area = usize::from(self.w(0x79f));
        if area >= 8 {
            return Err("Ridley boss area extent");
        }
        self.ram.set_byte(0xd828 + area, self.ram.byte(0xd828 + area) | 1);
        Ok(())
    }
    fn initialize_death(&mut self) {
        if self.w(ENEMY + 0x14) == 0 && !negative(self.v(1)) {
            self.sv(1, 0xffff);
            self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) | TANGIBLE);
            self.sa(0xc538);
        }
    }
    fn offscreen(&self) -> bool {
        let x = self.w(ENEMY + 2);
        let y = self.w(ENEMY + 6);
        let sx = x.wrapping_add(32).wrapping_sub(self.w(0x911));
        let sy = y.wrapping_add(32).wrapping_sub(self.w(0x915));
        negative(x)
            || negative(y)
            || negative(sx)
            || negative(sy)
            || !negative(sx.wrapping_sub(320))
            || !negative(sy.wrapping_sub(288))
    }
    fn update_tangible(&mut self) {
        let active = self.v(1);
        if negative(active) {
            return;
        }
        if active != 0 {
            if self.offscreen() {
                self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) | TANGIBLE);
                return;
            }
            self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) & !TANGIBLE);
        }
        let timer = self.v(0x1e);
        if timer != 0 {
            self.sv(0x1e, timer - 1);
            if timer == 1 && !negative(self.v(1)) {
                self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) & !TANGIBLE);
            }
        }
    }
    fn handle_powerbomb_state(&mut self) {
        if !negative(self.v(1)) && self.v(1) != 0 && self.v(1) != 2 && self.w(0xcee) != 0 && self.v(0x1b) == 0 {
            self.st(0, 1);
            self.st(10, 1);
            self.sa(0xbab7);
        }
    }
    fn grabbed_offset(&self) -> Result<(u16, u16)> {
        let o = self.v(0x10);
        let stance = self.v(0x1d) >> 1;
        Ok((self.table(0xb9d5, o)?, self.table(0xb9db, stance)?))
    }
    fn grab(&mut self) -> Result<()> {
        let (x, y) = self.grabbed_offset()?;
        self.sv(0x14, self.w(0xaf6).wrapping_sub(self.w(ENEMY + 2).wrapping_add(x)));
        self.sv(0x15, self.w(0xafa).wrapping_sub(self.w(ENEMY + 6).wrapping_add(y)));
        self.set(ENEMY + 0xe, self.w(ENEMY + 0xe) | TANGIBLE);
        self.sv(0x1b, 1);
        self.service(Service::SamusCode { code: 0 })?;
        self.carry_graphics(true)
    }
    fn release(&mut self) -> Result<()> {
        self.st(2, 1);
        self.st(0, 1);
        if !negative(self.v(1)) {
            self.sv(0x1e, if self.morph_carry()? { 6 } else { 10 });
        }
        self.sv(0x1b, 0);
        self.service(Service::SamusCode { code: 1 })?;
        self.carry_graphics(false)
    }
    fn drag_samus(&mut self) -> Result<()> {
        for index in [0x14, 0x15] {
            let value = self.v(index);
            if value != 0 {
                let magnitude = abs(value).wrapping_sub(4);
                self.sv(
                    index,
                    if negative(magnitude) {
                        0
                    } else if negative(value) {
                        magnitude.wrapping_neg()
                    } else {
                        magnitude
                    },
                );
            }
        }
        let (x, y) = self.grabbed_offset()?;
        self.set(0xaf6, self.v(0x14).wrapping_add(self.w(ENEMY + 2)).wrapping_add(x));
        self.set(0xafa, self.v(0x15).wrapping_add(self.w(ENEMY + 6)).wrapping_add(y));
        Ok(())
    }
    fn grab_rect(&self, width: u16, height: u16) -> Result<bool> {
        let (x, y) = self.grabbed_offset()?;
        Ok(self.samus_rect(self.w(ENEMY + 2).wrapping_add(x), self.w(ENEMY + 6).wrapping_add(y), width, height))
    }
    fn block_solid(&self, x: u16, y: u16) -> Result<bool> {
        let index = u16::from((y >> 4) as u8).wrapping_mul(u16::from(self.w(0x7a5) as u8)).wrapping_add(x >> 4);
        let p = 0x10002 + usize::from(index) * 2;
        if p + 2 > crate::WRAM_SIZE {
            return Err("Ridley block probe WRAM extent");
        }
        Ok(self.w(p) & 0xf000 != 0)
    }
    fn landing_tail(&self) -> Result<bool> {
        for (x, y, add) in [(82, 83, 16), (72, 73, 18), (62, 63, 18), (52, 53, 18), (42, 43, 18)] {
            if self.block_solid(self.t(x), self.t(y).wrapping_add(add))? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn reflect_horizontal(&mut self) {
        if self.b() == 0 {
            self.sb(if self.w(ENEMY + 2) & 0x80 != 0 { (-192i16) as u16 } else { 192 });
        }
        let x = self.w(ENEMY + 2);
        let lower = negative(x.wrapping_sub(self.v(0x22)));
        let upper = !negative(x.wrapping_sub(self.v(0x23)));
        let towards = (self.b() ^ x.wrapping_sub(self.w(0xaf6))) & 0x8000 == 0;
        let reflect = if lower {
            negative(self.b())
        } else if upper {
            !negative(self.b())
        } else if towards {
            self.w(0x5e5) >= 0x555
        } else {
            self.w(0x5e5) < 0x555
        };
        if reflect {
            self.sb(self.b().wrapping_neg());
        }
    }
    fn bounce_velocity(&mut self) -> Result<()> {
        let index = self.w(0x5e5) & 3;
        let x = self.table(0xb965, index)?;
        let y = self.table(0xb96d, index)?;
        let tier = self.v(0x12).wrapping_add(2);
        self.st(7, self.table(0xb94d, tier)?);
        self.st(6, self.table(0xb959, tier)?);
        self.sc(self.table(y, tier)?);
        let speed = self.table(x, tier)?;
        self.sb(if negative(self.b()) { speed.wrapping_neg() } else { speed });
        Ok(())
    }
    fn choose_state(&mut self) -> Result<()> {
        let health = self.w(ENEMY + 0x14);
        let source = if self.ram.byte(0xa1f) == 3 {
            0xb3cc
        } else if health == 0 {
            self.sv(0x25, self.v(0x25).wrapping_add(1));
            0xb3dc
        } else if negative(health.wrapping_sub(14400)) {
            0xb38c
        } else if !negative(self.w(0xafa).wrapping_sub(352)) {
            0xb3bc
        } else if self.can_carry()? {
            0xb3ac
        } else if negative(health.wrapping_sub(9000)) {
            0xb39c
        } else {
            0xb38c
        };
        let random = self.random();
        let next = self.table(source, random & 7)?;
        self.sa(next);
        self.run_state(next)
    }
    fn post_attack_choice(&mut self) -> Result<()> {
        if negative(self.v(0x25).wrapping_sub(10)) {
            if self.w(0xcee) != 0 {
                self.sa(0xbd4e);
                self.run_state(0xbd4e)
            } else {
                self.sa(0xb3ec);
                self.st(2, 1);
                Ok(())
            }
        } else {
            self.initialize_death();
            self.sa(0xc53e);
            self.run_state(0xc53e)
        }
    }
    fn escape_grab(&mut self) -> Result<()> {
        let o = self.v(0x10);
        if o > 2 {
            return Err("Ridley grab orientation extent");
        }
        self.sv(0x17, [64, 0, 208][usize::from(o)]);
        let y = self.w(ENEMY + 6);
        self.sv(0x18, if negative(y.wrapping_sub(320)) { 256 } else { y.wrapping_sub(64) });
        if self.v(0x1b) == 0 {
            self.grab()?;
        }
        self.sa(0xbbc4);
        self.sf(32);
        self.run_state(0xbbc4)
    }
    fn contact_projectile(&mut self, x: u16, y: u16, width: u16, height: u16) -> Option<u16> {
        if self.w(0xcce) == 0 {
            return None;
        }
        for slot in (0..10).step_by(2) {
            let kind = self.w(0xc18 + slot);
            if !negative(kind) || !negative(((kind >> 8) & 15).wrapping_sub(3)) {
                continue;
            }
            let dx = abs(self.w(0xb64 + slot).wrapping_sub(x));
            let dy = abs(self.w(0xb78 + slot).wrapping_sub(y));
            if (dx < self.w(0xbb4 + slot) || dx.wrapping_sub(self.w(0xbb4 + slot)) < width)
                && (dy < self.w(0xbc8 + slot) || dy.wrapping_sub(self.w(0xbc8 + slot)) < height)
            {
                self.set(0xb64 + slot, x);
                self.set(0xb78 + slot, y);
                self.set(0xc04 + slot, self.w(0xc04 + slot) | 0x10);
                return Some(slot as u16);
            }
        }
        None
    }
    fn projectile_contact(&mut self) -> Result<()> {
        if self.w(ENEMY + 0xe) & TANGIBLE != 0 {
            return Ok(());
        }
        if let Some(slot) = self
            .contact_projectile(self.t(82), self.t(83), 14, 14)
            .or_else(|| self.contact_projectile(self.t(72), self.t(73), 10, 10))
        {
            self.projectile.x = self.w(0xb64 + usize::from(slot));
            self.projectile.y = self.w(0xb78 + usize::from(slot));
            let missile = ((self.w(0xc18 + usize::from(slot)) >> 8) & 15) == 1;
            if missile {
                self.sound(1, 0x3d, 6)?;
            }
            self.spawn(0xe509, if missile { 6 } else { 12 })?;
        }
        Ok(())
    }
    fn main(&mut self, hurt: bool) -> Result<()> {
        if !hurt {
            let value = self.v(0x24).wrapping_sub(4);
            self.sv(0x24, if negative(value) { 0 } else { value });
        }
        let even = self.w(ENEMY + 0x2c) & 1 == 0;
        if !hurt || even {
            self.update_tangible();
            self.handle_powerbomb_state();
            self.run_state(self.a())?;
            if self.v(2) == 0 && hurt {
                if self.v(0x1b) != 0 {
                    self.drag_samus()?;
                }
                return Ok(());
            }
        }
        if self.v(2) != 0 || hurt {
            if !hurt {
                self.update_palette_index();
                self.integrate(self.w(CURRENT), true)?;
                self.update_wing_animation();
                self.tail_main()?;
                self.projectile_contact()?;
                self.draw_tail()?;
                self.draw_body()?;
                self.graphics_timer()?;
            } else {
                if even {
                    self.integrate(self.w(CURRENT), true)?;
                    self.update_wing_animation();
                    self.tail_main()?;
                }
                self.projectile_contact()?;
                self.update_palette_index();
                self.draw_tail()?;
                self.draw_body()?;
                self.health_palette()?;
                self.graphics_timer()?;
                let value = self.v(0x24).wrapping_add(1);
                self.sv(0x24, if !negative(value.wrapping_sub(8)) { 8 } else { value });
            }
            if self.v(0x1b) != 0 && (!hurt || !negative(self.v(0x24).wrapping_sub(8))) {
                self.drag_samus()?;
            }
        }
        if !hurt {
            self.health_palette()?;
        }
        Ok(())
    }
    fn run_state(&mut self, address: u16) -> Result<()> {
        self.calls += 1;
        if self.calls > 4096 {
            return Err("Ridley lifecycle state recursion limit");
        }
        if self.ceres_state(address)? || self.death_state(address)? {
            return Ok(());
        }
        match address {
            0xb2f3 => {
                self.seek(0, 14, Some(0), 64, 256)?;
                if !self.outside_rect(0, 64, 256, 8, 8)? {
                    self.sv(1, 1);
                    self.sa(0xb321);
                    self.choose_state()?;
                }
            }
            0xb321 => self.choose_state()?,
            0xb3ec => {
                self.sa(0xb3f8);
                self.sf(128);
                self.run_state(0xb3f8)?;
            }
            0xb3f8 => {
                let mut done = self.decrement_f();
                if !done {
                    let x = if self.v(0x10) != 0 { 96 } else { 192 };
                    self.seek(0, self.speed_profile()?, Some(0), x, 256)?;
                    done = !self.outside_rect(0, x, 256, 8, 8)?;
                }
                if done {
                    self.sa(0xb321);
                }
            }
            0xb441 => {
                self.sa(0xb455);
                self.sv(0, 10);
                self.sv(0x0a, 0);
                self.run_state(0xb455)?;
            }
            0xb455 => {
                let x = if self.v(0x10) != 0 { 64 } else { 192 };
                self.seek(0, 1, Some(0), x, 128)?;
                if !self.outside_rect(0, x, 128, 8, 8)? {
                    self.sa(0xb493);
                    self.sv(0, 32);
                    self.sv(0x0a, 0);
                }
            }
            0xb493 | 0xb4d1 | 0xb516 | 0xb554 | 0xb594 => {
                let o = self.v(0x10) != 0;
                let (acceleration, target, speed, next, timer) = match address {
                    0xb493 => {
                        (if o { 32 } else { (-32i16) as u16 }, if o { 512 } else { (-512i16) as u16 }, 1152, 0xb4d1, 20)
                    }
                    0xb4d1 => (
                        if o { 320 } else { (-320i16) as u16 },
                        if o { 0x4000 } else { (-16384i16) as u16 },
                        1280,
                        0xb516,
                        16,
                    ),
                    0xb516 => (
                        if o { 512 } else { (-512i16) as u16 },
                        if o { 30720 } else { (-30720i16) as u16 },
                        768,
                        0xb554,
                        32,
                    ),
                    0xb554 => (
                        if o { 1024 } else { (-1024i16) as u16 },
                        if o { 30720 } else { (-30720i16) as u16 },
                        768,
                        0xb594,
                        32,
                    ),
                    _ => (0, 0x8000, 448, 0, 0),
                };
                self.arc(acceleration, target, speed);
                if self.v(0) != 0 {
                    self.sv(0, self.v(0) - 1);
                } else {
                    let next = if address == 0xb594 { if self.can_carry()? { 0xbab7 } else { 0xb321 } } else { next };
                    self.sa(next);
                    self.sv(0, timer);
                    if address == 0xb4d1 {
                        self.st(2, 1);
                    }
                    if address == 0xb554 {
                        self.flip_animation()?;
                    }
                }
            }
            0xb5c4 => {
                self.st(15, 11);
                self.st(9, 384);
                self.sa(0xb5e5);
                self.sf((self.w(0x5e5) & 31) + 32);
                self.run_state(0xb5e5)?;
            }
            0xb5e5 | 0xb613 => {
                let x = self.table(if address == 0xb5e5 { 0xb60d } else { 0xb63b }, self.v(0x10))?;
                let y = if !negative(self.w(0xafa).wrapping_sub(352)) { 352 } else { self.w(0xafa) };
                let profile = self.speed_profile()?;
                self.seek(0, profile, Some(0), x, y)?;
                self.st(2, 1);
                if self.ram.byte(0xa1f) != 3 {
                    self.st(9, 240);
                    self.st(15, 16);
                    self.st(0, 1);
                    self.sa(0xb6a7);
                } else {
                    if self.ram.byte(0x5e5) >= 0x80 && self.v(0x0f) == 0 && self.v(0x10) != 1 {
                        self.animation(0xe73a, 1);
                    }
                    if self.decrement_f() {
                        self.sa(if address == 0xb5e5 { 0xb613 } else { 0xb5e5 });
                        self.sf(128);
                        self.flip_animation()?;
                    }
                }
            }
            0xb6a7 => {
                if negative(self.w(ENEMY + 6).wrapping_sub(288)) {
                    self.flip_animation()?;
                    self.sa(0xb6dd);
                    self.sf(32);
                    self.run_state(0xb6dd)?;
                } else {
                    self.seek(0, 0, Some(0), self.table(0xb6c8, self.v(0x10))?, 288)?;
                }
            }
            0xb6dd => {
                self.seek(0, 0, Some(0), self.w(ENEMY + 2), 288)?;
                if self.decrement_f() {
                    self.tail_phase(2)?;
                    self.bounce_velocity()?;
                    self.sa(0xb70e);
                    self.sf((self.w(0x5e5) & 63) + 128);
                }
            }
            0xb70e => {
                if self.can_carry()? && self.grab_rect(4, 4)? {
                    self.st(0, 1);
                    self.st(10, 1);
                    let speed = if negative(self.c()) { self.c() } else { self.c().wrapping_neg() };
                    self.sc(if !negative(speed.wrapping_add(512)) { (-512i16) as u16 } else { speed });
                    self.sa(0xbb8f);
                    self.escape_grab()?;
                } else {
                    let velocity = self.t(6).wrapping_add(self.c());
                    self.sc(if !negative(velocity.wrapping_sub(1536)) { 1536 } else { velocity });
                    if self.landing_tail()? {
                        self.projectile.x = self.t(82);
                        self.projectile.y = self.t(83).wrapping_add(12);
                        self.spawn(0xe509, 9)?;
                        self.sound(2, 0x76, 6)?;
                        self.set(0x183e, 13);
                        self.set(0x1840, 4);
                        self.reflect_horizontal();
                        self.bounce_velocity()?;
                        self.tail_copy(0x2026, 0xd37c)?;
                        for i in (20..=80).step_by(10) {
                            self.st(i, 3072);
                        }
                        self.st(0, 4);
                        let prior = self.v(0x26);
                        let next = if !negative(prior.wrapping_sub(1)) {
                            if self.v(0x10) != 1 {
                                self.animation(0xe73a, 1);
                            }
                            0
                        } else {
                            prior.wrapping_add(1)
                        };
                        self.sv(0x26, next);
                        self.sa(0xb7b9);
                    }
                }
            }
            0xb7b9 => {
                if negative(self.w(0xafa).wrapping_sub(352)) || self.decrement_f() {
                    self.st(0, 1);
                    self.st(10, 1);
                    self.sa(0xb321);
                } else {
                    let velocity = self.t(7).wrapping_add(self.c());
                    self.sc(velocity);
                    if !negative(velocity) {
                        self.sc(0);
                        self.sa(0xb70e);
                    }
                }
            }
            0xbab7 => {
                let o = self.v(0x10);
                let delta = self.w(ENEMY + 2).wrapping_sub(self.w(0xaf6));
                let side = self.table(0xbb48, o)?;
                if self.can_carry()?
                    && self.v(0x1f) == 0
                    && ((side ^ delta) & 0x8000 == 0 || negative(abs(delta).wrapping_sub(32)))
                    && negative(self.w(ENEMY + 6).wrapping_add(35).wrapping_sub(self.w(0xafa)))
                {
                    let x = self.w(0xaf6).wrapping_add(if o != 0 { 16 } else { (-16i16) as u16 });
                    self.seek(0, self.table(0xbb4e, self.v(0x12))?, Some(0), x, self.w(0xafa).wrapping_sub(4))?;
                    let (gx, _) = self.grabbed_offset()?;
                    if self.samus_rect(self.w(ENEMY + 2).wrapping_add(gx), self.w(ENEMY + 6).wrapping_add(35), 8, 12) {
                        self.sc(self.c().wrapping_neg());
                        if self.w(ENEMY + 0x14) != 0 {
                            if self.w(0xcee) != 0 {
                                self.grab()?;
                                self.sa(0xbd4e);
                                self.run_state(0xbd4e)?;
                            } else {
                                self.escape_grab()?;
                            }
                        } else {
                            if self.v(0x1b) == 0 {
                                self.grab()?;
                            }
                            self.initialize_death();
                            self.sa(0xc538);
                            self.run_state(0xc538)?;
                        }
                    }
                } else {
                    self.post_attack_choice()?;
                }
            }
            0xbb8f => self.escape_grab()?,
            0xbbc4 => {
                self.seek(0, 0, Some(0), self.v(0x17), self.v(0x18))?;
                if self.decrement_f() {
                    self.sa(0xbbf1);
                    self.sf(32);
                }
            }
            0xbbf1 => {
                if self.decrement_f() {
                    self.st(15, 8);
                    self.st(9, 240);
                    self.release()?;
                    self.sa(0xbc2e);
                    self.sf(64);
                } else {
                    self.seek(0, 0, Some(0), self.v(0x17), 256)?;
                }
            }
            0xbc2e => {
                if self.decrement_f() {
                    self.st(15, 16);
                    self.st(9, 240);
                    self.sa(0xb321);
                } else {
                    let o = self.v(0x10);
                    if o > 2 {
                        return Err("Ridley release orientation extent");
                    }
                    self.seek(0, 0, Some(0), [176, 0, 80][usize::from(o)], 224)?;
                }
            }
            0xbd4e => {
                if self.w(0xcee) != 0 {
                    self.sv(1, 2);
                    let x = if negative(self.w(0xce2).wrapping_sub(128)) { 192 } else { 80 };
                    let y = if negative(self.w(0xce4).wrapping_sub(256)) { 384 } else { 192 };
                    self.seek(0, self.speed_profile()?, Some(0), x, y)?;
                } else {
                    self.sv(1, 1);
                    self.sa(if self.v(0x1b) != 0 { 0xbb8f } else { 0xb321 });
                }
            }
            0xbd9a => {
                self.seek(0, 1, None, 192, 128)?;
                if !negative(self.w(ENEMY + 2).wrapping_sub(192)) {
                    self.sa(0xbdbc);
                }
            }
            0xbdbc => {
                self.sv(0x20, (-192i16) as u16);
                self.seek(0, 1, None, 192, (-128i16) as u16)?;
                if negative(self.w(ENEMY + 6).wrapping_sub(32)) {
                    self.sv(0x40, 0xbeca);
                    self.sa(0xbdf2);
                    self.sf(21);
                }
            }
            0xbdf2 => {
                if self.decrement_f() {
                    self.animation(0xe658, 1);
                    self.sa(0xbe03);
                    self.run_state(0xbe03)?;
                }
            }
            0xbe03 => {
                self.seek(0, 12, Some(0), self.v(0x42).wrapping_sub(10), self.v(0x44).wrapping_sub(56))?;
                let dx = abs(self.v(0x42).wrapping_sub(self.w(ENEMY + 2).wrapping_add(14)));
                let dy = abs(self.v(0x44).wrapping_sub(self.w(ENEMY + 6).wrapping_add(66)));
                if (dx < 4 || dx.wrapping_sub(4) < 4) && (dy < 4 || dy.wrapping_sub(4) < 4) {
                    self.sv(0x1b, 1);
                    self.carry_graphics(true)?;
                    self.sc((-512i16) as u16);
                    self.sv(0x40, 0xbeb3);
                    self.sa(0xa971);
                }
            }
            _ => return Err("Ridley state is outside complete native lifecycle"),
        }
        Ok(())
    }
    fn flip_animation(&mut self) -> Result<()> {
        let orientation = self.v(0x10);
        let overlapping = self.w(ENEMY + 1);
        if orientation == 0 {
            if negative(overlapping) {
                return Ok(());
            }
            self.animation(0xe6f0, 2);
        } else if orientation != 1 && negative(overlapping) {
            self.animation(0xe706, 2);
        }
        Ok(())
    }
}

/// Immutable selection of the whole lifecycle boundary, before any gameplay mutation.
pub fn supports_call(ram: &Wram<'_>, rom: &[u8], symbol: &str) -> Result<bool> {
    admission::supports(ram, rom, symbol)
}

/// Immutable direct-callback selection for the actual pinned script boundary.
pub fn supports_callback(ram: &Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<bool> {
    instructions::callback_admits(ram, rom, symbol, a, b)
}

pub fn dispatch_host(
    ram: &mut Wram<'_>,
    rom: &[u8],
    projectile: &mut ProjectileHost,
    host: &mut impl Host,
    symbol: &str,
) -> Result<u32> {
    if !supports_call(ram, rom, symbol)? {
        return Err("unsupported complete Ridley lifecycle invocation");
    }
    execute(ram, rom, projectile, host, symbol)
}

fn execute(
    ram: &mut Wram<'_>,
    rom: &[u8],
    projectile: &mut ProjectileHost,
    host: &mut impl Host,
    symbol: &str,
) -> Result<u32> {
    if symbol == "ProcessEnemyInstructions" {
        Context { ram, rom, projectile, host, calls: 0 }.instructions(4096)?;
        return Ok(0);
    }
    if symbol == "Ridley_Func_70" {
        Context { ram, rom, projectile, host, calls: 0 }.tail_main()?;
        return Ok(0);
    }
    let mut context = Context { ram, rom, projectile, host, calls: 0 };
    match symbol {
        "CeresRidley_Init" => context.initialize()?,
        "CeresRidley_Main" => context.ceres_main()?,
        "CeresRidley_Hurt" => {
            context.update_palette_index();
            context.draw_tail()?;
            context.draw_body()?;
            context.ceres_draw_hook()?;
        }
        "Ridley_Main" => context.main(false)?,
        "Ridley_Hurt" => context.main(true)?,
        "Ridley_Func_2" => {
            context.sv(0x24, 0);
            context.set(ENEMY + 0x2c, 1);
            context.main(true)?;
        }
        "Ridley_Shot" => {
            if context.w(0x79f) == 2 {
                context.service(Service::NormalShot)?;
            } else {
                let flash = context.w(ENEMY + 0x24);
                context.set(ENEMY + 0x24, if flash != 0 && flash & 1 != 0 { 14 } else { 13 });
                context.sv(0x0d, context.v(0x0d).wrapping_add(1));
            }
        }
        "Ridley_Powerbomb" => {
            context.service(Service::NormalPowerbomb)?;
        }
        "sub_A6DF59" => {
            context.service(Service::NormalTouch)?;
        }
        "sub_A6DF60" => {
            let e = record(context.w(CURRENT))?;
            let amount = rom_word(rom, 0xa00000 | (u32::from(context.w(e)) + 6))?;
            context.damage(amount)?;
            context.knockback(context.w(e + 2));
        }
        "Ridley_A2F2" => context.ceres_draw()?,
        "HandleCeresRidleyGetawayCutscene" => {
            if context.v(0x32) == 0 {
                context.getaway()?;
            }
        }
        "RidleysExplosion_Init" => context.explosion_init()?,
        "RidleysExplosion_Main" => context.explosion_main()?,
        _ => return Err("Ridley lifecycle entry vanished after admission"),
    }
    Ok(0)
}

/// Controlled differential seam. The fixture manifest declares the domain;
/// this entry deliberately permits native signatures outside live admission.
pub fn dispatch_fixture(
    ram: &mut Wram<'_>,
    rom: &[u8],
    projectile: &mut ProjectileHost,
    host: &mut impl Host,
    symbol: &str,
    a: i32,
    b: i32,
) -> Result<u32> {
    if symbol == "CallRidleyFunc" {
        let address = u32::try_from(a).map_err(|_| "Ridley state address extent")?;
        if address >> 16 != 0xa6 {
            return Err("Ridley state address outside native bank");
        }
        Context { ram, rom, projectile, host, calls: 0 }.run_state(address as u16)?;
        return Ok(0);
    }
    if symbol == "Ridley_Func_70" {
        Context { ram, rom, projectile, host, calls: 0 }.tail_main()?;
        return Ok(0);
    }
    if symbol == "ProcessEnemyInstructions" {
        let mut context = Context { ram, rom, projectile, host, calls: 0 };
        context.instructions(4096)?;
        return Ok(0);
    }
    if symbol.starts_with("Ridley_Instr_") || matches!(symbol, "EnemyInstr_Sleep" | "EnemyInstr_Goto") {
        return Context { ram, rom, projectile, host, calls: 0 }.instruction_fixture(symbol, a, b);
    }
    execute(ram, rom, projectile, host, symbol)
}

pub fn dispatch_callback(
    ram: &mut Wram<'_>,
    rom: &[u8],
    projectile: &mut ProjectileHost,
    host: &mut impl Host,
    symbol: &str,
    a: i32,
    b: i32,
) -> Result<u32> {
    if !supports_callback(ram, rom, symbol, a, b)? {
        return Err("unsupported complete Ridley instruction callback");
    }
    Context { ram, rom, projectile, host, calls: 0 }.instruction_fixture(symbol, a, b)
}

pub fn process_instructions(
    ram: &mut Wram<'_>,
    rom: &[u8],
    projectile: &mut ProjectileHost,
    host: &mut impl Host,
    limit: usize,
) -> Result<ScriptReport> {
    if !instructions::admits(ram, rom, limit)? {
        return Err("unsupported complete Ridley instruction frame");
    }
    Context { ram, rom, projectile, host, calls: 0 }.instructions(limit)
}
