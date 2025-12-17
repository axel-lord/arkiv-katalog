//! [ThemeArg] impl.

use ::core::{borrow::Borrow, fmt::Display, str::FromStr};
use ::std::sync::OnceLock;

use ::clap::{ValueEnum, builder::PossibleValue};
use ::iced::{Theme, theme::Base};
use ::serde::{Deserialize, Serialize};

/// Theme wrapper implementing needed traits.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ThemeArg(&'static Theme);

impl ThemeArg {
    /// Check if argument is default value.
    pub const fn is_default(&self) -> bool {
        matches!(self.0, Theme::Dark)
    }

    /// Get the cyclic next variant.
    #[expect(
        clippy::missing_panics_doc,
        reason = "panic should not happen, due to cyclic iterator"
    )]
    pub fn next_cyclic(self) -> Self {
        Theme::ALL
            .iter()
            .cycle()
            .skip_while(|theme| theme != &self.0)
            .nth(1)
            .map(Self)
            .expect("cyclic iterator should always have values")
    }

    /// Get the cyclic prev variant.
    #[expect(
        clippy::missing_panics_doc,
        reason = "panic should not happen, due to cyclic iterator"
    )]
    pub fn prev_cyclic(self) -> Self {
        Theme::ALL
            .iter()
            .rev()
            .cycle()
            .skip_while(|theme| theme != &self.0)
            .nth(1)
            .map(Self)
            .expect("cyclic iterator should always have values")
    }
}

impl PartialEq for ThemeArg {
    fn eq(&self, other: &Self) -> bool {
        self.0.name() == other.0.name()
    }
}

impl Display for ThemeArg {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        Display::fmt(self.0.name(), f)
    }
}

impl Borrow<Theme> for ThemeArg {
    fn borrow(&self) -> &Theme {
        self.0
    }
}

impl Default for ThemeArg {
    fn default() -> Self {
        Self(&Theme::Dark)
    }
}

impl TryFrom<String> for ThemeArg {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<ThemeArg> for String {
    fn from(value: ThemeArg) -> Self {
        value.0.name().to_owned()
    }
}

impl FromStr for ThemeArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for theme in Theme::ALL {
            if theme.name().eq_ignore_ascii_case(s) {
                return Ok(Self(theme));
            }
        }
        Err(format!("could not get iced theme {s}"))
    }
}

impl From<ThemeArg> for Theme {
    fn from(value: ThemeArg) -> Self {
        Theme::clone(value.0)
    }
}

impl From<&'static Theme> for ThemeArg {
    fn from(value: &'static Theme) -> Self {
        Self(value)
    }
}

impl ValueEnum for ThemeArg {
    fn value_variants<'a>() -> &'a [Self] {
        static VARIANTS: OnceLock<Vec<ThemeArg>> = OnceLock::new();
        VARIANTS.get_or_init(|| Theme::ALL.iter().map(ThemeArg).collect())
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(PossibleValue::new(self.0.name()))
    }

    fn from_str(input: &str, ignore_case: bool) -> Result<Self, String> {
        for theme in Theme::ALL {
            if ignore_case {
                if theme.name().eq_ignore_ascii_case(input) {
                    return Ok(Self(theme));
                }
            } else if theme.name() == input {
                return Ok(Self(theme));
            }
        }
        Err(format!("could not get iced theme {input}"))
    }
}
