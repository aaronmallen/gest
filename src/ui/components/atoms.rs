//! Atomic UI components — the smallest self-contained visual elements.

mod badge;
mod block;
mod column;
mod icon;
mod id;
mod indent;
mod label;
mod separator;
mod tag;
mod title;
mod value;

pub use badge::Component as Badge;
pub use column::{Component as Column, Flex};
pub use icon::Component as Icon;
pub use id::{Component as Id, min_unique_prefix};
pub use label::Component as Label;
pub use separator::Component as Separator;
pub use tag::Component as Tag;
pub use title::Component as Title;
pub use value::Component as Value;
