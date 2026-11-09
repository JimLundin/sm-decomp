//! Runs every vector of `scenarios/vectors/camera.toml` through the model, and checks that every
//! catalogue entry checked by vectors has one, from the code and the vector file alone. A key the
//! format doesn't give fails the test. The checks that read the spec are `tests/spec.rs`.

#[cfg(test)]
mod tests {
    use core::iter::Peekable;
    use core::num::Wrapping;
    use core::str::Chars;

    use clean_model_camera::{
        CATALOGUE, Camera, Frame, Heights, LeadSetting, Room, RoomError, Rules, Samus, ScreenKind,
    };
    use clean_types::{Checked, Facing, Fixed, Pair, Parts, fixed};

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/camera.toml");

    /// A TOML value, as far as the vector file uses them.
    #[derive(Debug)]
    enum Value {
        Str(String),
        Int(u16),
        Bool(bool),
        Array(Vec<Self>),
        Table(Table),
    }

    /// A table whose keys are taken as they are read; `done` fails on any left.
    #[derive(Debug, Default)]
    struct Table(Vec<(String, Value)>);

    impl Table {
        fn take(&mut self, key: &str) -> Option<Value> {
            let at = self.0.iter().position(|(k, _)| k == key)?;
            Some(self.0.remove(at).1)
        }

        fn need(&mut self, key: &str) -> Value {
            self.take(key)
                .unwrap_or_else(|| panic!("missing key {key}"))
        }

        fn done(self, what: &str) {
            let keys: Vec<&String> = self.0.iter().map(|(k, _)| k).collect();
            assert!(keys.is_empty(), "unknown keys in {what}: {keys:?}");
        }
    }

    impl Value {
        fn str(self) -> String {
            let Self::Str(s) = self else {
                panic!("not a string: {self:?}")
            };
            s
        }

        fn int(self) -> u16 {
            let Self::Int(n) = self else {
                panic!("not an integer: {self:?}")
            };
            n
        }

        fn bool(self) -> bool {
            let Self::Bool(b) = self else {
                panic!("not a bool: {self:?}")
            };
            b
        }

        fn array(self) -> Vec<Self> {
            let Self::Array(a) = self else {
                panic!("not an array: {self:?}")
            };
            a
        }

        fn table(self) -> Table {
            let Self::Table(t) = self else {
                panic!("not a table: {self:?}")
            };
            t
        }

        /// A `PPPP.SSSS` quantity.
        fn fixed(self) -> Fixed {
            let s = self.str();
            let (p, sub) = s.split_once('.').expect("P.S");
            fixed(
                Wrapping(u16::from_str_radix(p, 16).expect("pixel part")),
                u16::from_str_radix(sub, 16).expect("subpixel part"),
            )
        }

        /// `[x, y]` of `P.S` quantities.
        fn pair(self) -> Pair<Fixed> {
            let mut a = self.array().into_iter();
            let x = a.next().expect("x").fixed();
            let y = a.next().expect("y").fixed();
            assert!(a.next().is_none(), "a pair has two parts");
            Pair { x, y }
        }

        /// A string of screens, one character a screen.
        fn screens(self) -> Vec<ScreenKind> {
            self.str().chars().map(screen).collect()
        }
    }

    fn screen(c: char) -> ScreenKind {
        match c {
            'C' => ScreenKind::Closed,
            'O' => ScreenKind::Open,
            'S' => ScreenKind::Short,
            other => panic!("unknown screen {other}"),
        }
    }

