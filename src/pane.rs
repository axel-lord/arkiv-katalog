//! [Pane] impl.

use ::std::{
    borrow::Cow,
    collections::BTreeMap,
    path::Path,
    sync::{Arc, LazyLock},
};

use ::derive_more::IsVariant;
use ::iced::{
    Element,
    Length::Fill,
    Padding,
    widget::{self, text::Wrapping},
};
use ::tap::Pipe;

use crate::{Message, shorten_text};

/// A Single main window pain.
#[derive(Debug, Clone, Default, IsVariant)]
pub enum DirView {
    /// Empty pane.
    #[default]
    Empty,
    /// Display a directory view.
    Dir {
        /// View Items.
        items: BTreeMap<Arc<Path>, Item>,
    },
}

/// Displayed item.
#[derive(Debug, Clone)]
pub struct Item {
    /// Name of item.
    pub name: String,
    /// Thumbnail of item.
    pub cover: Option<widget::image::Handle>,
}

impl DirView {
    /// View pane.
    pub fn view<'this>(
        &'this self,
        icon_width: f32,
        max_text_len: u16,
    ) -> impl Into<Element<'this, Message>> {
        static PLACEHOLDER: LazyLock<widget::svg::Handle> = LazyLock::new(|| {
            include_bytes!("./question.svg")
                .as_slice()
                .pipe(Cow::Borrowed)
                .pipe(widget::svg::Handle::from_memory)
        });
        match self {
            DirView::Empty => widget::button("Open...")
                .pipe(widget::container)
                .padding(5)
                .style(widget::container::bordered_box)
                .pipe(widget::container)
                .padding(5)
                .center(Fill),
            DirView::Dir { items } => widget::responsive(move |size| {
                let width = icon_width;
                let columns = size.width.div_euclid(width);
                widget::Grid::with_children(items.iter().map(|(_, Item { name, cover })| {
                    if let Some(handle) = cover {
                        widget::Stack::new().push(widget::image(handle).width(Fill).height(Fill))
                    } else {
                        widget::Stack::new()
                            .push(widget::svg(PLACEHOLDER.clone()).width(Fill).height(Fill))
                    }
                    .push(
                        widget::text(shorten_text(name, max_text_len.into()))
                            .wrapping(Wrapping::None)
                            .pipe(widget::container)
                            .style(widget::container::bordered_box)
                            .center_x(Fill)
                            .padding(3)
                            .pipe(widget::container)
                            .padding(Padding {
                                left: 5.0,
                                right: 5.0,
                                ..Padding::new(0.0)
                            })
                            .center_x(Fill)
                            .align_bottom(Fill),
                    )
                    .pipe(Element::from)
                }))
                .spacing(3)
                .columns(items.len().min(columns as usize))
                .width(if items.len() < columns as usize {
                    (items.len() as f32 + 1.0) * width
                } else {
                    size.width
                })
                .into()
            })
            .pipe(widget::scrollable)
            .pipe(widget::container)
            .padding(Padding {
                bottom: 0.0,
                ..Padding::new(5.0)
            }),
        }
    }
}
