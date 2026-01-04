#![doc = include_str!("../README.md")]

use ::std::{borrow::Cow, collections::BTreeMap, io::Write, path::Path, sync::Arc};

use ::color_eyre::{Report, Section, eyre::eyre};
use ::derive_more::IsVariant;
use ::iced::{
    Element, Size, Subscription, Task, Theme,
    keyboard::{Key, key::Named},
    mouse::ScrollDelta,
    widget::pane_grid,
    window,
};
use ::katalog_lib::{PartialVariants, ThemeValueEnum, discrete_scroll};
use ::serde::{Deserialize, Serialize};
use ::smol::stream::StreamExt;
use ::tap::Pipe;
use ::unicode_segmentation::UnicodeSegmentation;

use crate::{pane::DirView, window_state::Window};

pub use self::cli::Cli;

mod cli;
mod pane;
mod window_state;

/// Shorten text such that it is at most max_len long.
fn shorten_text(text: &str, max_len: usize) -> Cow<'_, str> {
    fn inner(text: &str, max_len: usize) -> Option<String> {
        let mut grapheme_indices = text.grapheme_indices(true);
        let (end, _) = grapheme_indices.nth(max_len.saturating_sub(3))?;
        _ = grapheme_indices.nth(3)?;
        let text = text.get(..end)?;
        let mut buf = String::with_capacity(end + 3);
        buf.push_str(text);
        buf.push_str("...");
        Some(buf)
    }
    inner(text, max_len).map_or(Cow::Borrowed(text), Cow::Owned)
}

/// Application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Application theme to use.
    pub theme: ThemeValueEnum,

    /// Card width to use.
    pub card_width: u16,

    /// Max width of card text.
    pub max_card_text_width: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Default::default(),
            card_width: 150,
            max_card_text_width: 12,
        }
    }
}

/// Path to a [DirView].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewPath {
    /// Window id of item.
    pub window_id: window::Id,
    /// Pane grid pane of item.
    pub pane: pane_grid::Pane,
}

/// Path to an item.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemPath {
    /// Path to [DirView] of item.
    pub view_path: ViewPath,
    /// Path of item.
    pub path: Arc<Path>,
}

/// Application message.
#[derive(Debug, Clone, IsVariant)]
enum Message {
    /// Add window displaying given directory.
    AddDirWindow(window::Id, Arc<Path>),
    /// Add empty window.
    AddEmptyWindow(window::Id),
    /// Add settings window.
    AddSettingsWindow(window::Id),
    /// Remove a window from application state.
    RemoveWindow(window::Id),
    /// Set application theme.
    SetTheme(ThemeValueEnum),
    /// Scroll theme.
    ThemeScroll(ScrollDelta),
    /// Keyboard event.
    KeyEvent(::iced::keyboard::Event),
    /// Add a directory item.
    AddItem {
        /// Path to add item at.
        item_path: ItemPath,
        /// Item to add.
        item: pane::Item,
    },
    /// Save settings.
    SaveSettings,
    /// Reload settings.
    ReloadSettigns,
}

/// Application state.
#[derive(Debug, Default)]
struct State {
    /// Application windows.
    windows: BTreeMap<window::Id, Window>,

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
        let dir_path = cli.directory.as_deref().map(Arc::<Path>::from);
        move || {
            (
                Self {
                    cli: cli.clone(),
                    xdg_dirs: xdg_dirs.clone(),
                    settings: settings.clone(),
                    ..Self::default()
                },
                dir_path.as_ref().map_or_else(
                    || {
                        let (_, open_window) = window::open(window::Settings::default());
                        open_window.map(Message::AddEmptyWindow)
                    },
                    |path| {
                        let (_, open_window) = window::open(window::Settings::default());
                        let path = Arc::clone(path);
                        open_window.map(move |id| Message::AddDirWindow(id, Arc::clone(&path)))
                    },
                ),
            )
        }
    }

    /// Get a mutable reference to a directory view.
    fn get_dir_view_mut(&mut self, view_path: ViewPath) -> Option<&mut DirView> {
        let Window::Main { panes } = self.windows.get_mut(&view_path.window_id)? else {
            return None;
        };
        panes.get_mut(view_path.pane)
    }

    /// Open a directory.
    fn open_dir(
        &self,
        path: Arc<Path>,
        prefix: Option<Arc<str>>,
        view_path: ViewPath,
    ) -> Task<Message> {
        ::smol::fs::read_dir(Arc::clone(&path))
            .pipe(Task::future)
            .map({
                let path = Arc::clone(&path);
                move |result| {
                    result
                        .map_err(|err| ::log::error!("culd not read {path:?}\n{err}"))
                        .ok()
                }
            })
            .and_then(move |read_dir| {
                let prefix = prefix.clone();
                read_dir
                    .filter_map({
                        let path = Arc::clone(&path);
                        move |entry| {
                            entry
                                .map_err(|err| {
                                    ::log::warn!("io error while reading directory {path:?}\n{err}")
                                })
                                .ok()
                        }
                    })
                    .then(move |entry| {
                        let prefix = prefix.clone();
                        async move {
                            let name = format!(
                                "{prefix}{name}",
                                prefix = prefix.as_deref().unwrap_or(""),
                                name = entry.file_name().display()
                            );
                            let path = Arc::from(entry.path());
                            Message::AddItem {
                                item_path: ItemPath { view_path, path },
                                item: pane::Item { name, cover: None },
                            }
                        }
                    })
                    .pipe(Task::stream)
            })
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
            Message::AddDirWindow(window_id, path) => {
                let (panes, pane) = pane_grid::State::new(pane::DirView::Empty);
                self.windows.insert(window_id, Window::Main { panes });
                self.open_dir(path, None, ViewPath { window_id, pane })
            }
            Message::AddEmptyWindow(id) => {
                let (panes, _) = pane_grid::State::new(pane::DirView::Empty);
                self.windows.insert(id, Window::Main { panes });
                Task::none()
            }
            Message::AddSettingsWindow(id) => {
                self.windows.insert(id, Window::Settings);
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
                            task.map(Message::AddSettingsWindow)
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
                match discrete_scroll::Vertical.discrete_scroll(delta, &mut self.theme_scroll) {
                    discrete_scroll::Direction::Forwards => {
                        self.settings.theme = *self.settings.theme.partial_cycle_next();
                    }
                    discrete_scroll::Direction::Backwards => {
                        self.settings.theme = *self.settings.theme.partial_cycle_prev();
                    }
                    discrete_scroll::Direction::Stationary => {}
                }
                Task::none()
            }
            Message::AddItem {
                item_path: ItemPath { view_path, path },
                item,
            } => {
                let Some(view) = self.get_dir_view_mut(view_path) else {
                    ::log::warn!("could not resolve view path {view_path:?}");
                    return Task::none();
                };

                match view {
                    DirView::Empty => {
                        *view = DirView::Dir {
                            items: BTreeMap::from_iter([(path, item)]),
                        }
                    }
                    DirView::Dir { items, .. } => {
                        items.insert(path, item);
                    }
                }

                Task::none()
            }
        }
    }

    /// View application
    fn view(&self, id: window::Id) -> Element<'_, Message> {
        let ty = self.windows.get(&id).unwrap_or(&Window::Settings);
        ty.view(&self.cli, &self.settings).into()
    }
}
