//! Every vector of the discovery's vector file, run through the model, and the catalogue checked
//! against the vectors.
//!
//! A constructed vector restores its `before`, applies its inputs and compares the `after` fields
//! it supplies. A recorded vector is an observation of the shipped scene at the end of a cut, with
//! no `before`, no inputs and none of the shipped content; it is restored as a snapshot, read back
//! and carried through the saved form, so each of its fields must be one the model holds.

#[cfg(test)]
mod discovery_vectors {
    use core::num;

    use screens_discovery::catalogue::{BehaviourId, catalogue};
    use screens_discovery::scene::{
        Animation, DiscoveryScene, DrawingRef, DropletContent, FragmentContent, PaletteRef,
        ShellPictures, Shown, SoundOffer, SoundRef, SpriteRef, Track,
    };
    use screens_discovery::{Cut, Drawing, Encounter, Figure, IntroRequest, Report};
    use serde::Deserialize;
    use types::discovery::{
        BabyMotion, Coord, DiscoveryOrder, DiscoveryState, DropletStage, Droplets, Egg,
        FlyingFragment, Fragments, Hatchling, Offset, PieceId, Pulse, RoomPoint, Slime,
        SlimeDroplet, Speed,
    };
    use types::{Checked, Pair, QueueAllowance};

    /// The vector file.
    const VECTORS: &str = include_str!("../../../../scenarios/vectors/screens-discovery.toml");
    /// 65,536ths of a pixel in an eighth, the vectors' unit of position.
    const EIGHTH: i32 = 8_192;
    /// Samus's whole-pixel centre where a vector names none.
    const DEFAULT_CENTRE: [i32; 2] = [180, 147];
    /// The sample's sound offers wait behind at most six sounds.
    const SAMPLE_ALLOWANCE: u16 = 6;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        scene: SceneRows,
        vector: Vec<Vector>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SceneRows {
        home: [i32; 2],
        closed: String,
        spent: String,
        hatch: Vec<(String, u16)>,
        burst_at: u16,
        rise_at: u16,
        baby: Vec<(String, u16)>,
        fragment: Vec<(i32, i32, i32, Vec<i32>)>,
        drop: Vec<(i32, Vec<i32>)>,
        splash: Vec<(String, u16)>,
        shatter: String,
        cry: String,
    }

    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    #[serde(rename_all = "lowercase")]
    enum Source {
        Constructed,
        Recorded,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        source: Source,
        entries: Vec<String>,
        #[serde(default)]
        before: Fields,
        #[serde(default)]
        inputs: Inputs,
        after: Fields,
        row: Option<String>,
        cut: Option<u32>,
    }

