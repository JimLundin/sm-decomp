use super::{Kind, TraitImpl, Visibility, of};

/// An item as the test compares it: kind, name, visibility, owner and inline modules.
type ItemRow<'a> = (Kind, &'a str, Visibility, Option<&'a str>, Vec<&'a str>);

const SOURCE: &str = r#"//! The fixture: a small module. Its second sentence is left out.
//!
//! More docs.
#![expect(clippy::all, reason = "a fixture")]

mod inner;
pub mod open;

pub use inner::{Alpha, Beta as B, deep::*};
use crate::other::{self, Gamma};

/// A state.
#[derive(Clone)]
pub enum Mode {
    /// One.
    Still,
    #[doc = "two"]
    Moving { speed: u8 },
    Gone(u8),
}

pub(crate) struct Body(u8);

const LIMIT: u8 = Body { x: 1 }.x;

pub mod button {
    pub const RUN: u16 = 1;
}

impl<T: Clone> Pair<T> {
    /// Makes one.
    pub const fn new(
        x: T,
        y: T,
    ) -> Self {
        let z = other::helper::make(x);
        Self { x, y }
    }

    fn hidden(&self) {}
}

impl fmt::Display for Pair<u8> where u8: Copy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { Ok(()) }
}

macro_rules! rows { ($a:ident) => { $a::b::c } }

#[cfg(test)]
mod tests {
    pub fn not_listed() {}
}
"#;

#[test]
fn the_doc_is_its_first_sentence() {
    assert_eq!(
        of(SOURCE).doc.as_deref(),
        Some("The fixture: a small module.")
    );
    assert_eq!(of("fn a() {}\n").doc, None);
}

#[test]
fn items_carry_their_kind_visibility_and_owner() {
    let outline = of(SOURCE);
    let items: Vec<ItemRow<'_>> = outline
        .items
        .iter()
        .map(|item| {
            (
                item.kind,
                item.name,
                item.visibility,
                item.owner,
                item.module.clone(),
            )
        })
        .collect();
    assert_eq!(
        items,
        vec![
            (
                Kind::Module { inline: false },
                "inner",
                Visibility::Private,
                None,
                vec![]
            ),
            (
                Kind::Module { inline: false },
                "open",
                Visibility::Public,
                None,
                vec![]
            ),
            (
                Kind::Enum { variants: 3 },
                "Mode",
                Visibility::Public,
                None,
                vec![]
            ),
            (Kind::Struct, "Body", Visibility::Restricted, None, vec![]),
            (Kind::Constant, "LIMIT", Visibility::Private, None, vec![]),
            (
                Kind::Module { inline: true },
                "button",
                Visibility::Public,
                None,
                vec![]
            ),
            (
                Kind::Constant,
                "RUN",
                Visibility::Public,
                None,
                vec!["button"]
            ),
            (
                Kind::Function,
                "new",
                Visibility::Public,
                Some("Pair"),
                vec![]
            ),
            (
                Kind::Function,
                "hidden",
                Visibility::Private,
                Some("Pair"),
                vec![]
            ),
        ]
    );
    let new = outline.items.iter().find(|item| item.name == "new");
    assert_eq!(
        new.and_then(|item| item.signature.as_deref()),
        Some("new(x: T, y: T) -> Self")
    );
}

#[test]
fn trait_impls_are_listed_not_their_functions() {
    assert_eq!(
        of(SOURCE).impls,
        vec![TraitImpl {
            trait_name: "fmt::Display".to_owned(),
            self_type: "Pair<u8>".to_owned(),
        }]
    );
}

#[test]
fn use_trees_give_their_leaves() {
    let outline = of(SOURCE);
    let leaves: Vec<(Vec<&str>, Option<&str>, bool, Visibility)> = outline
        .uses
        .iter()
        .map(|leaf| (leaf.path.clone(), leaf.name(), leaf.glob, leaf.visibility))
        .collect();
    assert_eq!(
        leaves,
        vec![
            (
                vec!["inner", "Alpha"],
                Some("Alpha"),
                false,
                Visibility::Public
            ),
            (vec!["inner", "Beta"], Some("B"), false, Visibility::Public),
            (vec!["inner", "deep"], None, true, Visibility::Public),
            (
                vec!["crate", "other"],
                Some("other"),
                false,
                Visibility::Private
            ),
            (
                vec!["crate", "other", "Gamma"],
                Some("Gamma"),
                false,
                Visibility::Private
            ),
        ]
    );
}

#[test]
fn paths_outside_use_items_and_macros_are_read() {
    assert_eq!(
        of(SOURCE).paths,
        vec![
            vec!["fmt", "Display"],
            vec!["fmt", "Formatter"],
            vec!["fmt", "Result"],
            vec!["other", "helper", "make"],
        ]
    );
}
