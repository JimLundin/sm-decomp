use super::{Score, Size, functions, of, sizes};

fn tokens(src: &str) -> usize {
    let size = of(src);
    size.logic
        .saturating_add(size.types)
        .saturating_add(size.data)
}

/// The score in half tokens, of code that nothing nests or repeats.
fn halves(src: &str) -> usize {
    let size = of(src);
    assert_eq!(
        (size.depth, size.duplication),
        (Score(0), Score(0)),
        "{src}"
    );
    size.score.eighths() / 4
}

/// The part of the score depth adds, in eighths.
fn depth(src: &str) -> usize {
    of(src).depth.eighths()
}

/// The part of the score clumps add to each source measured together, in eighths.
fn clumps(srcs: &[&str]) -> Vec<usize> {
    sizes(srcs).iter().map(|s| s.clumps.eighths()).collect()
}

#[test]
fn layout_and_comments_cost_nothing() {
    // fn f () a : u8 -> u8 {} a + 1: twelve, each bracket pair once.
    let tight = "fn f(a: u8) -> u8 { a + 1 }";
    let spread =
        "/// Adds one.\nfn f(\n    a: u8, // the input\n) -> u8 {\n    a + 1 /* sum */\n}\n";
    assert_eq!(tokens(tight), 12);
    // The same, plus the trailing comma, which weighs nothing.
    assert_eq!(tokens(spread), 13);
    assert_eq!(halves(spread), halves(tight));
}

