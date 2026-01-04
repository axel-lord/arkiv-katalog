//! [Window] impl.

use ::clap::ValueEnum;
use ::iced::{
    Alignment::{self, Center},
    Background, Element,
    Length::Fill,
    Padding,
    widget::{self, pane_grid},
};
use ::katalog_lib::ThemeValueEnum;
use ::tap::Pipe;

use crate::{Cli, Message, Settings, pane::DirView};

/// Window kinds.
#[derive(Debug, Clone)]
pub enum Window {
    /// Window is a main window.
    Main {
        /// Panes of window.
        panes: pane_grid::State<DirView>,
    },
    /// Window is a settings window.
    Settings,
}

impl Window {
    /// View window state.
    pub fn view<'this>(
        &'this self,
        cli: &'this Cli,
        settings: &'this Settings,
    ) -> impl Into<Element<'this, Message>> {
        match self {
            Window::Main { panes } => widget::Column::new()
                .push(widget::PaneGrid::new(panes, |pane, state, is_maximized| {
                    _ = (pane, is_maximized);
                    pane_grid::Content::new(state.view(settings.card_width))
                }))
                .push(
                    widget::Column::new()
                        .spacing(3)
                        .padding(Padding {
                            top: 0.0,
                            ..Padding::new(5.0)
                        })
                        .width(Fill)
                        .push(widget::rule::horizontal(2))
                        .push(
                            widget::Row::new()
                                .align_y(Center)
                                .spacing(0)
                                .push(widget::space::horizontal())
                                .push(widget::text(format!("profile: {}", cli.profile))),
                        )
                        .pipe(widget::container)
                        .style(|theme: &::iced::Theme| widget::container::Style {
                            background: Some(Background::Color(theme.palette().background)),
                            ..widget::container::transparent(theme)
                        }),
                ),
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
                                            ThemeValueEnum::value_variants(),
                                            Some(settings.theme),
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
                ),
        }
    }
}
