//! Runs every vector of the screens-explosion vector file through the model, and checks the
//! catalogue against the spec's entries and the vectors.

/// The runner, inside a test module so that it may unwrap, panic and index as tests do.
#[cfg(test)]
mod runner {
    use core::fmt::Debug;
    use core::num;

    use enum_map::{EnumMap, enum_map};
    use screens_explosion::catalogue::{BehaviourId, behaviour};
    use screens_explosion::{
        BurstFrame, MapRegion, MusicDelay, MusicRequest, MusicService, Rules, SceneEvent,
        Scheduler, sprites, step,
    };
    use serde::Deserialize;
    use types::explosion::{
        ArtSet, AsteroidContent, AsteroidField, BackdropContent, BackdropFrame, Burst, BurstGroup,
        BurstId, BurstTrack, ExplosionScene, ExplosionStage, ExplosionState, FadeWait, HoldFrames,
        Light, LightLevel, MapDrawing, PictureAge, PictureColours, PlaneAngle, PlanePose,
        PlaneView, PlaneZoom, SampleAge, SampleLook, ScreenPoint, SkyContent, SpacePalette,
        SpriteDrawing, StationFlashStyle, Subpixels, TrackSample,
    };
    use types::{Checked, Pair};

    /// Subpixels in a pixel, for reading the file's fractional pixels.
    const SUBPIXELS_PER_PIXEL: f64 = 65_536.0;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        source: String,
        entries: Vec<String>,
        music_busy: Option<bool>,
        ready: Option<bool>,
        advance: Option<bool>,
        rules: Option<RulesRow>,
        scene: Option<SceneRow>,
        before: Snapshot,
        after: Snapshot,
        output: Output,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RulesRow {
        hold_frames: u16,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SceneRow {
        art: Option<String>,
        palette: Option<String>,
        flash_style: Option<String>,
        ship_front: Option<String>,
        empty_map: Option<String>,
    }

    /// A burst row: identity, animation update number, x, y, displayed sample number.
    type BurstRow = (String, i16, f64, f64, u8);

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Snapshot {
        stage: Option<String>,
        pose: Option<(u8, u16, f64, f64)>,
        level: Option<u8>,
        fade: Option<u8>,
        age: Option<u32>,
        sky: Option<(f64, f64, u8)>,
        bursts: Option<Vec<BurstRow>>,
        view: Option<String>,
        hold: Option<u16>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Output {
        setup: Option<String>,
        palette: Option<String>,
        music: Option<Vec<(String, u8)>>,
        flash: Option<FlashRow>,
        map_writes: Option<Vec<(String, String)>>,
        begin_zebes: Option<bool>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct FlashRow {
        style: String,
        center: (f64, f64),
    }

    impl Scheduler for Vector {
        fn frame_finished(&self) -> bool {
            self.advance.unwrap_or(true)
        }

        fn setup_complete(&self) -> bool {
            self.ready.unwrap_or(false)
        }
    }

    impl MusicService for Vector {
        fn music_busy(&self) -> bool {
            self.music_busy
                .unwrap_or_else(|| panic!("{}: music_busy is read but not set", self.id))
        }
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(&vector_file::read("screens-explosion").unwrap())
            .unwrap()
            .vector
    }

    fn subpixels(pixels: f64) -> Subpixels {
        let scaled = pixels * SUBPIXELS_PER_PIXEL;
        Subpixels(
            format!("{scaled}")
                .parse()
                .unwrap_or_else(|_| panic!("{pixels} px is not a whole number of subpixels")),
        )
    }

    fn point(x: f64, y: f64) -> ScreenPoint {
        Pair {
            x: subpixels(x),
            y: subpixels(y),
        }
    }

    fn nonzero(n: u8) -> num::NonZeroU8 {
        num::NonZeroU8::new(n).unwrap()
    }

    fn sprite(name: &str) -> SpriteDrawing {
        SpriteDrawing(name.to_owned())
    }

    /// A track of `pictures` drawings of `frames` frames each, then `blank` frames of nothing if
    /// any, played `passes` times. Drawings are named by their animation position, from 1.
    fn track(pictures: u8, frames: u8, blank: Option<u8>, passes: u8) -> BurstTrack {
        let drawings = (1..=pictures).map(|position| TrackSample {
            frames: nonzero(frames),
            look: SampleLook::Drawing(sprite(&position.to_string())),
        });
        let blanks = blank.map(|frames| TrackSample {
            frames: nonzero(frames),
            look: SampleLook::Blank,
        });
        BurstTrack {
            samples: drawings.chain(blanks).collect(),
            passes: nonzero(passes),
            palette: PictureColours("Bursts".to_owned()),
        }
    }

    /// The shipped scene, as the spec describes it, with the vector's overrides.
    fn scene_for(row: Option<&SceneRow>) -> ExplosionScene {
        let pick = |chosen: Option<&String>, shipped: &str| {
            chosen.map_or_else(|| shipped.to_owned(), Clone::clone)
        };
        ExplosionScene {
            art: ArtSet(pick(row.and_then(|r| r.art.as_ref()), "StationArt")),
            space_palette: SpacePalette(pick(row.and_then(|r| r.palette.as_ref()), "SpacePalette")),
            flash_style: StationFlashStyle(pick(
                row.and_then(|r| r.flash_style.as_ref()),
                "StationFlash",
            )),
            ship_front: MapDrawing(pick(row.and_then(|r| r.ship_front.as_ref()), "ShipFront")),
            empty_drawing: MapDrawing(pick(row.and_then(|r| r.empty_map.as_ref()), "Empty")),
            pivot: Pair { x: 52, y: 48 },
            sky: SkyContent {
                asteroids: enum_map! {
                    AsteroidField::Large => AsteroidContent {
                        start: Pair { x: 80, y: 159 },
                        speed: Subpixels::fraction(1, 4),
                        drawing: sprite("LargeAsteroids"),
                    },
                    AsteroidField::Small => AsteroidContent {
                        start: Pair { x: 128, y: 96 },
                        speed: Subpixels::fraction(1, 32),
                        drawing: sprite("SmallAsteroids"),
                    },
                },
                backdrop: BackdropContent {
                    place: Pair { x: 112, y: 87 },
                    drawings: enum_map! {
                        BackdropFrame::First => sprite("PurpleFirst"),
                        BackdropFrame::Second => sprite("PurpleSecond"),
                    },
                },
                palette: PictureColours("Sky".to_owned()),
            },
            tracks: enum_map! {
                BurstGroup::Final => track(6, 5, None, 1),
                BurstGroup::First => track(6, 3, None, 1),
                BurstGroup::Middle => track(6, 3, Some(16), 6),
                BurstGroup::Last => track(4, 5, Some(8), 7),
            },
        }
    }

    fn rules_for(row: Option<&RulesRow>) -> Rules {
        row.map_or(Rules::REFERENCE, |row| Rules {
            hold_frames: HoldFrames(num::NonZeroU16::new(row.hold_frames).unwrap()),
        })
    }

    fn stage_of(name: &str, hold: Option<u16>) -> ExplosionStage {
        match name {
            "Pending" => ExplosionStage::Pending,
            "WaitingForMusic" => ExplosionStage::WaitingForMusic,
            "Revealing" => ExplosionStage::Revealing,
            "Station" => ExplosionStage::Station,
            "Departing" => ExplosionStage::Departing,
            "Holding" => ExplosionStage::Holding(HoldFrames(
                num::NonZeroU16::new(hold.unwrap_or(0)).expect("a hold of 1 or more"),
            )),
            "Dimming" => ExplosionStage::Dimming,
            "Complete" => ExplosionStage::Complete,
            other => panic!("unknown stage {other}"),
        }
    }

    fn stage_name(stage: ExplosionStage) -> &'static str {
        match stage {
            ExplosionStage::Pending => "Pending",
            ExplosionStage::WaitingForMusic => "WaitingForMusic",
            ExplosionStage::Revealing => "Revealing",
            ExplosionStage::Station => "Station",
            ExplosionStage::Departing => "Departing",
            ExplosionStage::Holding(_) => "Holding",
            ExplosionStage::Dimming => "Dimming",
            ExplosionStage::Complete => "Complete",
        }
    }

    fn hold_of(stage: ExplosionStage) -> u16 {
        match stage {
            ExplosionStage::Holding(left) => left.0.get(),
            ExplosionStage::Pending
            | ExplosionStage::WaitingForMusic
            | ExplosionStage::Revealing
            | ExplosionStage::Station
            | ExplosionStage::Departing
            | ExplosionStage::Dimming
            | ExplosionStage::Complete => 0,
        }
    }

    fn light_of(level: u8) -> Light {
        LightLevel::new(level).map_or(Light::Blanked, Light::Shown)
    }

    fn level_of(light: Light) -> u8 {
        match light {
            Light::Blanked => 0,
            Light::Shown(level) => level.get(),
        }
    }

    fn fade_of(wait: u8) -> FadeWait {
        match wait {
            0 => FadeWait::NoFrame,
            1 => FadeWait::OneFrame,
            other => panic!("a fade wait of {other}"),
        }
    }

    fn view_of(name: &str) -> PlaneView {
        match name {
            "Station" => PlaneView::Station,
            "Gunship" => PlaneView::Gunship,
            other => panic!("unknown view {other}"),
        }
    }

    fn pose_of((angle, zoom, x, y): (u8, u16, f64, f64)) -> PlanePose {
        PlanePose {
            angle: PlaneAngle(angle),
            zoom: PlaneZoom(zoom),
            scroll: point(x, y),
        }
    }

    fn burst_id_of(name: &str) -> BurstId {
        let (group, occurrence) = name
            .split_once('-')
            .expect("a burst named group-occurrence");
        let group = match group {
            "final" => BurstGroup::Final,
            "first" => BurstGroup::First,
            "middle" => BurstGroup::Middle,
            "last" => BurstGroup::Last,
            other => panic!("unknown burst group {other}"),
        };
        BurstId {
            group,
            occurrence: occurrence.parse().unwrap(),
        }
    }

    fn burst_of(row: &BurstRow) -> Burst {
        Burst {
            id: burst_id_of(&row.0),
            age: SampleAge(row.1),
            position: point(row.2, row.3),
        }
    }

    fn state_of(snapshot: &Snapshot) -> ExplosionState {
        let mut state = ExplosionState::pending();
        state.stage = stage_of(
            snapshot.stage.as_deref().unwrap_or("Pending"),
            snapshot.hold,
        );
        state.light = light_of(snapshot.level.unwrap_or(0));
        state.fade = fade_of(snapshot.fade.unwrap_or(0));
        if let Some(pose) = snapshot.pose {
            state.pose = pose_of(pose);
        }
        if let Some(view) = &snapshot.view {
            state.view = view_of(view);
        }
        state.age = PictureAge(snapshot.age.unwrap_or(0));
        if let Some((large, small, _)) = snapshot.sky {
            state.asteroids = enum_map! {
                AsteroidField::Large => subpixels(large),
                AsteroidField::Small => subpixels(small),
            };
        }
        state.bursts = snapshot.bursts.iter().flatten().map(burst_of).collect();
        state
    }

    /// A burst as a vector row reads it: identity, update number, place, displayed sample number.
    type BurstReading = (BurstId, i16, ScreenPoint, usize);

    fn sample_number(frame: BurstFrame) -> usize {
        match frame {
            BurstFrame::Hidden => 0,
            BurstFrame::Showing(index) => index.saturating_add(1),
        }
    }

    fn backdrop_number(frame: BackdropFrame) -> u8 {
        match frame {
            BackdropFrame::First => 0,
            BackdropFrame::Second => 1,
        }
    }

    /// The sky and bursts of `state` as the vector rows read them.
    fn picture_readings(
        state: &ExplosionState,
        scene: &ExplosionScene,
    ) -> ((Subpixels, Subpixels, u8), Vec<BurstReading>) {
        let picture = sprites(state, scene);
        let ages: Vec<i16> = state.bursts.iter().map(|burst| burst.age.0).collect();
        let bursts = picture
            .bursts
            .iter()
            .zip(ages)
            .map(|(burst, age)| (burst.id, age, burst.position, sample_number(burst.frame)))
            .collect();
        let sky = (
            picture.large_asteroids.x,
            picture.small_asteroids.x,
            backdrop_number(picture.backdrop),
        );
        (sky, bursts)
    }

    fn expected_sky((large, small, frame): (f64, f64, u8)) -> (Subpixels, Subpixels, u8) {
        (subpixels(large), subpixels(small), frame)
    }

    fn expected_bursts(rows: &[BurstRow]) -> Vec<BurstReading> {
        rows.iter()
            .map(|row| {
                let burst = burst_of(row);
                (burst.id, burst.age.0, burst.position, usize::from(row.4))
            })
            .collect()
    }

    fn music_request(name: &str) -> MusicRequest {
        match name {
            "Stop" => MusicRequest::Stop,
            "Ceres" => MusicRequest::CeresMusicData,
            "ZebesFlight" => MusicRequest::ZebesFlightSong,
            other => panic!("unknown music request {other}"),
        }
    }

    fn map_region(name: &str) -> MapRegion {
        match name {
            "Upper" => MapRegion::Upper,
            "Lower" => MapRegion::Lower,
            other => panic!("unknown map region {other}"),
        }
    }

    /// The events an output table lists, in the order the scene asks them.
    fn expected_events(output: &Output) -> Vec<SceneEvent> {
        let setup = output
            .setup
            .iter()
            .map(|art| SceneEvent::InstallArt(ArtSet(art.clone())));
        let palette = output
            .palette
            .iter()
            .map(|palette| SceneEvent::ReplaceSpacePalette(SpacePalette(palette.clone())));
        let music =
            output
                .music
                .iter()
                .flatten()
                .map(|(request, delay)| SceneEvent::RequestMusic {
                    request: music_request(request),
                    delay: MusicDelay(*delay),
                });
        let flash = output
            .flash
            .iter()
            .map(|flash| SceneEvent::StartStationFlash {
                style: StationFlashStyle(flash.style.clone()),
                center: point(flash.center.0, flash.center.1),
            });
        let map_writes = output.map_writes.iter().flatten().map(|(drawing, region)| {
            SceneEvent::InstallMapDrawing {
                drawing: MapDrawing(drawing.clone()),
                region: map_region(region),
            }
        });
        let handoff = output
            .begin_zebes
            .filter(|begins| *begins)
            .map(|_| SceneEvent::BeginZebesFlight);
        setup
            .chain(palette)
            .chain(music)
            .chain(flash)
            .chain(map_writes)
            .chain(handoff)
            .collect()
    }

    /// Notes a mismatch of one field.
    fn check<T: PartialEq + Debug>(
        gaps: &mut Vec<String>,
        id: &str,
        field: &str,
        expected: &T,
        actual: &T,
    ) {
        if expected != actual {
            gaps.push(format!(
                "{id} {field}: expected {expected:?}, got {actual:?}"
            ));
        }
    }

    /// Checks the before table's derived columns (sky picture, displayed samples) against the model.
    fn check_before(
        gaps: &mut Vec<String>,
        vector: &Vector,
        state: &ExplosionState,
        scene: &ExplosionScene,
    ) {
        let (sky, bursts) = picture_readings(state, scene);
        if let Some(row) = vector.before.sky {
            check(gaps, &vector.id, "before sky", &expected_sky(row), &sky);
        }
        if let Some(rows) = &vector.before.bursts {
            check(
                gaps,
                &vector.id,
                "before bursts",
                &expected_bursts(rows),
                &bursts,
            );
        }
    }

    /// Checks every field the after table states.
    fn check_after(
        gaps: &mut Vec<String>,
        vector: &Vector,
        state: &ExplosionState,
        scene: &ExplosionScene,
    ) {
        let after = &vector.after;
        let id = vector.id.as_str();
        if let Some(stage) = &after.stage {
            check(gaps, id, "stage", &stage.as_str(), &stage_name(state.stage));
        }
        if let Some(pose) = after.pose {
            check(gaps, id, "pose", &pose_of(pose), &state.pose);
        }
        if let Some(level) = after.level {
            check(gaps, id, "level", &level, &level_of(state.light));
        }
        if let Some(fade) = after.fade {
            check(gaps, id, "fade", &fade_of(fade), &state.fade);
        }
        if let Some(age) = after.age {
            check(gaps, id, "age", &PictureAge(age), &state.age);
        }
        if let Some(view) = &after.view {
            check(gaps, id, "view", &view_of(view), &state.view);
        }
        if let Some(hold) = after.hold {
            check(gaps, id, "hold", &hold, &hold_of(state.stage));
        }
        let (sky, bursts) = picture_readings(state, scene);
        if let Some(row) = after.sky {
            check(gaps, id, "sky", &expected_sky(row), &sky);
        }
        if let Some(rows) = &after.bursts {
            check(gaps, id, "bursts", &expected_bursts(rows), &bursts);
        }
    }

    #[test]
    fn every_vector_passes() {
        let vectors = vectors();
        let mut gaps = Vec::new();
        for vector in &vectors {
            assert!(
                ["recorded", "constructed"].contains(&vector.source.as_str()),
                "{}: unknown source {}",
                vector.id,
                vector.source
            );
            let scene = scene_for(vector.scene.as_ref());
            let rules = rules_for(vector.rules.as_ref());
            let mut state = state_of(&vector.before);
            check_before(&mut gaps, vector, &state, &scene);
            let events = step(&mut state, vector, vector, &scene, rules);
            check_after(&mut gaps, vector, &state, &scene);
            check(
                &mut gaps,
                &vector.id,
                "output",
                &expected_events(&vector.output),
                &events,
            );
        }
        assert!(
            gaps.is_empty(),
            "{} mismatches over {} vectors:\n{}",
            gaps.len(),
            vectors.len(),
            gaps.join("\n")
        );
    }

    #[test]
    fn the_vector_file_holds_34_vectors() {
        assert_eq!(vectors().len(), 34);
    }

    fn every_behaviour() -> impl Iterator<Item = BehaviourId> {
        enum_map! { id => id }.into_values()
    }

    fn named(id: BehaviourId) -> String {
        format!("{id:?}")
    }

    #[test]
    fn every_vector_entry_names_a_behaviour() {
        let names: Vec<String> = every_behaviour().map(named).collect();
        for vector in vectors() {
            for entry in &vector.entries {
                assert!(
                    names.contains(entry),
                    "{}: unknown entry {entry}",
                    vector.id
                );
            }
        }
    }

    #[test]
    fn every_behaviour_checked_by_vectors_has_one() {
        let vectors = vectors();
        for id in every_behaviour() {
            assert_eq!(behaviour(id).checked, Checked::ByVectors);
            let exercised = vectors
                .iter()
                .any(|vector| vector.entries.contains(&named(id)));
            assert!(exercised, "no vector exercises {id:?}");
        }
    }

    #[test]
    fn the_catalogue_has_one_entry_per_spec_entry() {
        let catalogue: EnumMap<BehaviourId, BehaviourId> = enum_map! { id => behaviour(id).id };
        assert_eq!(catalogue.len(), 9, "the spec has nine entries");
        for (id, entry) in catalogue {
            assert_eq!(id, entry);
        }
    }
}
