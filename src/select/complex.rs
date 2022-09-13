mod component;
mod view;

pub use component::{Kind, Message, Model};
#[allow(dead_code)]
pub(super) const DEFAULT_POP_WIDTH: u32 = 350;
pub(super) const MIN_POP_HEIGHT: u32 = 500;