    #[derive(Deserialize, Default, Clone, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct Fields {
        phase: Option<String>,
        hatch_age: Option<u16>,
        baby_phase: Option<String>,
        draw: Option<String>,
        egg: Option<String>,
        baby: Option<String>,
        x: Option<i32>,
        y: Option<i32>,
        vx: Option<i8>,
        vy: Option<i8>,
        hover: Option<u8>,
        cry: Option<u8>,
        baby_age: Option<i32>,
        drops_shed: Option<bool>,
        sound: Option<Vec<String>>,
        cue: Option<bool>,
        fragments: Option<Vec<Piece>>,
        drops: Option<Vec<Piece>>,
    }

    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct Piece {
        bit: u8,
        x: i32,
        y: i32,
        age: Option<u16>,
        splash: i32,
    }

    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct Inputs {
        start: Option<bool>,
        finish: Option<bool>,
        finished: Option<bool>,
        centre: Option<[i32; 2]>,
    }

    /// The sample's names for pictures and sounds, numbered as the scene refers to them.
    #[derive(Default)]
    struct Names {
        sprites: Vec<String>,
        sounds: Vec<String>,
    }

    impl Names {
        fn sprite(&mut self, name: &str) -> SpriteRef {
            let index = index_of(&mut self.sprites, name);
            SpriteRef {
                drawing: DrawingRef(index),
                palette: PaletteRef(0),
            }
        }

        fn sound(&mut self, name: &str) -> SoundOffer {
            SoundOffer {
                sound: SoundRef(index_of(&mut self.sounds, name)),
                allowance: QueueAllowance(SAMPLE_ALLOWANCE),
            }
        }

        fn sprite_name(&self, sprite: SpriteRef) -> String {
            self.sprites[usize::from(sprite.drawing.0)].clone()
        }

        fn sound_name(&self, offer: SoundOffer) -> String {
            assert_eq!(offer.allowance, QueueAllowance(SAMPLE_ALLOWANCE));
            self.sounds[usize::from(offer.sound.0)].clone()
        }
    }

    fn index_of(names: &mut Vec<String>, name: &str) -> u16 {
        let index = names
            .iter()
            .position(|known| known == name)
            .unwrap_or_else(|| {
                names.push(name.to_owned());
                names.len().checked_sub(1).unwrap()
            });
        u16::try_from(index).unwrap()
    }

    fn coord(eighths: i32) -> Coord {
        Coord(eighths.checked_mul(EIGHTH).unwrap())
    }

    fn offset(eighths: i32) -> Offset {
        Offset(eighths.checked_mul(EIGHTH).unwrap())
    }

    fn point(x: i32, y: i32) -> RoomPoint {
        Pair {
            x: coord(x),
            y: coord(y),
        }
    }

    fn in_eighths(at: Coord) -> i32 {
        assert_eq!(at.0.checked_rem(EIGHTH), Some(0), "not a whole eighth");
        at.0.checked_div(EIGHTH).unwrap()
    }

    fn animation(names: &mut Names, rows: &[(String, u16)]) -> Animation {
        let mut shown: Vec<Shown> = rows
            .iter()
            .map(|(name, hold)| Shown {
                sprite: names.sprite(name),
                hold: num::NonZeroU16::new(*hold).unwrap(),
            })
            .collect();
        let first = shown.remove(0);
        Animation::new(first, shown)
    }

    fn track(steps: &[i32]) -> Track {
        Track::new(steps.iter().copied().map(offset).collect()).unwrap()
    }

    fn scene(rows: &SceneRows, names: &mut Names) -> DiscoveryScene {
        let shell = ShellPictures {
            closed: names.sprite(&rows.closed),
            hatch: animation(names, &rows.hatch),
            spent: names.sprite(&rows.spent),
            scatter_at: rows.burst_at,
            release_at: rows.rise_at,
        };
        let fragments = (0..)
            .zip(&rows.fragment)
            .map(|(index, (x, y, drift, steps)): (u8, _)| FragmentContent {
                sprite: names.sprite(&format!("Fragment{index}")),
                anchor: point(*x, *y),
                drift: offset(*drift),
                track: track(steps),
            })
            .collect();
        let droplets = (0..)
            .zip(&rows.drop)
            .map(|(index, (drift, steps)): (u8, _)| DropletContent {
                sprite: names.sprite(&format!("Droplet{index}")),
                drift: offset(*drift),
                track: track(steps),
            })
            .collect();
        DiscoveryScene {
            home: point(rows.home[0], rows.home[1]),
            shell,
            baby: animation(names, &rows.baby),
            fragments,
            droplets,
            splash: animation(names, &rows.splash),
            hatch_sound: names.sound(&rows.shatter),
            cry_sound: names.sound(&rows.cry),
        }
    }

    fn restore(fields: &Fields, home: RoomPoint) -> DiscoveryState {
        let egg = match fields.phase.as_deref() {
            None | Some("absent") => Egg::Absent,
            Some("closed") => Egg::Waiting,
            Some("hatching") => Egg::Hatching {
                age: fields.hatch_age.unwrap_or(0),
            },
            Some("spent") => Egg::Spent,
            Some(other) => panic!("unknown egg phase {other}"),
        };
        let velocity = Pair {
            x: Speed(fields.vx.unwrap_or(0)),
            y: Speed(fields.vy.unwrap_or(0)),
        };
        let motion = match fields.baby_phase.as_deref() {
            None | Some("absent") => None,
            Some("inside") => Some(BabyMotion::Inside),
            Some("rising") => Some(BabyMotion::Rising {
                vertical: velocity.y,
                slime: if fields.drops_shed == Some(true) {
                    Slime::Shed
                } else {
                    Slime::Clinging
                },
            }),
            Some("hover") => Some(BabyMotion::Hovering {
                left: fields.hover.unwrap_or(0),
            }),
            Some("following") => Some(BabyMotion::Following {
                velocity,
                cry: fields.cry.unwrap_or(0),
            }),
            Some(other) => panic!("unknown baby phase {other}"),
        };
        let baby = motion.map(|motion| Hatchling {
            place: Pair {
                x: fields.x.map_or(home.x, coord),
                y: fields.y.map_or(home.y, coord),
            },
            motion,
            pulse: match fields.baby_age {
                None | Some(-1) => Pulse::Unseen,
                Some(age) => Pulse::Age(u16::try_from(age).unwrap()),
            },
        });
        let fragments = fields
            .fragments
            .iter()
            .flatten()
            .map(|piece| FlyingFragment {
                identity: PieceId(piece.bit),
                place: point(piece.x, piece.y),
                steps: piece.age.unwrap_or(0),
            });
        let droplets = fields.drops.iter().flatten().map(|piece| SlimeDroplet {
            identity: PieceId(piece.bit),
            place: point(piece.x, piece.y),
            stage: if piece.splash < 0 {
                DropletStage::Falling {
                    steps: piece.age.unwrap_or(0),
                }
            } else {
                DropletStage::Splashing {
                    age: u16::try_from(piece.splash).unwrap(),
                }
            },
        });
        DiscoveryState {
            egg,
            baby,
            fragments: Fragments::new(fragments.collect()).unwrap(),
            droplets: Droplets::new(droplets.collect()).unwrap(),
            order: match fields.draw.as_deref() {
                None | Some("Hidden") => DiscoveryOrder::Hidden,
                Some("SceneFirst") => DiscoveryOrder::SceneFirst,
                Some("SamusFirst") => DiscoveryOrder::SamusFirst,
                Some(other) => panic!("unknown draw order {other}"),
            },
        }
    }

    fn picture_of(drawings: &[Drawing], figure: Figure, names: &Names) -> String {
        drawings
            .iter()
            .find(|drawing| drawing.figure == figure)
            .map_or_else(
                || "none".to_owned(),
                |drawing| names.sprite_name(drawing.sprite),
            )
    }

    const fn baby_phase(baby: Option<&Hatchling>) -> &'static str {
        match baby {
            None => "absent",
            Some(Hatchling { motion, .. }) => match motion {
                BabyMotion::Inside => "inside",
                BabyMotion::Rising { .. } => "rising",
                BabyMotion::Hovering { .. } => "hover",
                BabyMotion::Following { .. } => "following",
            },
        }
    }

    /// The baby's speeds as the vectors read them: zero on an axis it isn't moving along.
    const fn speeds(motion: BabyMotion) -> (i8, i8) {
        match motion {
            BabyMotion::Inside | BabyMotion::Hovering { .. } => (0, 0),
            BabyMotion::Rising { vertical, .. } => (0, vertical.0),
            BabyMotion::Following { velocity, .. } => (velocity.x.0, velocity.y.0),
        }
    }

    fn observe(
        state: &DiscoveryState,
        report: &Report,
        drawings: &[Drawing],
        names: &Names,
    ) -> Fields {
        let motion = state.baby.map(|baby| baby.motion);
        let fragments = state.fragments.as_slice().iter().map(|fragment| Piece {
            bit: fragment.identity.0,
            x: in_eighths(fragment.place.x),
            y: in_eighths(fragment.place.y),
            age: Some(fragment.steps),
            splash: -1,
        });
        let drops = state.droplets.as_slice().iter().map(|droplet| {
            let (age, splash) = match droplet.stage {
                DropletStage::Falling { steps } => (Some(steps), -1),
                DropletStage::Splashing { age } => (None, i32::from(age)),
            };
            Piece {
                bit: droplet.identity.0,
                x: in_eighths(droplet.place.x),
                y: in_eighths(droplet.place.y),
                age,
                splash,
            }
        });
        Fields {
            phase: Some(
                match state.egg {
                    Egg::Absent => "absent",
                    Egg::Waiting => "closed",
                    Egg::Hatching { .. } => "hatching",
                    Egg::Spent => "spent",
                }
                .to_owned(),
            ),
            hatch_age: match state.egg {
                Egg::Hatching { age } => Some(age),
                Egg::Absent | Egg::Waiting | Egg::Spent => None,
            },
            baby_phase: Some(baby_phase(state.baby.as_ref()).to_owned()),
            draw: Some(format!("{:?}", state.order)),
            egg: Some(picture_of(drawings, Figure::Shell, names)),
            baby: Some(picture_of(drawings, Figure::Baby, names)),
            x: state.baby.map(|baby| in_eighths(baby.place.x)),
            y: state.baby.map(|baby| in_eighths(baby.place.y)),
            vx: motion.map(|motion| speeds(motion).0),
            vy: motion.map(|motion| speeds(motion).1),
            hover: motion.map(|motion| match motion {
                BabyMotion::Hovering { left } => left,
                BabyMotion::Inside | BabyMotion::Rising { .. } | BabyMotion::Following { .. } => 0,
            }),
            cry: match motion {
                Some(BabyMotion::Following { cry, .. }) => Some(cry),
                _ => None,
            },
            baby_age: state.baby.map(|baby| match baby.pulse {
                Pulse::Unseen => -1,
                Pulse::Age(age) => i32::from(age),
            }),
            drops_shed: match motion {
                Some(BabyMotion::Rising { slime, .. }) => Some(slime == Slime::Shed),
                _ => None,
            },
            sound: Some(
                report
                    .sounds
                    .iter()
                    .map(|offer| names.sound_name(*offer))
                    .collect(),
            ),
            cue: Some(report.cue.is_some()),
            fragments: Some(fragments.collect()),
            drops: Some(drops.collect()),
        }
    }

    /// The supplied fields of `expected` that `observed` doesn't match.
    fn mismatches(expected: &Fields, observed: &Fields) -> Vec<String> {
        let mut found = Vec::new();
        macro_rules! compare {
        ($($field:ident),*) => {$(
            if expected.$field.is_some() && expected.$field != observed.$field {
                found.push(format!(
                    "{}: expected {:?}, got {:?}",
                    stringify!($field),
                    expected.$field,
                    observed.$field
                ));
            }
        )*};
    }
        compare!(
            phase, hatch_age, baby_phase, draw, egg, baby, x, y, vx, vy, hover, cry, baby_age,
            drops_shed, sound, cue, fragments, drops
        );
        found
    }

    fn inputs(inputs: &Inputs) -> (Pair<i32>, IntroRequest, Cut) {
        let centre = inputs.centre.unwrap_or(DEFAULT_CENTRE);
        let request = match (inputs.start == Some(true), inputs.finish == Some(true)) {
            (false, false) => IntroRequest::None,
            (true, false) => IntroRequest::StartDiscovery,
            (false, true) => IntroRequest::FinishDiscovery,
            (true, true) => panic!("a frame starts or finishes the discovery, not both"),
        };
        let cut = if inputs.finished == Some(false) {
            Cut::Unfinished
        } else {
            Cut::Finished
        };
        (
            Pair {
                x: centre[0],
                y: centre[1],
            },
            request,
            cut,
        )
    }

    fn quiet() -> Report {
        Report {
            sounds: Vec::new(),
            cue: None,
        }
    }

    /// A constructed vector: restore `before`, apply the inputs, compare the supplied `after`.
    fn run_constructed(vector: &Vector, encounter: &Encounter, names: &Names) -> Vec<String> {
        let mut state = restore(&vector.before, encounter.scene.home);
        let mut found = Vec::new();
        if let Some(egg) = &vector.before.egg {
            let shown = picture_of(&encounter.drawings(&state), Figure::Shell, names);
            if &shown != egg {
                found.push(format!(
                    "before egg: restored state shows {shown}, not {egg}"
                ));
            }
        }
        let (samus, request, cut) = inputs(&vector.inputs);
        let report = encounter.step(&mut state, samus, request, cut);
        let observed = observe(&state, &report, &encounter.drawings(&state), names);
        found.extend(mismatches(&vector.after, &observed));
        found
    }

    /// A recorded vector: its observation restored as a snapshot, carried through the saved form and
    /// read back. Its sound offers and cue are events of the recorded cut, not snapshot fields.
    fn run_recorded(vector: &Vector, encounter: &Encounter, names: &Names) -> Vec<String> {
        assert!(
            vector.row.is_some() && vector.cut.is_some(),
            "a recorded vector names its cut"
        );
        assert_eq!(
            vector.before,
            Fields::default(),
            "a recorded vector restores nothing"
        );
        let observation = Fields {
            baby_phase: vector
                .after
                .baby_phase
                .clone()
                .or_else(|| implied_baby_phase(&vector.after)),
            ..vector.after.clone()
        };
        let state = restore(&observation, encounter.scene.home);
        let saved = toml::Value::try_from(&state).unwrap();
        let reloaded: DiscoveryState = saved.try_into().unwrap();
        let mut found = Vec::new();
        if reloaded != state {
            found.push("the saved snapshot reloads differently".to_owned());
        }
        let observed = observe(&reloaded, &quiet(), &encounter.drawings(&reloaded), names);
        let snapshot = Fields {
            sound: None,
            cue: None,
            ..vector.after.clone()
        };
        found.extend(mismatches(&snapshot, &observed));
        found
    }

    /// The baby's phase a recorded observation implies without naming it: following where it
    /// observes a cry age or a horizontal speed, rising where it observes the droplets' release or
    /// a vertical speed, otherwise inside where it observes the baby at all. Only the snapshot
    /// check reads it; any phase carries the place and pulse it observes.
    fn implied_baby_phase(observed: &Fields) -> Option<String> {
        let phase = if observed.cry.is_some() || observed.vx.is_some() {
            "following"
        } else if observed.drops_shed.is_some() || observed.vy.is_some() {
            "rising"
        } else if observed.x.is_some() || observed.y.is_some() || observed.baby_age.is_some() {
            "inside"
        } else {
            return None;
        };
        Some(phase.to_owned())
    }

    fn load() -> (VectorFile, Encounter, Names) {
        let file: VectorFile = toml::from_str(VECTORS).unwrap();
        let mut names = Names::default();
        let encounter = Encounter::new(scene(&file.scene, &mut names));
        (file, encounter, names)
    }

    #[test]
    fn every_vector_passes() {
        let (file, encounter, names) = load();
        let mut failures = Vec::new();
        for vector in &file.vector {
            let found = match vector.source {
                Source::Constructed => run_constructed(vector, &encounter, &names),
                Source::Recorded => run_recorded(vector, &encounter, &names),
            };
            failures.extend(
                found
                    .into_iter()
                    .map(|line| format!("{}: {line}", vector.id)),
            );
        }
        assert!(
            failures.is_empty(),
            "failing vectors:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    fn every_vector_names_known_entries() {
        let (file, ..) = load();
        let known = entry_names();
        for vector in &file.vector {
            assert!(!vector.entries.is_empty(), "{} names no entry", vector.id);
            for entry in &vector.entries {
                assert!(
                    known.contains(entry),
                    "{} names unknown entry {entry}",
                    vector.id
                );
            }
        }
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let (file, ..) = load();
        for (id, behaviour) in catalogue() {
            assert_eq!(behaviour.id, id);
            assert_eq!(behaviour.checked, Checked::ByVectors);
            let name = format!("{id:?}");
            let constructed = file.vector.iter().any(|vector| {
                vector.source == Source::Constructed && vector.entries.contains(&name)
            });
            assert!(constructed, "{name} has no constructed vector");
        }
    }

    #[test]
    fn the_catalogue_matches_the_spec() {
        let spec = include_str!("../../../../docs/behaviours/screens-discovery.md");
        let headings: Vec<&str> = spec
            .lines()
            .filter_map(|line| line.strip_prefix("### "))
            .collect();
        assert_eq!(headings, entry_names());
    }

    /// The catalogue's entry ids, in its order.
    fn entry_names() -> Vec<String> {
        catalogue()
            .into_iter()
            .map(|(id, _): (BehaviourId, _)| format!("{id:?}"))
            .collect()
    }
}
