//! The engine projection: observations read from the typed Session only.
//!
//! A domain is projected only where the Session owns it as typed fields
//! (REVIEW, lane S condition 4). Nothing here reads RAM, a byte projection or
//! a compatibility view; the domains this cannot project stay covered by the
//! verifier's whole-RAM comparison.

use crate::observation::{
    Actor, Actors, Camera, Domain, Effects, Music, Observation, Progress, Room, Samus, Sound,
};
use sm_game::session::Session;

/// The domains one projection covers, as a set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Domains([bool; Domain::ALL.len()]);

impl Domains {
    fn index(domain: Domain) -> usize {
        Domain::ALL
            .iter()
            .position(|&other| other == domain)
            .expect("listed domain")
    }
    pub fn insert(&mut self, domain: Domain) {
        self.0[Self::index(domain)] = true;
    }
    pub fn contains(&self, domain: Domain) -> bool {
        self.0[Self::index(domain)]
    }
}

/// The engine's observation and the domains it covers; the rest are default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Projection {
    pub observation: Observation,
    pub domains: Domains,
}

/// Project every domain `session` owns as typed fields.
pub fn project(session: &Session) -> Projection {
    let mut domains = Domains::default();
    for domain in [
        Domain::Samus,
        Domain::Camera,
        Domain::Room,
        Domain::Progress,
        Domain::Actors,
    ] {
        domains.insert(domain);
    }
    // Sound and music requests are typed only while the Session owns audio control.
    let effects = effects(session);
    if effects.is_some() {
        domains.insert(Domain::Effects);
    }
    Projection {
        observation: Observation {
            input: Default::default(),
            samus: samus(session),
            camera: camera(session),
            room: room(session),
            progress: progress(session),
            actors: actors(session),
            effects: effects.unwrap_or_default(),
        },
        domains,
    }
}

fn samus(session: &Session) -> Samus {
    let player = &session.player;
    let inventory = &session.inventory;
    let [missiles, super_missiles, power_bombs] =
        inventory.ammunition.each_ref().map(|supply| supply.current);
    Samus {
        x: player.x,
        x_fraction: player.x_fraction,
        y: player.y,
        y_fraction: player.y_fraction,
        y_speed: player.contact.y_speed.whole,
        y_speed_fraction: player.contact.y_speed.fraction,
        pose: player.pose,
        movement_and_facing: player.movement_and_facing,
        health: inventory.health.current,
        missiles,
        super_missiles,
        power_bombs,
        equipment: inventory.equipment.active,
        beams: inventory.beams.active,
        invincibility: player.invincibility,
        shine_timer: player.shine_timer,
        charge: session.projectiles.controls.charge,
    }
}

fn camera(session: &Session) -> Camera {
    let camera = &session.room.camera;
    let [layer1_x, layer1_y] = camera.layer1;
    let [layer1_x_fraction, layer1_y_fraction] = camera.layer1_fraction;
    let [layer2_x, layer2_y] = camera.layer2;
    Camera {
        layer1_x,
        layer1_x_fraction,
        layer1_y,
        layer1_y_fraction,
        layer2_x,
        layer2_y,
        distance_index: camera.distance_index,
    }
}

fn room(session: &Session) -> Room {
    Room {
        area: session.room.area,
        room: session.room_key(),
        state: session.state_key(),
        door: session.world.exit_key(&session.room.entry.exit),
        width: session.room.width,
        height: session.room.height,
    }
}

fn progress(session: &Session) -> Progress {
    let flags = &session.gameplay_flags;
    let inventory = &session.inventory;
    Progress {
        events: flags.events,
        bosses: flags.bosses_by_area,
        items: flags.collected_locations,
        doors: flags.opened_doors,
        collected_equipment: inventory.equipment.owned,
        collected_beams: inventory.beams.owned,
        max_health: inventory.health.maximum,
        max_missiles: inventory.ammunition[0].maximum,
    }
}

fn actors(session: &Session) -> Actors {
    let enemies = session
        .actors
        .slots
        .iter()
        .enumerate()
        .filter(|(_, actor)| actor.definition != 0)
        .map(|(slot, actor)| {
            let [ai_a, ai_b, ai_c, ai_d, ai_e] = actor.variables;
            Actor {
                slot: u16::try_from(slot).expect("enemy slot"),
                species: actor.definition,
                x: actor.x.whole,
                x_fraction: actor.x.fraction,
                y: actor.y.whole,
                y_fraction: actor.y.fraction,
                health: actor.health,
                ai_handlers: actor.ai_handlers,
                instruction: actor.instruction,
                frozen: actor.frozen,
                invincibility: actor.invincibility,
                ai_a,
                ai_b,
                ai_c,
                ai_d,
                ai_e,
            }
        })
        .collect();
    Actors { enemies }
}

fn effects(session: &Session) -> Option<Effects> {
    let requests = session.control.requests().ok()?;
    let mut sounds = Vec::new();
    for (library, queue) in requests.sounds.iter().enumerate() {
        let ring = queue.queue.len();
        let mut read = usize::from(queue.read) % ring;
        let write = usize::from(queue.write) % ring;
        while read != write {
            sounds.push(Sound {
                library: u8::try_from(library).expect("sound library"),
                sound: queue.queue[read],
            });
            read = (read + 1) % ring;
        }
    }
    // The music queue's positions are byte offsets into its word arrays; the
    // game keeps them even, and an entry is observed at its word.
    let music_queue = &requests.music;
    let entries = music_queue.tracks.len();
    let mut read = usize::from(music_queue.read) / size_of::<u16>() % entries;
    let write = usize::from(music_queue.write) / size_of::<u16>() % entries;
    let mut music = Vec::new();
    while read != write {
        music.push(Music {
            track: music_queue.tracks[read],
            delay: music_queue.delays[read],
        });
        read = (read + 1) % entries;
    }
    Some(Effects { sounds, music })
}
