//! Runs every vector of `scenarios/vectors/screens-zebes.toml` through the model. Each vector
//! is one finished frame from its `before` snapshot; a key the file doesn't define fails.

#[cfg(test)]
mod vectors {
    use std::collections::BTreeMap;

    use enum_map::EnumMap;
    use screens_zebes::{
        BehaviourId, ColourEffectRequest, FrameAdvance, FrameReport, Rules, StartingPalette,
        behaviour,
    };
    use serde::Deserialize;
    use types::Checked;
    use types::zebes::{
        Brightness, CaptionAge, CaptionContent, CaptionHolds, ColourEffectRef, DrawingPages,
        FadeWait, FlightView, HoldFrames, LitLevel, MosaicWidth, PageRefs, PaletteRef,
        PictureMotion, PixelPoint, PlacedPicture, PlacedPictures, PlaneAngle, PlanePose, PlaneZoom,
        PlanetPictures, QuarterBeat, ScenePicture, SpriteGraphicsRef, SpritePictureRef, StarField,
        SubpixelPoint, Subpixels, SubtitleChoice, TileGraphicsRef, TileMapRef, UpwardSpeed,
        ZebesScene, ZebesStage, ZebesState,
    };

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/screens-zebes.toml");

    /// Every content name the vectors use, and the invented ones this test adds, ending with a
    /// spare label no content uses; a name's id is its place here.
    const NAMES: [&str; 23] = [
        "Grid",
        "FlatTiles",
        "Scout",
        "ShipTiles",
        "NightSky",
        "AmberPulse",
        "Planet",
        "UpperLeft",
        "UpperRight",
        "LowerLeft",
        "LowerRight",
        "English",
        "Japanese",
        "PlanetPalette",
        "UpperLeftPalette",
        "UpperRightPalette",
        "LowerLeftPalette",
        "LowerRightPalette",
        "EnglishPalette",
        "JapanesePalette",
        "CaptionBrightening",
        "CaptionDimming",
        "Spare",
    ];

    /// The sample's star fields in compositing order, by the sample's labels.
    const STAR_LABELS: [(&str, StarField); 4] = [
        ("UpperLeft", StarField::First),
        ("UpperRight", StarField::Second),
        ("LowerLeft", StarField::Third),
        ("LowerRight", StarField::Fourth),
    ];

