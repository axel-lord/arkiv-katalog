//! [Pane] impl.

use ::std::{collections::BTreeMap, path::Path, sync::Arc};

use ::derive_more::IsVariant;
use ::iced::{Element, Length::Fill, Padding, widget};
use ::tap::Pipe;

use crate::Message;

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
}

impl DirView {
    /// View pane.
    pub fn view<'this>(&'this self, icon_width: f32) -> impl Into<Element<'this, Message>> {
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
                widget::Grid::with_children(items.iter().map(|(_, Item { name })| {
                    widget::text(name)
                        .pipe(widget::container)
                        .padding(5)
                        .style(widget::container::bordered_box)
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
