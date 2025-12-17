#![doc = include_str!("../README.md")]

use ::std::io::Write;

use ::clap::ValueEnum;
use ::color_eyre::{Report, Section, eyre::eyre};
use ::derive_more::IsVariant;
use ::hashbrown::HashMap;
use ::iced::{
    Alignment::{self, Center},
    Element,
    Length::Fill,
    Size, Subscription, Task, Theme,
    keyboard::{Key, key::Named},
    mouse::ScrollDelta,
    widget, window,
};
use ::rustc_hash::FxBuildHasher;
use ::serde::{Deserialize, Serialize};
use ::tap::Pipe;

pub use self::{cli::Cli, theme_arg::ThemeArg};

mod cli;
mod theme_arg;

/// Application settings.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Application theme to use.
    #[serde(default)]
    pub theme: ThemeArg,
}

/// Application message.
#[derive(Debug, Clone, IsVariant)]
enum Message {
    /// Add window id to state.
    AddWindow(window::Id, Window),
    /// Remove a window from application state.
    RemoveWindow(window::Id),
    /// Set application theme.
    SetTheme(ThemeArg),
    /// Scroll theme.
    ThemeScroll(ScrollDelta),
    /// Keyboard event.
    KeyEvent(::iced::keyboard::Event),
    /// Save settings.
    SaveSettings,
    /// Reload settings.
    ReloadSettigns,
}

/// Window kinds.
#[derive(Debug, Clone, Default)]
enum Window {
    /// Window is a main window.
    #[default]
    Main,
    /// Window is a settings window.
    Settings,
}

/// Application state.
#[derive(Debug, Default)]
struct State {
    /// Application windows.
    windows: HashMap<window::Id, Window, FxBuildHasher>,

    /// Cli arguments of application.
    cli: Cli,

    /// Xdg base directories for application.
    xdg_dirs: ::xdg::BaseDirectories,

    /// Settings used by application.
    settings: Settings,

    /// Scroll state of theme pick list.
    theme_scroll: f32,
}

impl State {
    /// Initilize state.
    fn init(
        cli: Cli,
        settings: Settings,
        xdg_dirs: ::xdg::BaseDirectories,
    ) -> impl Fn() -> (Self, Task<Message>) {
        move || {
            let (_, open_main) = window::open(window::Settings::default());

            (
                Self {
                    cli: cli.clone(),
                    xdg_dirs: xdg_dirs.clone(),
                    settings: settings.clone(),
                    ..Self::default()
                },
                open_main.map(|id| Message::AddWindow(id, Window::Main)),
            )
        }
    }

    /// Get main application theme.
    fn main_theme(&self) -> Theme {
        self.settings.theme.into()
    }

    /// Get application theme.
    fn theme(&self, _id: window::Id) -> Theme {
        self.main_theme()
    }

    /// Get Application title.
    fn title(&self, id: window::Id) -> String {
        match self.windows.get(&id) {
            Some(Window::Settings) => "Arkiv Katalog: Settings".to_owned(),
            _ => "Arkiv Katalog".to_owned(),
        }
    }

    /// Get application subscriptions.
    fn subscription(&self) -> Subscription<Message> {
        let close_window = window::close_events().map(Message::RemoveWindow);
        let key_event = ::iced::keyboard::listen().map(Message::KeyEvent);

        Subscription::batch([close_window, key_event])
    }