    /// An invented ship picture origin: the sample scene gives none.
    const SHIP_ORIGIN: PixelPoint = PixelPoint { x: 64, y: 48 };

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        scene: SampleScene,
        vector: Vec<Vector>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SampleScene {
        places: BTreeMap<String, [i16; 2]>,
        last_field: String,
        engine_accent: String,
        graphics: PageNames,
        holds: [u16; 4],
        english: [i16; 2],
        japanese: [i16; 2],
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct PageNames {
        flat_map: String,
        flat_tiles: String,
        ship_map: String,
        ship_tiles: String,
        sprite_graphics: String,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        source: String,
        entries: Vec<String>,
        before: Snapshot,
        after: Snapshot,
        output: Option<Output>,
        subtitle: Option<bool>,
        rules: Option<VectorRules>,
        last_field: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorRules {
        hold_frames: u16,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Snapshot {
        stage: String,
        view: Option<String>,
        level: Option<u8>,
        block: Option<u8>,
        beat: Option<u8>,
        fade: Option<u8>,
        age: Option<u16>,
        hold: Option<u16>,
        pose: Option<[Number; 4]>,
        pictures: Option<BTreeMap<String, [Number; 3]>>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Output {
        events: Option<Vec<String>>,
        install: Option<PageNames>,
        palette_replace: Option<PaletteReplace>,
        restore_full_energy: Option<bool>,
        begin_landing_site_load: Option<bool>,
        matrix: Option<[i32; 4]>,
        scroll: Option<[i16; 2]>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct PaletteReplace {
        effect: String,
        retain_colours: bool,
    }

    /// A number the file writes as an integer or as a decimal.
    #[derive(Clone, Copy, Deserialize)]
    #[serde(untagged)]
    enum Number {
        Whole(i64),
        Decimal(f64),
    }

    impl Number {
        /// The value in 65,536ths; every decimal in the file is an exact multiple.
        fn in_65536ths(self) -> i32 {
            let scaled = match self {
                Self::Whole(whole) => whole.checked_mul(65_536).unwrap(),
                Self::Decimal(decimal) => format!("{}", decimal * 65_536.0)
                    .parse()
                    .expect("a position is a whole number of 65,536ths"),
            };
            i32::try_from(scaled).expect("a position fits")
        }

        fn whole(self) -> i64 {
            match self {
                Self::Whole(whole) => whole,
                Self::Decimal(decimal) => panic!("expected a whole number, got {decimal}"),
            }
        }
    }

    fn content_id(name: &str) -> u16 {
        let place = NAMES
            .iter()
            .position(|known| *known == name)
            .unwrap_or_else(|| panic!("unknown content name {name}"));
        u16::try_from(place).unwrap()
    }

    fn label(name: &str) -> ScenePicture {
        match name {
            "Planet" => ScenePicture::Planet,
            "English" => ScenePicture::English,
            "Japanese" => ScenePicture::Japanese,
            star => ScenePicture::Stars(star_field(star)),
        }
    }

    fn star_field(name: &str) -> StarField {
        let (_, field) = STAR_LABELS
            .iter()
            .find(|(known, _)| *known == name)
            .unwrap_or_else(|| panic!("unknown picture label {name}"));
        *field
    }

    const fn point([x, y]: [i16; 2]) -> PixelPoint {
        PixelPoint { x, y }
    }

    fn placed(name: &str, anchor: [i16; 2]) -> PlacedPicture {
        PlacedPicture {
            picture: SpritePictureRef(content_id(name)),
            palette: PaletteRef(content_id(&format!("{name}Palette"))),
            anchor: point(anchor),
        }
    }

    fn hold(frames: u16) -> std::num::NonZeroU16 {
        std::num::NonZeroU16::new(frames).expect("a caption hold is positive")
    }

    fn pages(graphics: &PageNames) -> DrawingPages {
        DrawingPages {
            flat: PageRefs {
                map: TileMapRef(content_id(&graphics.flat_map)),
                graphics: TileGraphicsRef(content_id(&graphics.flat_tiles)),
            },
            ship: PageRefs {
                map: TileMapRef(content_id(&graphics.ship_map)),
                graphics: TileGraphicsRef(content_id(&graphics.ship_tiles)),
            },
            ship_origin: SHIP_ORIGIN,
            sprite_graphics: SpriteGraphicsRef(content_id(&graphics.sprite_graphics)),
        }
    }

    fn scene(sample: &SampleScene, last_field: Option<&str>) -> ZebesScene {
        let star_fields = EnumMap::from_fn(|field| {
            let (name, _) = STAR_LABELS.iter().find(|(_, f)| *f == field).unwrap();
            placed(name, sample.places[*name])
        });
        let [hidden_lead, english_lead, full_caption, fading_caption] = sample.holds.map(hold);
        ZebesScene {
            pages: pages(&sample.graphics),
            planet: PlanetPictures {
                planet: placed("Planet", sample.places["Planet"]),
                star_fields,
                last_field: star_field(last_field.unwrap_or(&sample.last_field)),
            },
            caption: CaptionContent {
                english: placed("English", sample.english),
                japanese: Some(placed("Japanese", sample.japanese)),
                holds: CaptionHolds {
                    hidden_lead,
                    english_lead,
                    full_caption,
                    fading_caption,
                },
                brightening: ColourEffectRef(content_id("CaptionBrightening")),
                dimming: ColourEffectRef(content_id("CaptionDimming")),
            },
            engine_accent: ColourEffectRef(content_id(&sample.engine_accent)),
        }
    }

    fn stage(snapshot: &Snapshot) -> ZebesStage {
        match snapshot.stage.as_str() {
            "Pending" => ZebesStage::Pending,
            "Revealing" => ZebesStage::Revealing {
                fade: if snapshot.fade == Some(1) {
                    FadeWait::OneFrame
                } else {
                    FadeWait::Elapsed
                },
                beat: QuarterBeat::new(snapshot.beat.unwrap_or(0)).unwrap(),
            },
            "Caption" => ZebesStage::Caption {
                age: CaptionAge(snapshot.age.unwrap_or(0)),
            },
            "Rightward" => ZebesStage::Rightward,
            "Banking" => ZebesStage::Banking,
            "Leftward" => ZebesStage::Leftward,
            "Still" => ZebesStage::Still {
                hold: HoldFrames::new(snapshot.hold.unwrap()).unwrap(),
            },
            "Sliding" => ZebesStage::Sliding,
            "Leaving" => ZebesStage::Leaving,
            "Complete" => ZebesStage::Complete,
            other => panic!("unknown stage {other}"),
        }
    }

    fn view(name: &str) -> FlightView {
        match name {
            "MosaicPage" => FlightView::MosaicPage,
            "ShipAndSprites" => FlightView::ShipAndSprites,
            "SpritesOnly" => FlightView::SpritesOnly,
            other => panic!("unknown view {other}"),
        }
    }

    fn light(level: u8) -> Brightness {
        LitLevel::new(level).map_or(Brightness::Blanked, Brightness::Shown)
    }

    fn pose([angle, zoom, x, y]: [Number; 4]) -> PlanePose {
        PlanePose {
            angle: PlaneAngle(u8::try_from(angle.whole()).unwrap()),
            zoom: PlaneZoom(u16::try_from(zoom.whole()).unwrap()),
            scroll: SubpixelPoint {
                x: Subpixels::from_65536ths(x.in_65536ths()),
                y: Subpixels::from_65536ths(y.in_65536ths()),
            },
        }
    }

    fn pictures(listed: &BTreeMap<String, [Number; 3]>) -> PlacedPictures {
        let mut pictures = PlacedPictures::default();
        for (name, [x, y, speed]) in listed {
            pictures[label(name)] = Some(PictureMotion {
                position: SubpixelPoint {
                    x: Subpixels::from_65536ths(x.in_65536ths()),
                    y: Subpixels::from_65536ths(y.in_65536ths()),
                },
                speed: UpwardSpeed(u16::try_from(speed.whole()).unwrap()),
            });
        }
        pictures
    }

    const fn zero_pose() -> [Number; 4] {
        [Number::Whole(0); 4]
    }

    fn state(before: &Snapshot) -> ZebesState {
        ZebesState {
            stage: stage(before),
            view: view(before.view.as_deref().unwrap_or("MosaicPage")),
            light: light(before.level.unwrap_or(0)),
            mosaic: MosaicWidth::new(before.block.unwrap_or(1)).unwrap(),
            pose: pose(before.pose.unwrap_or_else(zero_pose)),
            pictures: before.pictures.as_ref().map(pictures).unwrap_or_default(),
        }
    }

    /// The stage's output fields: its name, then its beat, fade, age and hold, each 0 for a
    /// stage that doesn't carry it.
    fn stage_fields(stage: ZebesStage) -> (String, u8, u8, u16, u16) {
        let name = format!("{stage:?}");
        let name = name.split([' ', '{']).next().unwrap().to_owned();
        match stage {
            ZebesStage::Revealing { fade, beat } => {
                (name, beat.get(), u8::from(fade == FadeWait::OneFrame), 0, 0)
            }
            ZebesStage::Caption { age } => (name, 0, 0, age.0, 0),
            ZebesStage::Still { hold } => (name, 0, 0, 0, hold.get()),
            ZebesStage::Pending
            | ZebesStage::Rightward
            | ZebesStage::Banking
            | ZebesStage::Leftward
            | ZebesStage::Sliding
            | ZebesStage::Leaving
            | ZebesStage::Complete => (name, 0, 0, 0, 0),
        }
    }

    fn check_state(id: &str, after: &Snapshot, actual: &ZebesState) {
        let (stage_name, beat, fade, age, hold) = stage_fields(actual.stage);
        assert_eq!(stage_name, after.stage, "{id}: stage");
        let counters = [
            (after.beat.map(u16::from), u16::from(beat), "beat"),
            (after.fade.map(u16::from), u16::from(fade), "fade"),
            (after.age, age, "age"),
            (after.hold, hold, "hold"),
        ];
        for (expected, got, field) in counters {
            if let Some(expected) = expected {
                assert_eq!(got, expected, "{id}: {field}");
            }
        }
        if let Some(name) = &after.view {
            assert_eq!(actual.view, view(name), "{id}: view");
        }
        if let Some(level) = after.level {
            assert_eq!(actual.light, light(level), "{id}: level");
        }
        if let Some(block) = after.block {
            assert_eq!(actual.mosaic.get(), block, "{id}: block");
        }
        if let Some(expected) = after.pose {
            assert_eq!(actual.pose, pose(expected), "{id}: pose");
        }
        if let Some(expected) = &after.pictures {
            assert_eq!(actual.pictures, pictures(expected), "{id}: pictures");
        }
    }

    fn caption_events(report: &FrameReport, scene: &ZebesScene) -> Vec<String> {
        match report.requests.colour_effect {
            Some(ColourEffectRequest::CaptionBrightens(effect)) => {
                assert_eq!(effect, scene.caption.brightening);
                vec!["CaptionBrightens".to_owned()]
            }
            Some(ColourEffectRequest::CaptionDims(effect)) => {
                assert_eq!(effect, scene.caption.dimming);
                vec!["CaptionDims".to_owned()]
            }
            Some(ColourEffectRequest::ReplaceActive { .. }) | None => Vec::new(),
        }
    }

    fn check_requests(id: &str, output: &Output, report: &FrameReport, scene: &ZebesScene) {
        if let Some(events) = &output.events {
            assert_eq!(&caption_events(report, scene), events, "{id}: events");
        }
        if let Some(install) = &output.install {
            let mut expected = pages(install);
            expected.ship_origin = scene.pages.ship_origin;
            assert_eq!(report.requests.pages, Some(expected), "{id}: install");
        }
        if let Some(replace) = &output.palette_replace {
            assert!(
                replace.retain_colours,
                "{id}: only carried colours are modelled"
            );
            let expected = ColourEffectRequest::ReplaceActive {
                accent: ColourEffectRef(content_id(&replace.effect)),
                starting_palette: StartingPalette::ColoursOnScreen,
            };
            assert_eq!(
                report.requests.colour_effect,
                Some(expected),
                "{id}: palette"
            );
        }
        let handoff = report.requests.handoff.is_some();
        for (expected, field) in [
            (output.restore_full_energy, "restore_full_energy"),
            (output.begin_landing_site_load, "begin_landing_site_load"),
        ] {
            if let Some(expected) = expected {
                assert_eq!(handoff, expected, "{id}: {field}");
            }
        }
    }

    fn check_plane(id: &str, output: &Output, report: &FrameReport, scene: &ZebesScene) {
        let plane = report.picture.plane;
        assert_eq!(plane.origin, scene.pages.ship_origin, "{id}: origin");
        if let Some(matrix) = output.matrix {
            let m = plane.matrix;
            assert_eq!([m.a, m.b, m.c, m.d], matrix, "{id}: matrix");
        }
        if let Some(scroll) = output.scroll {
            assert_eq!(plane.scroll, point(scroll), "{id}: scroll");
        }
    }

    /// The published sprites are the placed pictures, in compositing order, with their content.
    fn check_sprites(id: &str, report: &FrameReport, state: &ZebesState, scene: &ZebesScene) {
        let expected: Vec<_> = state
            .pictures
            .iter()
            .filter_map(|(which, motion)| Some((scene.picture(which)?, (*motion)?)))
            .map(|(content, motion)| {
                (
                    content.picture,
                    content.palette,
                    point([motion.position.x.pixel, motion.position.y.pixel]),
                )
            })
            .collect();
        let drawn: Vec<_> = report
            .picture
            .sprites
            .iter()
            .map(|sprite| (sprite.picture, sprite.palette, sprite.anchor))
            .collect();
        assert_eq!(drawn, expected, "{id}: sprites");
    }

    fn behaviour_id(name: &str) -> BehaviourId {
        BehaviourId::ALL
            .into_iter()
            .find(|id| format!("{id:?}") == name)
            .unwrap_or_else(|| panic!("unknown entry {name}"))
    }

    fn load() -> VectorFile {
        toml::from_str(VECTORS).expect("the vector file parses")
    }

    fn run(sample: &SampleScene, vector: &Vector) {
        let scene = scene(sample, vector.last_field.as_deref());
        let rules = vector
            .rules
            .as_ref()
            .map_or(Rules::REFERENCE, |rules| Rules {
                hold_frames: HoldFrames::new(rules.hold_frames).unwrap(),
            });
        let subtitles = if vector.subtitle == Some(true) {
            SubtitleChoice::WithJapanese
        } else {
            SubtitleChoice::EnglishOnly
        };
        let mut state = state(&vector.before);
        let report =
            screens_zebes::step(&mut state, &scene, rules, subtitles, FrameAdvance::Finished);
        check_state(&vector.id, &vector.after, &state);
        check_sprites(&vector.id, &report, &state, &scene);
        if let Some(output) = &vector.output {
            check_requests(&vector.id, output, &report, &scene);
            check_plane(&vector.id, output, &report, &scene);
        }
    }

    #[test]
    fn every_vector_passes() {
        let file = load();
        assert!(!file.vector.is_empty());
        for vector in &file.vector {
            assert!(
                ["recorded", "constructed"].contains(&vector.source.as_str()),
                "{}: source",
                vector.id
            );
            run(&file.scene, vector);
        }
    }

    #[test]
    fn every_entry_checked_by_vectors_has_a_vector() {
        let file = load();
        let exercised: Vec<BehaviourId> = file
            .vector
            .iter()
            .flat_map(|vector| vector.entries.iter().map(|name| behaviour_id(name)))
            .collect();
        for id in BehaviourId::ALL {
            if behaviour(id).checked == Checked::ByVectors {
                assert!(exercised.contains(&id), "{id:?} has no vector");
            }
        }
    }

    #[test]
    fn the_catalogue_has_one_behaviour_per_entry() {
        for (place, id) in BehaviourId::ALL.into_iter().enumerate() {
            assert_eq!(behaviour(id).id, id);
            assert_ne!(behaviour(id).summary, "");
            assert!(!BehaviourId::ALL[..place].contains(&id), "{id:?} repeated");
        }
    }

    // ---------------------------------------------------------------------------------------------
    // Whole flights through the sample scene.
    // ---------------------------------------------------------------------------------------------

    /// A whole flight from the setup frame to two frames past the hand-off: each frame's stage
    /// name and report.
    fn fly(subtitles: SubtitleChoice, rules: Rules) -> (Vec<String>, Vec<FrameReport>) {
        let file = load();
        let scene = scene(&file.scene, None);
        let mut state = ZebesState::pending();
        let mut stages = Vec::new();
        let mut reports = Vec::new();
        for _ in 0..20_000 {
            let report =
                screens_zebes::step(&mut state, &scene, rules, subtitles, FrameAdvance::Finished);
            stages.push(stage_fields(state.stage).0);
            reports.push(report);
            if stages.iter().rev().take(3).all(|name| name == "Complete") {
                break;
            }
        }
        (stages, reports)
    }

    fn seen_in_order(stages: &[String]) -> Vec<String> {
        let mut seen: Vec<String> = Vec::new();
        for name in stages {
            if seen.last() != Some(name) {
                seen.push(name.clone());
            }
        }
        seen
    }

    #[test]
    fn a_flight_is_seen_in_order_with_one_handoff_on_its_last_cut() {
        let (stages, reports) = fly(SubtitleChoice::EnglishOnly, Rules::REFERENCE);
        let order = [
            "Revealing",
            "Caption",
            "Rightward",
            "Banking",
            "Leftward",
            "Still",
            "Sliding",
            "Leaving",
            "Complete",
        ];
        assert_eq!(seen_in_order(&stages), order);
        let handoffs: Vec<usize> = reports
            .iter()
            .enumerate()
            .filter(|(_, report)| report.requests.handoff.is_some())
            .map(|(frame, _)| frame)
            .collect();
        let first_complete = stages.iter().position(|name| name == "Complete").unwrap();
        assert_eq!(handoffs, [first_complete]);
        let last = &reports[first_complete];
        assert_eq!(last.picture.light, Brightness::Blanked);
        assert_eq!(last.picture.sprites.len(), 0);
        assert_eq!(last.picture.view, FlightView::SpritesOnly);
    }

    #[test]
    fn the_subtitle_shows_only_when_chosen() {
        let japanese = |subtitles| {
            let (_, reports) = fly(subtitles, Rules::REFERENCE);
            let file = load();
            let scene = scene(&file.scene, None);
            let subtitle = scene.caption.japanese.unwrap().picture;
            reports
                .iter()
                .filter(|report| report.picture.sprites.iter().any(|s| s.picture == subtitle))
                .count()
        };
        let file = load();
        let [_, _, full_caption, _] = file.scene.holds;
        assert_eq!(
            japanese(SubtitleChoice::WithJapanese),
            usize::from(full_caption)
        );
        assert_eq!(japanese(SubtitleChoice::EnglishOnly), 0);
    }

    #[test]
    fn the_hold_lasts_the_adjusted_frames() {
        for frames in [1, 3, 64, 200] {
            let rules = Rules {
                hold_frames: HoldFrames::new(frames).unwrap(),
            };
            let (stages, _) = fly(SubtitleChoice::EnglishOnly, rules);
            let still = stages.iter().filter(|name| *name == "Still").count();
            assert_eq!(still, usize::from(frames), "hold of {frames}");
        }
    }

    #[test]
    fn a_borrowed_lag_cut_holds_the_picture_and_sends_nothing() {
        let file = load();
        let scene = scene(&file.scene, None);
        let mut state = ZebesState::pending();
        let subtitles = SubtitleChoice::WithJapanese;
        for _ in 0..400 {
            let published = screens_zebes::step(
                &mut state,
                &scene,
                Rules::REFERENCE,
                subtitles,
                FrameAdvance::Finished,
            );
            let before = state;
            let lag = screens_zebes::step(
                &mut state,
                &scene,
                Rules::REFERENCE,
                subtitles,
                FrameAdvance::BorrowedLag,
            );
            assert_eq!(state, before);
            assert_eq!(lag.picture, published.picture);
            assert_eq!(lag.requests, screens_zebes::Requests::NONE);
        }
    }
}