#[test]
fn literals_lifetimes_and_operators_are_one_each() {
    assert_eq!(tokens(r##"r#"a "b" c"#"##), 1);
    assert_eq!(tokens(r#"b"x\"y""#), 1);
    assert_eq!(tokens("b'x'"), 1);
    assert_eq!(tokens("'a'"), 1);
    assert_eq!(tokens("'\\n'"), 1);
    assert_eq!(tokens("'\\''"), 1);
    assert_eq!(tokens("&'a str"), 3);
    assert_eq!(tokens("0..=0x1F"), 3);
    assert_eq!(tokens("1.5e-3"), 1);
    assert_eq!(tokens("x.0.max(1)"), 7);
    assert_eq!(tokens("a::b"), 3);
    assert_eq!(tokens("r#type"), 1);
    assert_eq!(tokens("/* a /* nested */ b */ x"), 1);
}

#[test]
fn test_items_are_not_counted() {
    let src = "fn a() {}\n#[cfg(test)]\nmod tests { fn b() { c(); } }\n#[cfg(test)]\nuse x::y;\nfn d() {}";
    assert_eq!(tokens(src), tokens("fn a() {} fn d() {}"));
}

#[test]
fn a_constants_value_is_data_and_its_head_a_type() {
    // const ROWS : [] u8 ; 2 = and the ; are types; [] 1 , 2 is data; pub is logic, weighing
    // nothing.
    let table = "pub const ROWS: [u8; 2] = [1, 2];";
    assert_eq!(
        of(table),
        Size {
            logic: 1,
            types: 9,
            data: 4,
            ..Size::default()
        }
    );
    assert_eq!(
        of("static mut N: u8 = 3;"),
        Size {
            types: 7,
            data: 1,
            ..Size::default()
        }
    );
    // A trait's constant without a value is a declaration too.
    assert_eq!(of("trait T { const X: u8; }").types, 5);
}

#[test]
fn declarations_and_attributes_are_types() {
    // #[derive(Debug)] is 5 (# [ derive ( Debug), struct S { a : u8 } is 6; pub fn f ( { is logic.
    let src = "#[derive(Debug)]\npub struct S { a: u8 }\nfn f() {}";
    assert_eq!(
        of(src),
        Size {
            // pub 0, then fn f ( at half each, then { 1: 2.5 tokens.
            score: Score(20),
            logic: 5,
            types: 11,
            ..Size::default()
        }
    );
    assert_eq!(of("fn f() -> S { S { a: 1 } }").types, 0);
}

#[test]
fn const_fns_generics_and_locals_are_logic() {
    assert_eq!(of("const fn f() {}").data, 0);
    assert_eq!(of("fn f<const N: usize>() {}").data, 0);
    assert_eq!(of("fn f() { let a = 1; }").data, 0);
    // A constant inside a function body is still a constant's value.
    assert_eq!(of("fn f() { const A: u8 = 1; }").data, 1);
}

#[test]
fn imports_and_module_declarations_weigh_nothing() {
    let src = "use a::{b, c};\nmod m;\nmod n { fn f() {} }";
    // f's body brace alone weighs one; fn f ( weigh half each.
    assert_eq!(of(src).logic, 18);
    assert_eq!(halves(src), halves("fn f() {}"));
    // A `use` inside a body is an import too.
    assert_eq!(halves("{ use E::A; }"), 2);
}

#[test]
fn a_const_fns_const_weighs_nothing() {
    assert_eq!(halves("const fn f() {}"), halves("fn f() {}"));
    assert_eq!(of("const fn f() {}").logic, 5);
}

#[test]
fn a_signature_weighs_half() {
    // fn f ( a : u8 -> u8 is eight tokens at half, the body's { a + 1 four at one.
    assert_eq!(halves("fn f(a: u8) -> u8 { a + 1 }"), 8 + 2 * 4);
    // Up to a declaration's `;`, with generics and bounds; the `;` is the item's, at one.
    assert_eq!(halves("fn g<T: Into<u8>>(t: T) where T: Copy;"), 17 + 2);
    // A function-pointer type is not a signature.
    assert_eq!(halves("let p: fn(u8) = q;"), 2 * 9);
}

#[test]
fn true_and_false_weigh_double_in_logic() {
    assert_eq!(halves("x = true;"), 2 + 2 + 4 + 2);
    assert_eq!(halves("x = y;"), 2 * 4);
    // In a constant's value they are data.
    assert_eq!(halves("const A: bool = true;"), 0);
    assert_eq!(of("const A: bool = true;").data, 1);
}

#[test]
fn a_table_of_constructors_stays_data() {
    let rows = "const ROWS: &[Row] = &[Row { needs: &[Held(Key::Jump), Not(&Airborne)], \
                aim: AIM_UP_ELSE_DOWN, at: Some(Fixed(3)) }, Row { needs: &[], ..BASE }];";
    let size = of(rows);
    assert_eq!((size.logic, size.score), (0, Score(0)));
    assert!(size.data > 0);
}

#[test]
fn a_per_situation_column_of_enum_values_stays_data() {
    // What happens at the ground, stated once per situation (Jim, 2026-11-07, decision 4).
    let rows = "const SITUATIONS: &[Situation] = &[\
                Situation { doing: Doing::Turning, ledge: Ledge::Sinks, gait: Gait::Walk }, \
                Situation { doing: Doing::Shaping, ledge: Stays, ..FALLING }];";
    let size = of(rows);
    assert_eq!((size.logic, size.score), (0, Score(0)));
    assert!(size.data > 0);
    // The same decision written as a method on the row is logic.
    let method = "const SITUATIONS: &[Situation] = &[Situation { ledge: Doing::Turning.ledge() }];";
    assert!(of(method).logic > 0);
}

#[test]
fn a_constant_built_from_data_is_data() {
    // Each form a value that is data may take (root's L3 ruling, 2026-11-09).
    for value in [
        "3",
        "-3",
        "\"text\"",
        "true",
        "COLOURS_3",
        "Flip::None",
        "::m::A",
        "Self::A",
        "&A",
        "(1, B)",
        "()",
        "[1, 2,]",
        "[0; 16]",
        "[Picture::new(0x009); N]",
        "S { a: 1, b, ..BASE }",
        "S(1, -2)",
        "S {}",
        "Some(Fixed(3))",
        "Picture::new(0x009)",
        "Count::new(99)",
        "Tile::new(Picture::new(0x00F), COLOURS_3, Flip::None)",
        "[GridPos::at(1, 1), GridPos::at(1, 2)]",
        "build()",
        "Divisor::new(100)",
        "0..16",
        "0x280..=0x500",
        "-2..Self::MAX",
        "S { window: A..B }",
    ] {
        let size = of(&format!("const A: T = {value};"));
        assert_eq!((size.logic, size.score), (0, Score(0)), "{value}");
        assert!(size.data > 0, "{value}");
    }
}

#[test]
fn a_constant_holding_behaviour_is_logic() {
    for value in [
        "if A { 1 } else { 2 }",
        "match A { B if C => 1, _ => 2 }",
        "loop { break 3 }",
        "unsafe { A }",
        "{ let a = 3; a }",
        "{ A }",
        "|a: u8| a",
        "&[(1, |a| a)]",
        "move || 0",
        "Divisor::new(100).expect(\"not zero\")",
        "Count::new(99).unwrap()",
        "A.wrapping_add(B)",
        "Picture::new(0x009).0",
        "TABLE[2]",
        "size_of::<u8>()",
        "f(A.get(2))",
        "[f(1), g(2)?]",
        "concat!(1)",
        "S { a: 3 + B }",
        "A | B << 4",
        "-A",
        "!A",
        "3 as u16",
        "0..A + 1",
        "0..f(1)",
        "Picture::new(0x009)..B",
        "A.0..3",
        "..3",
        "Tile::new(Picture::new(0x00F), COLOURS_3, Flip::None).flipped()",
    ] {
        let size = of(&format!("const A: T = {value};"));
        assert_eq!(size.data, 0, "{value}");
        assert!(size.logic > 0, "{value}");
        // The head is still a declaration.
        assert_eq!(size.types, 6, "{value}");
    }
}

#[test]
fn a_score_prints_its_fraction() {
    assert_eq!(format!("{:>6}", Score(12)), "   1.5");
    assert_eq!(Score(16).to_string(), "2");
    assert_eq!(Score(1).to_string(), "0.125");
    assert_eq!(Score(14).to_string(), "1.75");
}

#[test]
fn a_nested_token_weighs_a_quarter_more_per_level() {
    // b ; inside the if's block, at depth 1: two tokens, each a quarter of one more.
    assert_eq!(depth("fn f() { if a { b; } }"), 2 * 2);
    // The if b { at depth 1, c at depth 2.
    assert_eq!(depth("fn f() { if a { if b { c } } }"), 3 * 2 + 2 * 2);
    // Each branch of an else-if chain is one level, not a level per else.
    assert_eq!(
        depth("fn f() { if a { b } else if c { d } else { e } }"),
        3 * 2
    );
    // The score holds the depth: four halves at depth 0 and two at depth 1.
    let size = of("fn f() { if a { b } }");
    assert_eq!(size.score, Score(11 * 4 + 2 * 5));
}

#[test]
fn loops_let_else_and_closures_nest() {
    assert_eq!(depth("fn f() { for x in y { z } }"), 2);
    assert_eq!(depth("fn f() { while a { b } }"), 2);
    assert_eq!(depth("fn f() { loop { b } }"), 2);
    // return ; in the let-else's block.
    assert_eq!(depth("fn f() { let Some(a) = b else { return; }; }"), 2 * 2);
    // An expression closure's body (x + 1) nests; its parameters don't.
    assert_eq!(depth("fn f() { g(|x| x + 1); }"), 3 * 2);
    assert_eq!(depth("fn f() { g(|x| { x }); }"), 2);
    assert_eq!(depth("fn f() { g(move || x, y); }"), 2);
}

#[test]
fn a_flat_match_does_not_nest() {
    assert_eq!(depth("fn f() { match a { B => c, D => e } }"), 0);
    // A guard's if is part of the arm's pattern.
    assert_eq!(depth("fn f() { match a { B if c => d, E => g } }"), 0);
    // An arm written as a block nests.
    assert_eq!(depth("fn f() { match a { B => { c } D => e } }"), 2);
    // Outside a function body nothing nests.
    assert_eq!(depth("const A: T = if a { b } else { c };"), 0);
}

#[test]
fn depth_composes_with_the_weights() {
    // true weighs double, so its quarter is of two.
    assert_eq!(depth("fn f() { if a { true } }"), 4);
}

#[test]
fn a_type_pair_in_three_signatures_loses_the_half() {
    let three = "fn f(a: &A, b: B) {} fn g(a: A, b: &mut B, c: C) {} fn h(b: B, a: A) {}";
    // f: a : & A , b : B are 8 tokens; g: a : A , b : & mut B , 10 (C is in no clump);
    // h: b : B , a : A 7. Each loses half a token: 4 eighths.
    assert_eq!(clumps(&[three]), [4 * (8 + 10 + 7)]);
    // In two signatures it is no clump.
    assert_eq!(clumps(&["fn f(a: A, b: B) {} fn g(a: A, b: B) {}"]), [0]);
    // bool is a flag, gated apart, and pairs with nothing.
    assert_eq!(
        clumps(&["fn f(a: bool, b: B) {} fn g(a: bool, b: B) {} fn h(a: bool, b: B) {}"]),
        [0]
    );
    // A primitive pairs like any type, and self is no parameter.
    let primitive = "fn f(&self, p: u16, h: Heading) {}";
    assert_eq!(clumps(&[primitive, primitive, primitive]), [4 * 7; 3]);
}

#[test]
fn an_argument_pair_at_three_sites_pays_again() {
    // The third site pays the shorter of a and b.x again: one token, eight eighths.
    assert_eq!(
        clumps(&["fn f() { g(a, b.x); h(b.x, a, c); g(a, b.x); }"]),
        [8]
    );
    // A fourth pays once more, here in another source.
    assert_eq!(
        clumps(&[
            "fn f() { g(a, b.x); g(a, b.x); g(a, b.x); }",
            "fn k() { m(a, b.x); }"
        ]),
        [8, 8]
    );
    // Calls and literals vary from site to site.
    assert_eq!(
        clumps(&["fn f() { g(a(), b); g(a(), b); g(a(), b); }"]),
        [0]
    );
    assert_eq!(clumps(&["fn f() { g(1, b); g(1, b); g(1, b); }"]), [0]);
    // A function's own parameters are not a call.
    assert_eq!(
        clumps(&["fn f() { fn g(a, b) {} fn h(a, b) {} fn k(a, b) {} }"]),
        [0]
    );
}

#[test]
fn a_struct_literals_fields_are_argument_pairs() {
    let src = "struct S { x: u8, y: u8 }\n\
               fn f() { g(S { x: a, y }); g(S { x: a, y }); let s = S { y, x: a, ..BASE }; }\n\
               fn h(s: S) { let S { x, y } = s; match s { S { x, y } => x } }";
    // The third literal pays a (or y) again; a destructuring pattern builds nothing.
    assert_eq!(clumps(&[src]), [8]);
}

#[test]
fn a_sources_functions_and_the_rest_add_up_to_its_size() {
    let srcs = [
        "use a::b;\nimpl S {\n    pub(crate) const fn f(x: u8) -> u8 { if x > 0 { g(a, b) } else { 0 } }\n    \
         fn h() { fn inner() {} g(a, b); }\n}\nfn k() { g(a, b); }\n",
        "fn m() { g(a, b); }",
    ];
    let measured = functions(&srcs);
    let names: Vec<Vec<&str>> = measured
        .iter()
        .map(|source| source.iter().map(|function| function.name).collect())
        .collect();
    // A function inside another's body is part of it.
    assert_eq!(names, [vec!["f", "h", "k"], vec!["m"]]);
    // The visibility and the qualifiers are the function's.
    assert!(measured[0][0].text.starts_with("pub(crate) const fn f"));
    assert!(measured[0][1].text.ends_with("g(a, b); }"));
    let whole = sizes(&srcs);
    for (source, size) in measured.iter().zip(&whole) {
        let parts = source
            .iter()
            .fold(Size::default(), |sum, function| sum.add(function.size));
        assert!(parts.score <= size.score);
        assert!(parts.logic <= size.logic);
    }
    // `impl S { }` and the `use`, outside every function: both weigh nothing.
    let rest = |at: usize| {
        whole[at].score.eighths()
            - measured[at]
                .iter()
                .map(|function| function.size.score.eighths())
                .sum::<usize>()
    };
    assert_eq!(rest(0), 0);
    assert_eq!(rest(1), 0);
    // The pair `(a, b)` recurs at a third site in `k` and a fourth in `m`: each pays there.
    assert_eq!(measured[0][2].size.clumps.eighths(), 8);
    assert_eq!(measured[1][0].size.clumps.eighths(), 8);
}

#[test]
fn visibility_weighs_nothing() {
    assert_eq!(halves("pub fn f() {}"), halves("fn f() {}"));
    assert_eq!(halves("pub(crate) fn f() {}"), halves("fn f() {}"));
    assert_eq!(halves("pub(in crate::m) const A: u8 = 1;"), 0);
    // Its tokens are still logic: pub ( crate, the bracket pair once.
    assert_eq!(of("pub(crate) fn f() {}").logic, of("fn f() {}").logic + 3);
}

#[test]
fn an_impl_header_weighs_nothing() {
    let header = "impl<const N: usize, T: Copy> Grid<T, N> where T: Default { fn f() {} }";
    assert_eq!(halves(header), halves("fn f() {}"));
    assert_eq!(
        halves("impl Trait for S { fn f() {} }"),
        halves("fn f() {}")
    );
    // An `impl` in a parameter's or a return's type is the signature's, at half.
    assert_eq!(
        halves("fn f(w: &mut impl World) {}"),
        halves("fn f(w: &mut W) {}") + 1
    );
    assert_eq!(
        halves("fn f() -> impl World {}"),
        halves("fn f() -> W {}") + 1
    );
}

#[test]
fn an_exhaustive_match_is_data() {
    // The arms are data; the match, its scrutinee and its braces stay logic.
    let lookup = "fn f(k: K) -> u16 { match k { K::A => 3, K::B | K::C => LIMIT, \
                  K::D => Some(Fixed::new(2)), } }";
    let empty = "fn f(k: K) -> u16 { match k {} }";
    assert_eq!(halves(lookup), halves(empty));
    assert!(of(lookup).data > 0);
    // Literals, a range of two and `_` are keys too.
    let literals = "fn f(n: u8) -> K { match n { 0 => K::A, 2..=4 | 7 => K::B, _ => K::C } }";
    assert_eq!(halves(literals), halves("fn f(n: u8) -> K { match n {} }"));
    // A variant nested in a tuple variant, with no binding, is a key.
    let nested = "fn f(s: S) -> u16 { match s { S::A(K::B) => 3, S::A(K::C | K::D, _) => 4,                   S::E(T::F(0)) => 5, S::G => 6 } }";
    assert_eq!(halves(nested), halves("fn f(s: S) -> u16 { match s {} }"));
    // In a constant's value, the match is logic and its arms data.
    let constant = of("const A: u16 = match B { C => 1, D => 2 };");
    assert!(constant.logic > 0 && constant.data > 0);
}

#[test]
fn enum_map_rows_are_data() {
    // The rows are data; the macro's name and brackets stay logic.
    let rows = "fn f() -> M { enum_map! { K::A => 3, K::B | K::C => S { x: 1, ..BASE } } }";
    assert_eq!(halves(rows), halves("fn f() -> M { enum_map! {} }"));
    // A row that reads a local or computes is a decision.
    for row in ["K::A => a", "K::A => 3 + B", "K::A => B.min(C)"] {
        let src = format!("fn f() -> M {{ enum_map! {{ {row}, K::B => 0 }} }}");
        assert_eq!(of(&src).data, 0, "{row}");
    }
}

#[test]
fn a_match_that_decides_stays_logic() {
    for arms in [
        "K::A if b => 3, _ => 4",
        "K::A => { 3 } K::B => 4",
        "K::A => a, K::B => 4",
        "K::A => self.x, K::B => 4",
        "K::A => B.get(), K::B => 4",
        "K::A => B + 1, K::B => 4",
        "K::A => S { x }, K::B => S { x: 4 }",
        "Some(x) => 3, None => 4",
        "S::A(K::B(x)) => 3, S::C => 4",
        "S::A(K::B, n) => 3, S::C => 4",
        "S::A(K::B) + 1 => 3, S::C => 4",
        "x => 3, _ => 4",
        "K::A => 3",
    ] {
        let src = format!("fn f(k: K) -> u16 {{ match k {{ {arms} }} }}");
        assert_eq!(of(&src).data, 0, "{arms}");
    }
}

#[test]
fn a_repeated_run_pays_again_whatever_its_locals_are_named() {
    let body = "let d = a.x.wrapping_add(b.y); if d > c.max { c.max = d; }";
    let renamed = "let e = p.x.wrapping_add(q.y); if e > r.max { r.max = e; }";
    let twice = format!("fn f() {{ {body} }} fn g() {{ {body} }}");
    let copy = format!("fn f() {{ {body} }} fn g() {{ {renamed} }}");
    let duplication = |src: &str| of(src).duplication;
    assert!(duplication(&twice) > Score(0));
    assert_eq!(duplication(&copy), duplication(&twice));
    let once = of(&format!("fn f() {{ {body} }}"));
    assert_eq!(once.duplication, Score(0));
    // The score holds the repeat's part: two copies cost twice one, and the part again.
    assert_eq!(
        of(&twice).score,
        Score(once.score.0 * 2).add(duplication(&twice))
    );
    // A field or a method renamed makes another run.
    let other = "let d = a.z.wrapping_sub(b.w); if d > c.min { c.min = d; }";
    let different = format!("fn f() {{ {body} }} fn g() {{ {other} }}");
    assert_eq!(duplication(&different), Score(0));
    // Written once in a step both call, the run costs less than its copy.
    let shared = format!(
        "fn h(a: A, b: B, c: &mut C) {{ {body} }} fn f() {{ h(a, b, c); }} fn g() {{ h(p, q, r); }}"
    );
    assert!(of(&shared).score < of(&copy).score);
}

#[test]
fn clumps_are_reported_but_weigh_nothing() {
    let clumped = of("fn f(a: A, b: B) {} fn g(a: A, b: B) {} fn h(a: A, b: B) {}");
    let apart = of("fn f(a: A, b: C) {} fn g(a: A, b: D) {} fn h(a: A, b: E) {}");
    assert!(clumped.clumps > Score(0));
    assert_eq!(apart.clumps, Score(0));
    assert_eq!(clumped.score, apart.score);
}
