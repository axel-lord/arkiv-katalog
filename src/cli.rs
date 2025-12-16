//! [Cli] impl.

use ::clap::Parser;
use ::color_eyre::eyre::eyre;
use ::iced::daemon;

use crate::{Settings, State, theme_arg::ThemeArg};

/// Application to display a comic archive catalogue.
#[derive(Debug, Default, Clone, Parser)]
pub struct Cli {
    /// Theme to use for application.
    #[arg(long, short, value_enum)]
    pub theme: Option<ThemeArg>,

    /// Application name used when querying xdg directories.
    #[arg(long, short, default_value = "arkiv-katalog")]
    pub app_name: String,

    /// Profile to use, separates cache, config and data based on profile.
    #[arg(long, short, default_value = "default")]
    pub profile: String,
}

impl Cli {
    /// Run application.
    ///
    /// # Errors
    /// On application errors
    pub fn run(self) -> ::color_eyre::Result<()> {
        let xdg_dirs = ::xdg::BaseDirectories::with_profile(&self.app_name, &self.profile);
        let mut settings = xdg_dirs
            .find_config_file("config.toml")
            .map(::std::fs::read_to_string)
            .transpose()
            .map_err(|err| eyre!(err))?
            .map(|content| ::toml::from_str::<Settings>(&content))
            .transpose()
            .map_err(|err| eyre!(err))?
            .unwrap_or_default();
        if let Some(theme) = self.theme {
            settings.theme = theme;
        }
        daemon(
            State::init(self, settings, xdg_dirs),
            State::update,
            State::view,
        )
        .title(State::title)
        .theme(State::theme)
        .subscription(State::subscription)
        .run()
        .map_err(|err| eyre!(err))
    }
}
