//! Runs every vector of `scenarios/vectors/camera.toml` through the model, and checks that every
//! catalogue entry checked by vectors has one, from the code and the vector file alone. The checks
//! that read the spec are `tests/spec.rs`.
//!
//! The file is read with `toml` into the shape its header defines, declared once below. Every
//! table of it refuses a key the shape doesn't give, and every name in it (a source, a spec entry,
//! a screen's character, a facing, a reason for going backwards, a lead setting) is one the shape
//! knows, so anything else fails the read. The shape is then turned into the model's own types.

#[cfg(test)]
mod tests {
    use core::num::Wrapping;

    use clean_model_camera::{
        CATALOGUE, Camera, Frame, Heights, LeadSetting, Room, RoomError, Rules, Samus,
    };
    use clean_types::{Checked, Facing, Fixed, Pair, Parts, ScreenKind, fixed};
    use serde::Deserialize;

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/camera.toml");

    /// The file: its `[[vector]]` tables.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
    }

    /// One frame of the camera, from its state at the frame's start to its state at the end.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        source: Source,
        entries: Vec<Entry>,
        room: RoomShape,
        #[serde(default)]
        lead: Lead,
        #[serde(default)]
        time_stopped: bool,
        before: State,
        samus: SamusShape,
        after: State,
    }

    /// Where a vector comes from: every vector of the file is constructed.
    #[derive(Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum Source {
        Constructed,
    }

    /// A spec entry a vector exercises, by its name in the catalogue.
    #[derive(Deserialize)]
    #[serde(try_from = "String")]
    struct Entry(String);

    impl TryFrom<String> for Entry {
        type Error = String;

        fn try_from(name: String) -> Result<Self, String> {
            if CATALOGUE.iter().any(|b| format!("{:?}", b.id) == name) {
                Ok(Self(name))
            } else {
                Err(format!("a vector names {name}, no catalogue entry"))
            }
        }
    }

    /// The room: its rows of screens, the run below them, and its aim heights.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RoomShape {
        screens: Vec<Screens>,
        #[serde(default)]
        below: Option<Screens>,
        heights: [u16; 2],
    }

    /// A string of screens, one character a screen: "C" Closed, "O" Open, "S" Short.
    #[derive(Deserialize)]
    #[serde(try_from = "String")]
    struct Screens(Vec<ScreenKind>);

    impl TryFrom<String> for Screens {
        type Error = String;

        fn try_from(screens: String) -> Result<Self, String> {
            screens
                .chars()
                .map(|c| match c {
                    'C' => Ok(ScreenKind::Closed),
                    'O' => Ok(ScreenKind::Open),
                    'S' => Ok(ScreenKind::Short),
                    other => Err(format!("unknown screen {other}")),
                })
                .collect::<Result<_, _>>()
                .map(Self)
        }
    }

    /// The lead setting; `Ahead` where a vector gives none.
    #[derive(Deserialize, Default)]
    enum Lead {
        #[default]
        Ahead,
        KeepLeft,
        KeepRight,
        Near,
    }

    impl Lead {
        const fn setting(self) -> LeadSetting {
            match self {
                Self::Ahead => LeadSetting::Ahead,
                Self::KeepLeft => LeadSetting::KeepLeft,
                Self::KeepRight => LeadSetting::KeepRight,
                Self::Near => LeadSetting::Near,
            }
        }
    }

    /// The camera at the frame's start or end: its view's top-left corner, and where it last saw
    /// her.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct State {
        camera: [Ps; 2],
        seen: [Ps; 2],
    }

    impl State {
        fn camera(&self) -> Camera {
            Camera::new(pair(self.camera), pair(self.seen))
        }
    }

    /// What the camera reads of her this frame.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SamusShape {
        position: [Ps; 2],
        facing: FacingName,
        #[serde(default)]
        backwards: Option<Backwards>,
        #[serde(default)]
        rising: bool,
        #[serde(default)]
        placed: Option<Ps>,
    }

    impl SamusShape {
        fn samus(&self) -> Samus {
            Samus {
                position: pair(self.position),
                placed: self.placed.map(|p| p.0),
                facing: self.facing.facing(),
                backwards: self.backwards.is_some(),
                rising: self.rising,
            }
        }
    }

    /// Which way she faces.
    #[derive(Deserialize, Clone, Copy)]
    enum FacingName {
        Left,
        Right,
        Screen,
    }

    impl FacingName {
        const fn facing(self) -> Facing {
            match self {
                Self::Left => Facing::Left,
                Self::Right => Facing::Right,
                Self::Screen => Facing::Screen,
            }
        }
    }

    /// Why she is going backwards; each reads the same, as the spec's input is the bool.
    #[derive(Deserialize, Clone, Copy)]
    enum Backwards {
        Knockback,
        Moonwalk,
        Skid,
    }

    /// A `PPPP.SSSS` quantity: four hex digits of pixel part, four of subpixel part.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(try_from = "String")]
    struct Ps(Fixed);

    impl TryFrom<String> for Ps {
        type Error = String;

        fn try_from(s: String) -> Result<Self, String> {
            let part = |p: &str| {
                (p.len() == 4 && p.chars().all(|c| c.is_ascii_hexdigit()))
                    .then(|| u16::from_str_radix(p, 16).ok())
                    .flatten()
            };
            s.split_once('.')
                .and_then(|(p, sub)| Some(fixed(Wrapping(part(p)?), part(sub)?)))
                .map(Self)
                .ok_or_else(|| format!("not a P.S quantity: {s}"))
        }
    }

    /// `[x, y]` as a pair.
    const fn pair([x, y]: [Ps; 2]) -> Pair<Fixed> {
        Pair { x: x.0, y: y.0 }
    }

    impl RoomShape {
        /// The room: its rows of screens, each as wide as the first, then the run below them
        /// continuing them.
        fn room(self) -> Room {
            let width = self.screens.first().map_or(0, |row| row.0.len());
            assert!(
                self.screens.iter().all(|row| row.0.len() == width),
                "rows of one width"
            );
            let size = Pair {
                x: width,
                y: self.screens.len(),
            };
            let below = self.below.map(|b| b.0).unwrap_or_default();
            let run = self.screens.into_iter().flat_map(|row| row.0).chain(below);
            let [while_not_rising, while_rising] = self.heights;
            let heights = Heights {
                while_not_rising: Wrapping(while_not_rising),
                while_rising: Wrapping(while_rising),
            };
            Room::new(run.collect(), size, heights).expect("a valid room")
        }
    }

    /// The file's vectors; a key or a name the shape doesn't know fails the read.
    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(VECTORS)
            .unwrap_or_else(|e| panic!("the vector file doesn't read: {e}"))
            .vector
    }

    fn show(p: Pair<Fixed>) -> String {
        let part = |v: Fixed| format!("{:04X}.{:04X}", v.pixel().0, v.sub());
        format!("[{}, {}]", part(p.x), part(p.y))
    }

    /// Runs one vector: its id, and `Err` naming what differs.
    fn run(v: Vector) -> (String, Result<(), String>) {
        let Vector {
            id,
            source: Source::Constructed,
            entries: _,
            room,
            lead,
            time_stopped,
            before,
            samus,
            after,
        } = v;
        let room = room.room();
        let frame = Frame {
            samus: samus.samus(),
            room: &room,
            lead: lead.setting(),
            time_stopped,
        };
        let got = before.camera().step(&Rules::REFERENCE, &frame);
        let after = after.camera();
        let mut wrong = Vec::new();
        if got.position() != after.position() {
            wrong.push(format!(
                "camera: expected {}, got {}",
                show(after.position()),
                show(got.position())
            ));
        }
        if got.seen() != after.seen() {
            wrong.push(format!(
                "seen: expected {}, got {}",
                show(after.seen()),
                show(got.seen())
            ));
        }
        let result = if wrong.is_empty() {
            Ok(())
        } else {
            Err(wrong.join("; "))
        };
        (id, result)
    }

    #[test]
    fn every_vector_passes() {
        let results: Vec<_> = vectors().into_iter().map(run).collect();
        assert!(!results.is_empty(), "the file holds vectors");
        let failures: Vec<String> = results
            .iter()
            .filter_map(|(id, r)| r.as_ref().err().map(|e| format!("{id}: {e}")))
            .collect();
        assert!(
            failures.is_empty(),
            "{} of {} vectors fail:\n{}",
            failures.len(),
            results.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        // A name no catalogue entry has fails the read (`Entry`).
        let named: Vec<String> = vectors()
            .into_iter()
            .flat_map(|v| v.entries)
            .map(|entry| entry.0)
            .collect();
        for b in &CATALOGUE {
            let id = format!("{:?}", b.id);
            let has = named.contains(&id);
            match b.checked {
                Checked::ByVectors => assert!(has, "{id} is checked by vectors but has none"),
                Checked::OriginalOnly => assert!(!has, "{id} is checked on the original only"),
            }
        }
    }

    /// A vector the shape reads, to change one key or name of at a time.
    const ONE: &str = r#"
[[vector]]
id = "one"
source = "constructed"
entries = ["Settling"]
room = { screens = ["O"], heights = [104, 150] }
before = { camera = ["0000.0000", "0000.0000"], seen = ["0000.0000", "0000.0000"] }
samus = { position = ["0000.0000", "0000.0000"], facing = "Right" }
after = { camera = ["0000.0000", "0000.0000"], seen = ["0000.0000", "0000.0000"] }
"#;

    #[test]
    fn a_key_or_a_name_the_shape_doesnt_know_fails_the_read() {
        let reads = |text: &str| toml::from_str::<VectorFile>(text).is_ok();
        assert!(reads(ONE), "the vector reads as it stands");
        let changes = [
            // A key no table gives, in each table of the file.
            ("[[vector]]", "extra = 1\n[[vector]]"),
            ("id = \"one\"", "id = \"one\"\nextra = 1"),
            ("heights = [104, 150]", "heights = [104, 150], extra = 1"),
            ("before = { camera", "before = { extra = 1, camera"),
            ("facing = \"Right\"", "facing = \"Right\", extra = 1"),
            ("after = { camera", "after = { extra = 1, camera"),
            // A name of each kind the shape doesn't know.
            ("\"constructed\"", "\"recorded\""),
            ("[\"Settling\"]", "[\"Unknown\"]"),
            ("[\"O\"]", "[\"X\"]"),
            ("\"Right\"", "\"Up\""),
            (
                "facing = \"Right\"",
                "facing = \"Right\", backwards = \"Slide\"",
            ),
            ("source", "lead = \"Far\"\nsource"),
            ("\"0000.0000\"", "\"0.0\""),
        ];
        for (from, to) in changes {
            assert!(!reads(&ONE.replacen(from, to, 1)), "{to} reads");
        }
    }

    #[test]
    fn a_room_is_built_only_from_a_run_that_holds_it() {
        use clean_types::ScreenKind::Open;
        let heights = Heights {
            while_not_rising: Wrapping(112),
            while_rising: Wrapping(160),
        };
        let build =
            |run: Vec<ScreenKind>, x: usize, y: usize| Room::new(run, Pair { x, y }, heights).err();
        assert_eq!(build(vec![Open], 0, 1), Some(RoomError::NoScreens));
        assert_eq!(build(vec![Open], 1, 0), Some(RoomError::NoScreens));
        assert_eq!(build(vec![Open; 3], 2, 2), Some(RoomError::MissingScreens));
        assert_eq!(build(vec![Open; 257], 257, 1), Some(RoomError::TooLarge));
        assert_eq!(build(vec![Open; 257], 1, 257), Some(RoomError::TooLarge));
        assert_eq!(build(vec![Open; 257], 256, 1), None);
    }
}
