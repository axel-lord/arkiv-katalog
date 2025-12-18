//! discrete scrolling from scroll delta events.

use ::derive_more::IsVariant;
use ::iced::mouse::ScrollDelta;

pub use Axis::{Horizontal, Vertical};

/// Scroll direction.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, IsVariant)]
pub enum Direction {
    /// Forward scroll was performed.
    Forwards,
    /// Backward scroll was performed.
    Backwards,
    /// No Scroll was performed.
    #[default]
    Stationary,
}

/// Axis to scroll along.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, IsVariant)]
pub enum Axis {
    /// Vertical top <-> bottom scroll.
    #[default]
    Vertical,
    /// Horizontal left <-> right scroll.
    Horizontal,
}

impl Axis {
    /// Perform a discrete scroll along axis using  given delta and accumulator.
    pub fn discrete_scroll(self, delta: ScrollDelta, accumulator: &mut f32) -> Direction {
        let steps = match delta {
            ScrollDelta::Lines { x, y } => match self {
                Axis::Vertical => y,
                Axis::Horizontal => x,
            },
            ScrollDelta::Pixels { x, y } => {
                let amount = match self {
                    Axis::Vertical => y,
                    Axis::Horizontal => x,
                };
                *accumulator += amount;
                let steps = accumulator.div_euclid(50.0);
                *accumulator = accumulator.rem_euclid(50.0);
                steps
            }
        };

        if steps < -0.9 {
            Direction::Forwards
        } else if steps > 0.9 {
            Direction::Backwards
        } else {
            Direction::Stationary
        }
    }
}
