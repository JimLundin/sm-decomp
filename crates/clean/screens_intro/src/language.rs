//! The language the options chose, which the caller checks at the intro's start.

/// The options' language setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    /// English: the pages this model plays.
    English,
    /// Japanese: other pages, and another end of page 5; not played here.
    Japanese,
}

/// A language whose pages this model doesn't play.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unsupported(pub Language);

/// Whether the intro plays in this language: English does; Japanese is `Unsupported`.
///
/// # Errors
///
/// `Unsupported` for a Japanese setting.
pub const fn plays(language: Language) -> Result<(), Unsupported> {
    match language {
        Language::English => Ok(()),
        Language::Japanese => Err(Unsupported(language)),
    }
}