    /// Update application state.
    fn update(&mut self, message: Message) -> Task<Message> {
        let report_err = |err: Report| {
            writeln!(::std::io::stdout().lock(), "{err}").expect("write to stdout should not fail")
        };
        match message {
            Message::AddWindow(id, window) => {
                self.windows.insert(id, window);
                Task::none()
            }
            Message::RemoveWindow(id) => {
                self.windows.remove(&id);
                if self.windows.is_empty() {
                    ::iced::exit()
                } else {
                    Task::none()
                }
            }
            Message::SetTheme(theme_arg) => {
                self.settings.theme = theme_arg;
                Task::none()
            }
            Message::KeyEvent(event) => match event {
                ::iced::keyboard::Event::KeyReleased { key, modifiers, .. } => match key.as_ref() {
                    Key::Named(Named::F2) if modifiers.is_empty() => {
                        let to_close = self
                            .windows
                            .iter()
                            .filter(|&(_id, ty)| matches!(ty, Window::Settings))
                            .map(|(id, _ty)| window::close(*id).map(Message::RemoveWindow))
                            .collect::<Vec<_>>();

                        if to_close.is_empty() {
                            let (_, task) = window::open(window::Settings {
                                size: Size {
                                    width: 400.0,
                                    height: 400.0,
                                },
                                ..window::Settings::default()
                            });
                            task.map(|id| Message::AddWindow(id, Window::Settings))
                        } else {
                            Task::batch(to_close)
                        }
                    }
                    _ => Task::none(),
                },
                _ => Task::none(),
            },
            Message::SaveSettings => {
                if let Err(err) = self
                    .xdg_dirs
                    .place_config_file("config.toml")
                    .map_err(|err| eyre!(err))
                    .and_then(|path| {
                        let content =
                            ::toml::to_string_pretty(&self.settings).map_err(|err| eyre!(err))?;
                        ::std::fs::write(&path, content).map_err(|err| {
                            eyre!("could not write settings to {path:?}").error(err)
                        })?;
                        Ok(())
                    })
                {
                    report_err(err);
                }
                Task::none()
            }
            Message::ReloadSettigns => {
                let settings = self
                    .xdg_dirs
                    .find_config_file("config.toml")
                    .map(|path| -> ::color_eyre::Result<Settings> {
                        let content = ::std::fs::read_to_string(&path).map_err(|err| {
                            eyre!("could not read to {path:?} to a utf-8 string").error(err)
                        })?;
                        ::toml::from_str(&content).map_err(|err| eyre!(err))
                    })
                    .transpose();

                match settings {
                    Ok(settings) => {
                        self.settings = settings.unwrap_or_default();
                        Task::none()
                    }
                    Err(err) => {
                        report_err(err);
                        Task::none()
                    }
                }
            }
            Message::ThemeScroll(delta) => {
                if let ScrollDelta::Pixels { y, .. } = delta {
                    self.theme_scroll += y;
                    let steps = self.theme_scroll.div_euclid(50.0);
                    let rem = self.theme_scroll.rem_euclid(50.0);
                    self.theme_scroll = rem;
                    if steps != 0.0 {
                        println!("{steps}, {rem}");
                    }
                }
                Task::none()
            }
        }
    }

    /// View application
    fn view(&self, id: window::Id) -> Element<'_, Message> {
        let ty = self.windows.get(&id).unwrap_or(&Window::Main);
        match ty {
            Window::Main => widget::Column::new()
                .padding(5)
                .spacing(3)
                .push(widget::space::vertical())
                .push(widget::rule::horizontal(2))
                .push(
                    widget::Row::new()
                        .align_y(Center)
                        .spacing(0)
                        .push(widget::space::horizontal())
                        .push(widget::text(format!("profile: {}", self.cli.profile,))),
                )
                .into(),
            Window::Settings => widget::Column::new()
                .padding(5)
                .spacing(3)
                .align_x(Center)
                .push(widget::space::vertical())
                .push(
                    widget::Column::new()
                        .spacing(3)
                        .push(
                            widget::Row::new()
                                .align_y(Center)
                                .spacing(3)
                                .push("Theme")
                                .push(
                                    widget::mouse_area(
                                        widget::pick_list(
                                            ThemeArg::value_variants(),
                                            Some(self.settings.theme),
                                            Message::SetTheme,
                                        )
                                        .padding(3),
                                    )
                                    .on_scroll(Message::ThemeScroll),
                                ),
                        )
                        .pipe(widget::container)
                        .style(widget::container::bordered_box)
                        .padding(5),
                )
                .push(widget::space::vertical())
                .push(
                    widget::Row::new()
                        .spacing(3)
                        .push(
                            widget::button("Save")
                                .padding(3)
                                .on_press(Message::SaveSettings)
                                .style(widget::button::success),
                        )
                        .push(
                            widget::button("Load")
                                .padding(3)
                                .on_press(Message::ReloadSettigns),
                        )
                        .pipe(widget::container)
                        .style(widget::container::bordered_box)
                        .padding(5)
                        .pipe(widget::container)
                        .width(Fill)
                        .align_x(Alignment::End),
                )
                .into(),
        }
    }
}