    /// A reader of one line's `key = value`.
    struct Reader<'a>(Peekable<Chars<'a>>);

    impl Reader<'_> {
        fn skip(&mut self) {
            while self.0.next_if(|c| c.is_whitespace()).is_some() {}
        }

        fn word(&mut self) -> String {
            self.skip();
            let mut w = String::new();
            while let Some(c) = self.0.next_if(|c| c.is_alphanumeric() || *c == '_') {
                w.push(c);
            }
            w
        }

        fn pair(&mut self) -> (String, Value) {
            let key = self.word();
            self.skip();
            assert_eq!(self.0.next(), Some('='), "key {key} wants a value");
            (key, self.value())
        }

        fn value(&mut self) -> Value {
            self.skip();
            match self.0.peek() {
                Some('"') => {
                    self.0.next();
                    let s: String = self.0.by_ref().take_while(|c| *c != '"').collect();
                    assert!(!s.contains('\\'), "no escapes expected");
                    Value::Str(s)
                }
                Some('[') => Value::Array(self.list(']', Self::value)),
                Some('{') => Value::Table(Table(self.list('}', Self::pair))),
                _ => match self.word().as_str() {
                    "true" => Value::Bool(true),
                    "false" => Value::Bool(false),
                    n => Value::Int(n.parse().unwrap_or_else(|_| panic!("bad value {n}"))),
                },
            }
        }

        fn list<T>(&mut self, close: char, item: fn(&mut Self) -> T) -> Vec<T> {
            self.0.next();
            let mut items = Vec::new();
            loop {
                self.skip();
                if self.0.next_if_eq(&close).is_some() {
                    return items;
                }
                items.push(item(self));
                self.skip();
                self.0.next_if_eq(&',');
            }
        }
    }

    /// The file's `[[vector]]` tables.
    fn vectors() -> Vec<Table> {
        let mut tables: Vec<Table> = Vec::new();
        for line in VECTORS.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line == "[[vector]]" {
                tables.push(Table::default());
                continue;
            }
            let mut reader = Reader(line.chars().peekable());
            let pair = reader.pair();
            reader.skip();
            assert!(reader.0.next().is_none(), "trailing text on {line}");
            tables
                .last_mut()
                .expect("a key before any vector")
                .0
                .push(pair);
        }
        tables
    }

    fn facing(name: &str) -> Facing {
        match name {
            "Left" => Facing::Left,
            "Right" => Facing::Right,
            "Screen" => Facing::Screen,
            other => panic!("unknown facing {other}"),
        }
    }

    fn lead(name: &str) -> LeadSetting {
        match name {
            "Ahead" => LeadSetting::Ahead,
            "KeepLeft" => LeadSetting::KeepLeft,
            "KeepRight" => LeadSetting::KeepRight,
            "Near" => LeadSetting::Near,
            other => panic!("unknown lead {other}"),
        }
    }

    fn backwards(why: &str) -> bool {
        assert!(
            matches!(why, "Knockback" | "Moonwalk" | "Skid"),
            "unknown backwards {why}"
        );
        true
    }

    /// A camera as `before` or `after` gives it: its view, and where it last saw her.
    fn camera(value: Value, what: &str) -> Camera {
        let mut t = value.table();
        let position = t.need("camera").pair();
        let seen = t.need("seen").pair();
        t.done(what);
        Camera::new(position, seen)
    }

    /// A room as the vector gives it: its rows, each as wide as the first, then the run below
    /// them continuing them.
    fn room(value: Value) -> Room {
        let mut t = value.table();
        let rows: Vec<Vec<ScreenKind>> = t
            .need("screens")
            .array()
            .into_iter()
            .map(Value::screens)
            .collect();
        let width = rows.first().map_or(0, Vec::len);
        assert!(rows.iter().all(|r| r.len() == width), "rows of one width");
        let height = rows.len();
        let mut run: Vec<ScreenKind> = rows.into_iter().flatten().collect();
        run.extend(t.take("below").map_or_else(Vec::new, Value::screens));
        let mut h = t.need("heights").array().into_iter();
        let heights = Heights {
            level: Wrapping(h.next().expect("hn").int()),
            rising: Wrapping(h.next().expect("hr").int()),
        };
        assert!(h.next().is_none(), "two heights");
        t.done("room");
        Room::new(run, width, height, heights).expect("a valid room")
    }

    fn samus(value: Value, seen: Pair<Fixed>) -> Samus {
        let mut t = value.table();
        let s = Samus {
            position: t.need("position").pair(),
            camera_from: t.take("camera_from").map_or(seen.y, Value::fixed),
            facing: facing(&t.need("facing").str()),
            backwards: t.take("backwards").is_some_and(|v| backwards(&v.str())),
            rising: t.take("rising").is_some_and(Value::bool),
        };
        t.done("samus");
        s
    }

    fn show(p: Pair<Fixed>) -> String {
        let part = |v: Fixed| format!("{:04X}.{:04X}", v.pixel().0, v.sub());
        format!("[{}, {}]", part(p.x), part(p.y))
    }

    /// Runs one vector; `Err` names what differs.
    fn run(mut v: Table) -> (String, Vec<String>, Result<(), String>) {
        let id = v.need("id").str();
        v.need("source").str();
        let entries = v
            .need("entries")
            .array()
            .into_iter()
            .map(Value::str)
            .collect();
        let room = room(v.need("room"));
        let lead = v
            .take("lead")
            .map_or(LeadSetting::Ahead, |l| lead(&l.str()));
        let time_stopped = v.take("time_stopped").is_some_and(Value::bool);
        let before = camera(v.need("before"), "before");
        let samus = samus(v.need("samus"), before.seen());
        let after = camera(v.need("after"), "after");
        v.done(&id);
        let frame = Frame {
            samus,
            room: &room,
            lead,
            time_stopped,
        };
        let got = before.step(&Rules::REFERENCE, &frame);
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
        (id, entries, result)
    }

    #[test]
    fn every_vector_passes() {
        let results: Vec<_> = vectors().into_iter().map(run).collect();
        assert!(!results.is_empty(), "the file holds vectors");
        let failures: Vec<String> = results
            .iter()
            .filter_map(|(id, _, r)| r.as_ref().err().map(|e| format!("{id}: {e}")))
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
        let named: Vec<String> = vectors().into_iter().flat_map(|v| run(v).1).collect();
        let ids: Vec<String> = CATALOGUE.iter().map(|b| format!("{:?}", b.id)).collect();
        for name in &named {
            assert!(
                ids.contains(name),
                "a vector names {name}, no catalogue entry"
            );
        }
        for b in &CATALOGUE {
            let id = format!("{:?}", b.id);
            let has = named.contains(&id);
            match b.checked {
                Checked::ByVectors => assert!(has, "{id} is checked by vectors but has none"),
                Checked::OriginalOnly => assert!(!has, "{id} is checked on the original only"),
            }
        }
    }

    #[test]
    fn a_room_is_built_only_from_a_run_that_holds_it() {
        use ScreenKind::Open;
        let heights = Heights {
            level: Wrapping(112),
            rising: Wrapping(160),
        };
        let build = |run: Vec<ScreenKind>, width: usize, height: usize| {
            Room::new(run, width, height, heights).err()
        };
        assert_eq!(build(vec![Open], 0, 1), Some(RoomError::NoScreens));
        assert_eq!(build(vec![Open], 1, 0), Some(RoomError::NoScreens));
        assert_eq!(build(vec![Open; 3], 2, 2), Some(RoomError::MissingScreens));
        assert_eq!(build(vec![Open; 257], 257, 1), Some(RoomError::TooLarge));
        assert_eq!(build(vec![Open; 257], 1, 257), Some(RoomError::TooLarge));
        assert_eq!(build(vec![Open; 257], 256, 1), None);
    }
}
