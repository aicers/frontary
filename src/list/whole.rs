mod component;
mod function;
mod view;

use component::Message;
pub use component::Model;
pub use component::SortColumn;
use component::ViewInputStatus;

#[cfg(not(feature = "test"))]
const DEFAULT_NUM_PER_PAGE: usize = 10;
#[cfg(not(feature = "test"))]
const DEFAULT_NUM_PAGES: usize = 10;

#[cfg(feature = "test")]
const DEFAULT_NUM_PER_PAGE: usize = 2;
#[cfg(feature = "test")]
const DEFAULT_NUM_PAGES: usize = 2;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
    Add,
    Edit,
    Delete,

    AddSecond,
    EditSecond,
    DeleteSecond,

    AddTag,
    EditTag,
    DeleteTag,

    InputError,
    FileLoadError,
}
