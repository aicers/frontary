#![allow(clippy::non_ascii_literal)]

mod checkbox;
mod home;
mod input;
mod language;
mod list;
mod modal;
mod notification;
mod pages;
mod radio;
mod radio_separate;
mod select;
mod sort;
mod tab_menu;

use crate::home::Props;
use num_traits::ToPrimitive;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Clone, PartialEq, Eq)]
pub enum CommonError {
    SendGraphQLQueryError,
    HttpStatusNoSuccess(u16),
    GraphQLResponseError,
    GraphQLParseError,
    UnknownError,
}

const NBSP: &str = "&nbsp;";

#[wasm_bindgen(module = "/js/custom-select.js")]
extern "C" {
    fn toggle_visibility(id: &str);
    fn toggle_visibility_complex(id: &str);
    fn visibile_tag_select(id: &str);
}

fn window_inner_height() -> u32 {
    web_sys::window()
        .expect("Window should exist")
        .inner_height()
        .expect("should have height")
        .as_f64()
        .expect("should be a number")
        .to_u32()
        .unwrap_or(u32::MAX)
}

trait Rerender {
    fn rerender_serial(&mut self) -> &mut u64;
    fn increase_rerender_serial(&mut self) {
        *self.rerender_serial() = self.rerender_serial().wrapping_add(1);
    }
}

fn main() {
    let props = Props {};

    yew::start_app_with_props::<crate::home::Model>(props);
}
