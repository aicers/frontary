mod component;
mod view;

pub use component::{ItemKind, Kind, Message, Model};

#[cfg(feature = "pumpkin")]
pub(super) const DEFAULT_POP_WIDTH: u32 = 450;

#[cfg(not(feature = "pumpkin"))]
pub(super) const DEFAULT_POP_WIDTH: u32 = 350;

pub(super) const MIN_POP_HEIGHT: u32 = 500;
