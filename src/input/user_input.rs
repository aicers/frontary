use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gloo_file::File;
use json_gettext::get_text;
use num_bigint::BigUint;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yew::{Component, Context, Html, events::InputEvent, html, html::TargetCast};

use super::{
    InputItem, cal_index,
    component::{InputSecondId, InvalidMessage, Message, Model},
};
use crate::{
    HostNetworkHtml, HostNetworkKind, InputEssential, InvalidPasswordKind as Kind, Tag, Theme,
    input::{component::Verification, config::ValidationFn},
    text,
};

const CHANGE_PASSWORD_NOTICE: &str = "If you want to change your password, input a new one.";
const EXISTING_MSG: &str = "The input already exists.";
const REQUIRED_MSG: &str = "This field is required.";
const INVALID_MSG: &str = "Invalid input";
const PASSWD_NOT_MATCH_MSG: &str = "Passwords must match.";
const PASSWD_HAS_SPACE_MSG: &str = "Your password must not constain any spaces.";
const PASSWD_HAS_CONTROL_CHARACTER_MSG: &str =
    "Your password must not contain any control characters.";
const PASSWD_TOO_SHORT_MSG: &str = "Your password is too short.";
const PASSWD_NO_LOWER_MSG: &str = "Your password must contain at least one lowercase alphabet.";
const PASSWD_NO_UPPER_MSG: &str = "Your password must contain at least one uppercase alphabet.";
const PASSWD_NO_NUMBER_MSG: &str = "Your password must contain at least one number.";
const PASSWD_NO_SYMBOL_MSG: &str = "Your password must contain at least one special character.";
const PASSWD_CONSECUTIVE_MSG: &str =
    "Your password must not contain consecutive repeating characters.";
const PASSWD_ADJACENT_MSG: &str =
    "Your password must not contain more than 3 adjacent keyboard characters.";
const PASSWD_REQUIREMENT: &str = if cfg!(feature = "cc-password") {
    "no spaces, between 9 and 64 characters, at least one number/uppercase/lowercase/special characters, no consecutive repetition, and less than 4 adjacent keyboard characters"
} else {
    "no spaces, between 8 and 64 characters, at least one number/uppercase/lowercase/special characters"
};
const FLOAT64_STEP_DEFAULT: f64 = 0.1;

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_text(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        length: Option<usize>,
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        autofocus: bool,
        group: bool,
        immutable: bool,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputText(
                        my_index_clone.clone(),
                        input.value(),
                        input_data_clone.clone(),
                    )
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Text(data) = &(*input_data) {
                data.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let class = if self.required_msg.contains(&my_index)
            || self.unique_msg.contains(&my_index)
            || matches!(
                self.verification.get(&my_index),
                Some(Verification::Invalid(InvalidMessage::InvalidCustom(_)))
            ) {
            "frontary-input-text-alert"
        } else {
            "frontary-input-text"
        };
        let class_item = if group {
            "input-item-group"
        } else {
            "input-item"
        };
        let style = format!(
            "width: {};",
            width.map_or("100%".to_string(), |w| format!("{w}px"))
        );
        let is_edit_mode = match ctx.props().input_second_id.as_ref() {
            Some(InputSecondId::Add) => false,
            Some(InputSecondId::Edit(_)) => true,
            None => ctx.props().input_id.is_some(),
        };
        html! {
            <div class={class_item}>
                {
                    if group {
                        html! {}
                    } else {
                        html! {
                            if !ess.title.is_empty() {
                                if cfg!(feature = "pumpkin") {
                                    <div class="input-contents-text-item-title">
                                        { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                                    </div>
                                } else {
                                    <div class="input-contents-item-title">
                                        { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                                    </div>
                                }
                            }
                        }
                    }
                }
                {
                    if let Some(length) = length {
                        html! {
                            <>
                                <input type="text" class={class} style={style}
                                    value={value}
                                    placeholder={placeholder}
                                    autofocus={autofocus}
                                    autocomplete="off"
                                    oninput={oninput}
                                    maxlength={length.to_string()}
                                    disabled={is_edit_mode && immutable}
                                />
                                { Self::view_explanation_msg(ctx)}
                            </>
                        }
                    } else {
                        html! {
                            <>
                                <input type="text" class={class} style={style}
                                    value={value}
                                    placeholder={placeholder}
                                    autofocus={autofocus}
                                    autocomplete="off"
                                    oninput={oninput}
                                    disabled={is_edit_mode && immutable}
                                />
                                { Self::view_explanation_msg(ctx)}
                            </>
                        }
                    }
                }
                <div class="input-text-message">
                    { self.view_required_msg(ctx, &my_index) }
                </div>
                {
                    if self.unique_msg.contains(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                                { text!(txt, ctx.props().language, EXISTING_MSG)}
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    match self.verification.get(&my_index) {
                        Some(Verification::Invalid(InvalidMessage::InvalidCustom(msg))) => html! {
                            <div class="input-contents-item-alert-message">
                               { text!(txt, ctx.props().language, msg) }
                            </div>
                        },
                        _ => html! {}
                    }
                }
            </div>
        }
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_domain_name(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        autofocus: bool,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputDomainName(
                        my_index_clone.clone(),
                        input.value(),
                        input_data_clone.clone(),
                    )
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::DomainName(data) = &(*input_data) {
                data.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let class = if self.required_msg.contains(&my_index)
            || self.unique_msg.contains(&my_index)
            || self.verification.get(&my_index)
                == Some(&Verification::Invalid(InvalidMessage::InvalidDomain))
        {
            "frontary-input-text-alert"
        } else {
            "frontary-input-text"
        };
        let style = format!(
            "width: {};",
            width.map_or("100%".to_string(), |w| format!("{w}px"))
        );

        html! {
            <div class="input-item">
                {
                    if ess.title.is_empty() {
                        html! {}
                    } else {
                        html! {
                            if cfg!(feature = "pumpkin") {
                                <div class="input-contents-text-item-title">
                                    { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                                </div>
                            } else {
                                <div class="input-contents-item-title">
                                    { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                                </div>
                            }
                        }
                    }
                }
                <input type="text" class={class} style={style}
                    value={value}
                    placeholder={placeholder}
                    autofocus={autofocus}
                    autocomplete="off"
                    oninput={oninput}
                />
                { Self::view_explanation_msg(ctx)}
                <div class="input-text-message">
                    { self.view_required_msg(ctx, &my_index) }
                </div>
                {
                    if self.unique_msg.contains(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                                { text!(txt, ctx.props().language, EXISTING_MSG)}
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if self.verification.get(&my_index) == Some(&Verification::Invalid(InvalidMessage::InvalidDomain)) {
                        html! {
                            <div class="input-contents-item-alert-message">
                                { text!(txt, ctx.props().language, "Invalid domain name") }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_password(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        length: Option<usize>,
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        autofocus: bool,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputPassword(
                        my_index_clone.clone(),
                        input.value(),
                        input_data_clone.clone(),
                    )
                })
        });
        let oninput_confirm = |index: BigUint| {
            ctx.link().callback(move |e: InputEvent| {
                e.target()
                    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                    .map_or(Message::InputError, |input| {
                        Message::InputConfirmPassword(index.clone(), input.value())
                    })
            })
        };
        let placeholder = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Password(item) = &(*input_data) {
                if item.is_empty() {
                    if ctx.props().input_id.is_none() {
                        text!(txt, ctx.props().language, ess.notice).to_string()
                    } else {
                        text!(txt, ctx.props().language, CHANGE_PASSWORD_NOTICE).to_string()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let class = if self.required_msg.contains(&my_index) {
            "input-password-alert"
        } else {
            "input-password"
        };
        let style = format!(
            "width: {};",
            width.map_or("100%".to_string(), |w| format!("{w}px"))
        );

        html! {
            <div class="input-item">
                <div class="input-contents-item-title">
                    { text!(txt, ctx.props().language, ess.title()) }
                    {
                        if ctx.props().input_id.is_none() {
                            { view_asterisk(ess.required) }
                        } else {
                            html! {}
                        }
                    }
                </div>
                <div class="input-contents-item-input-password">
                    <input type="password" class={class} style={style.clone()}
                        placeholder={placeholder.clone()}
                        autofocus={autofocus}
                        autocomplete="new-password"
                        oninput={oninput}
                        maxlength={length.map(|l| l.to_string())}
                    />
                </div>
                <div class="input-password-notice">
                    { text!(txt, ctx.props().language, PASSWD_REQUIREMENT)}
                </div>
                <div class="input-reenter-password-title">
                    { text!(txt, ctx.props().language, "Re-enter password") }
                </div>
                <input type="password" class={class} style={style}
                    placeholder={placeholder}
                    autocomplete="new-password"
                    oninput={oninput_confirm(my_index.clone())}
                    maxlength={length.map(|l| l.to_string())}
                />
                <div class="input-text-message">
                    { self.view_required_msg(ctx, &my_index) }
                </div>
                {
                    if let Some(Verification::Invalid(InvalidMessage::InvalidPassword(m))) = self.verification.get(&my_index) {
                        let msg = match m {
                            Kind::HasSpace => Some(PASSWD_HAS_SPACE_MSG),
                            Kind::HasControlCharacter => Some(PASSWD_HAS_CONTROL_CHARACTER_MSG),
                            Kind::NotMatch => Some(PASSWD_NOT_MATCH_MSG),
                            Kind::TooShort => Some(PASSWD_TOO_SHORT_MSG),
                            Kind::NoLowercaseLetter => Some(PASSWD_NO_LOWER_MSG),
                            Kind::NoUppercaseLetter => Some(PASSWD_NO_UPPER_MSG),
                            Kind::NoNumber => Some(PASSWD_NO_NUMBER_MSG),
                            Kind::NoSymbol => Some(PASSWD_NO_SYMBOL_MSG),
                            Kind::HasConsecutiveLetters => Some(PASSWD_CONSECUTIVE_MSG),
                            Kind::HasAdjacentLetters => Some(PASSWD_ADJACENT_MSG),
                        };
                        if let Some(msg) = msg {
                            html! {
                                <div class="input-contents-item-alert-message">
                                    { text!(txt, ctx.props().language, msg) }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_unsigned_32(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        min: u32,
        max: u32,
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        autofocus: bool,
        group: bool,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    let value = input.value();
                    if value.is_empty() {
                        Message::InputUnsigned32(
                            my_index_clone.clone(),
                            None,
                            input_data_clone.clone(),
                        )
                    } else if let Ok(parsed) = value.parse::<u32>() {
                        Message::InputUnsigned32(
                            my_index_clone.clone(),
                            Some(parsed),
                            input_data_clone.clone(),
                        )
                    } else {
                        Message::InvalidInputUnsigned32
                    }
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Unsigned32(value) = &*input_data {
                value.into_inner()
            } else {
                None
            }
        } else {
            None
        };
        let class = if self.required_msg.contains(&my_index) {
            "input-number-alert"
        } else {
            "input-number"
        };
        let class_item = if group {
            "input-item-group"
        } else {
            "input-item"
        };
        let style = format!(
            "width: {};",
            width.map_or("100%".to_string(), |w| format!("{w}px"))
        );

        html! {
            <div class={class_item}>
                if cfg!(feature = "debug") {
                    { format!("({}:{}={})", base_index.map_or_else(String::new, ToString::to_string), layer_index, my_index.clone()) }
                }
                {
                    if group {
                        html! {}
                    } else {
                        html! {
                            <div class="input-contents-item-title">
                                { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                            </div>
                        }
                    }
                }
                <div class="input-contents-item-input">
                {
                    if let Some(value) = value {
                        html! {
                            <input type="number" class={class} style={style}
                                value={value.to_string()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                                min={min.to_string()}
                                max={max.to_string()}
                            />
                        }
                    } else {
                        html! {
                            <input type="number" class={class} style={style}
                                // HIGHLIGHT: This must be set to empty string. If not, the previous
                                // input shows here when another item in the group is added.
                                value={String::new()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                                min={min.to_string()}
                                max={max.to_string()}
                            />
                        }
                    }
                }
                </div>
                <div class="input-text-message">
                    { self.view_required_msg(ctx, &my_index) }
                </div>
                {
                    if self.unique_msg.contains(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                                { text!(txt, ctx.props().language, EXISTING_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if let Some(Verification::Invalid(InvalidMessage::InvalidInput)) = self.verification.get(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                               { text!(txt, ctx.props().language, INVALID_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_unsigned_16(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        min: u16,
        max: u16,
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        autofocus: bool,
        group: bool,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    let value = input.value();
                    if value.is_empty() {
                        Message::InputUnsigned16(
                            my_index_clone.clone(),
                            None,
                            input_data_clone.clone(),
                        )
                    } else if let Ok(parsed) = value.parse::<u16>() {
                        Message::InputUnsigned16(
                            my_index_clone.clone(),
                            Some(parsed),
                            input_data_clone.clone(),
                        )
                    } else {
                        Message::InvalidInputUnsigned16
                    }
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Unsigned16(value) = &*input_data {
                value.into_inner()
            } else {
                None
            }
        } else {
            None
        };
        let class = if self.required_msg.contains(&my_index) {
            "input-number-alert"
        } else {
            "input-number"
        };
        let class_item = if group {
            "input-item-group"
        } else {
            "input-item"
        };
        let style = format!(
            "width: {};",
            width.map_or("100%".to_string(), |w| format!("{w}px"))
        );

        html! {
            <div class={class_item}>
                if cfg!(feature = "debug") {
                    { format!("({}:{}={})", base_index.map_or_else(String::new, ToString::to_string), layer_index, my_index.clone()) }
                }
                {
                    if group {
                        html! {}
                    } else {
                        html! {
                            <div class="input-contents-item-title">
                                { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                            </div>
                        }
                    }
                }
                <div class="input-contents-item-input">
                {
                    if let Some(value) = value {
                        html! {
                            <input type="number" class={class} style={style}
                                value={value.to_string()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                                min={min.to_string()}
                                max={max.to_string()}
                            />
                        }
                    } else {
                        html! {
                            <input type="number" class={class} style={style}
                                // HIGHLIGHT: This must be set to empty string. If not, the previous
                                // input shows here when another item in the group is added.
                                value={String::new()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                                min={min.to_string()}
                                max={max.to_string()}
                            />
                        }
                    }
                }
                </div>
                <div class="input-text-message">
                    { self.view_required_msg(ctx, &my_index) }
                </div>
                {
                    if self.unique_msg.contains(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                                { text!(txt, ctx.props().language, EXISTING_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if let Some(Verification::Invalid(InvalidMessage::InvalidInput)) = self.verification.get(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                               { text!(txt, ctx.props().language, INVALID_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_unsigned_8(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        min: u8,
        max: u8,
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        autofocus: bool,
        group: bool,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    let value = input.value();
                    if value.is_empty() {
                        Message::InputUnsigned8(
                            my_index_clone.clone(),
                            None,
                            input_data_clone.clone(),
                        )
                    } else if let Ok(parsed) = value.parse::<u8>() {
                        Message::InputUnsigned8(
                            my_index_clone.clone(),
                            Some(parsed),
                            input_data_clone.clone(),
                        )
                    } else {
                        Message::InvalidInputUnsigned8
                    }
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Unsigned8(value) = &*input_data {
                value.into_inner()
            } else {
                None
            }
        } else {
            None
        };
        let class = if self.required_msg.contains(&my_index) {
            "input-number-alert"
        } else {
            "input-number"
        };
        let class_item = if group {
            "input-item-group"
        } else {
            "input-item"
        };
        let style = format!(
            "width: {};",
            width.map_or("100%".to_string(), |w| format!("{w}px"))
        );

        html! {
            <div class={class_item}>
                if cfg!(feature = "debug") {
                    { format!("({}:{}={})", base_index.map_or_else(String::new, ToString::to_string), layer_index, my_index.clone()) }
                }
                {
                    if group {
                        html! {}
                    } else {
                        html! {
                            <div class="input-contents-item-title">
                                { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                            </div>
                        }
                    }
                }
                <div class="input-contents-item-input">
                {
                    if let Some(value) = value {
                        html! {
                            <input type="number" class={class} style={style}
                                value={value.to_string()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                                min={min.to_string()}
                                max={max.to_string()}
                            />
                        }
                    } else {
                        html! {
                            <input type="number" class={class} style={style}
                                // HIGHLIGHT: This must be set to empty string. If not, the previous
                                // input shows here when another item in the group is added.
                                value={String::new()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                                min={min.to_string()}
                                max={max.to_string()}
                            />
                        }
                    }
                }
                </div>
                <div class="input-text-message">
                    { self.view_required_msg(ctx, &my_index) }
                </div>
                {
                    if self.unique_msg.contains(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                                { text!(txt, ctx.props().language, EXISTING_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if let Some(Verification::Invalid(InvalidMessage::InvalidInput)) = self.verification.get(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                               { text!(txt, ctx.props().language, INVALID_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                <div class="input-contents-item-space">
                </div>
            </div>
        }
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_float_64(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        step: Option<f64>,
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        autofocus: bool,
        group: bool,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    let value = input.value();
                    if value.is_empty() {
                        Message::InputFloat64(
                            my_index_clone.clone(),
                            None,
                            input_data_clone.clone(),
                        )
                    } else if let Ok(parsed) = value.parse::<f64>() {
                        Message::InputFloat64(
                            my_index_clone.clone(),
                            Some(parsed),
                            input_data_clone.clone(),
                        )
                    } else {
                        Message::InvalidInputFloat64
                    }
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Float64(value) = &*input_data {
                value.into_inner()
            } else {
                None
            }
        } else {
            None
        };
        let class = if self.required_msg.contains(&my_index) {
            "input-number-alert"
        } else {
            "input-number"
        };
        let class_item = if group {
            "input-item-group"
        } else {
            "input-item"
        };
        let style = if cfg!(feature = "pumpkin") {
            None
        } else {
            Some(format!(
                "width: {};",
                width.map_or("100%".to_string(), |w| format!("{w}px"))
            ))
        };
        let step = step.unwrap_or(FLOAT64_STEP_DEFAULT);

        html! {
            <div class={class_item}>
            {
                if group {
                    html! {}
                } else {
                    html! {
                        <div class="input-contents-item-title">
                            { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                        </div>
                    }
                }
            }
                <div class="input-contents-item-input" style={style}>
                {
                    if let Some(value) = value {
                        html! {
                            <input type="number" class={class}
                                value={value.to_string()}
                                step={step.to_string()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                            />
                        }
                    } else {
                        html! {
                            <input type="number" class={class}
                                // HIGHLIGHT: This must be set to empty string. If not, the previous
                                // input shows here when another item in the group is added.
                                value={String::new()}
                                step={step.to_string()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                            />
                        }
                    }
                }
                </div>
                <div class="input-text-message">
                    { self.view_required_msg(ctx, &my_index) }
                </div>
                {
                    if self.unique_msg.contains(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                                { text!(txt, ctx.props().language, EXISTING_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if let Some(Verification::Invalid(InvalidMessage::InvalidInput)) = self.verification.get(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                               { text!(txt, ctx.props().language, INVALID_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines)]
    pub(super) fn view_percentage(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        min: Option<f32>,
        max: Option<f32>,
        decimals: Option<usize>,
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        autofocus: bool,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    let input_value = input.value();
                    if input_value.is_empty() {
                        Message::InputPercentage(
                            my_index_clone.clone(),
                            None,
                            input_data_clone.clone(),
                        )
                    } else if let Ok(value) = input_value.parse::<f32>() {
                        Message::InputPercentage(
                            my_index_clone.clone(),
                            Some(value / 100.0),
                            input_data_clone.clone(),
                        )
                    } else {
                        Message::InvalidInputPercentage
                    }
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Percentage(value) = &*input_data {
                value.as_ref().map(|v| v * 100.0)
            } else {
                None
            }
        } else {
            None
        };

        let class = if self.required_msg.contains(&my_index) {
            "input-number-alert"
        } else {
            "input-number"
        };
        let style = format!(
            "width: {};",
            width.map_or("100%".to_string(), |w| format!("{w}px"))
        );

        let min = min.map_or(0.0, |v| v * 100.0);
        let max = max.map_or(100.0, |v| v * 100.0);
        let decimals = decimals.map_or("0.1".to_string(), |d| {
            let mut r = "0.".to_string();
            if d > 0 {
                r.push_str(&"0".repeat(d - 1));
            }
            r.push('1');
            r
        });

        html! {
            <div class="input-item">
                <div class="input-contents-item-title">
                    { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                </div>
                <div class="input-contents-item-input">
                {
                    if let Some(value) = value {
                        html! {
                            <input type="number" class={class} style={style}
                                value={value.to_string()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                                min={min.to_string()}
                                max={max.to_string()}
                                step={decimals}
                            />
                        }
                    } else {
                        html! {
                            <input type="number" class={class} style={style}
                                // HIGHLIGHT: This must be set to empty string. If not, the previous
                                // input shows here when another item in the group is added.
                                value={String::new()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                                min={min.to_string()}
                                max={max.to_string()}
                                step={decimals}
                            />
                        }
                    }
                }
                </div>
                <div class="input-text-message">
                    { self.view_required_msg(ctx, &my_index) }
                </div>
                {
                    if self.unique_msg.contains(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                                { text!(txt, ctx.props().language, EXISTING_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    if let Some(Verification::Invalid(InvalidMessage::InvalidInput)) = self.verification.get(&my_index) {
                        html! {
                            <div class="input-contents-item-alert-message">
                               { text!(txt, ctx.props().language, INVALID_MSG) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_host_network_group(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        kind: HostNetworkKind,
        num: Option<usize>,
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        theme: Option<Theme>,
        length: Option<usize>,
        validation: Option<ValidationFn>,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let txt = ctx.props().txt.txt.clone();
        if let Some(buffer) = self.host_network_buffer.get(&my_index) {
            html! {
                <div class="input-host-network-group">
                    <div class="input-contents-item-general-title">
                        { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                    </div>
                    <HostNetworkHtml<Self>
                        txt={ctx.props().txt.clone()}
                        language={ctx.props().language}
                        rerender_serial={self.rerender_serial_host_network}
                        {kind}
                        {num}
                        {width}
                        {length}
                        input_data={Rc::clone(buffer)}
                        input_notice={Some(ess.notice)}
                        parent_message={Some(Message::InputHostNetworkGroup(my_index.clone(), input_data.clone()))}
                        parent_message_save={Some(Message::RightHostNetworkGroup(my_index.clone(), input_data.clone()))}
                        parent_message_no_save={Some(Message::WrongHostNetworkGroup(my_index.clone()))}
                        parent_message_user_input={Some(Message::UserInputHostNetworkGroup(my_index.clone()))}
                        verify_to_save={self.verify_host_network_group}
                        is_required={self.required_msg.contains(&my_index)}
                        {theme}
                        {validation}
                    />
                    { self.view_required_msg(ctx, &my_index) }
                </div>
            }
        } else {
            html! {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_tag_group(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        prev_list: &HashMap<String, String>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        theme: Option<Theme>,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let txt = ctx.props().txt.txt.clone();
        if let Some(buffer) = self.tag_buffer.get(&my_index) {
            let prev_list = Rc::new(prev_list.clone());
            html! {
                <div class="input-tag-group">
                    <div class="input-contents-item-general-title">
                        { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                    </div>
                    <Tag<Self>
                        txt={ctx.props().txt.clone()}
                        language={ctx.props().language}
                        prev_list={Rc::clone(&prev_list)}
                        input_data={Rc::clone(buffer)}
                        input_notice={Some(ess.notice)}
                        parent_message={Some(Message::InputTagGroup(my_index.clone(), input_data.clone()))}
                        {theme}
                    />
                    { self.view_required_msg(ctx, &my_index) }
                </div>
            }
        } else {
            html! {}
        }
    }

    pub(super) fn view_file(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        extensions: &[String],
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let input_data_clone = input_data.clone();
        let onchange = ctx.link().callback(move |e: Event| {
            let mut result = Vec::new();
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                let files = js_sys::try_iter(&files).ok().and_then(|x| x);
                if let Some(files) = files {
                    let files = files
                        .filter_map(Result::ok)
                        .map(web_sys::File::from)
                        .map(File::from);
                    result.extend(files);
                }
            }
            Message::ChooseFile(my_index_clone.clone(), result, input_data_clone.clone())
        });
        let extensions = validate_extensions(extensions).join(",");
        let txt = ctx.props().txt.txt.clone();
        let file_name = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::File(file) = &(*input_data) {
                file.name().to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        html! {
            <div class="input-item">
                <div class="input-contents-item-title">
                    { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                </div>
                <div class="input-file">
                    <input class="input-file-file-name"
                        value={file_name}
                        readonly={true} />
                    <label for="input-file" class="input-file-choose-file">
                        { text!(txt, ctx.props().language, "Choose a file") }
                    </label>
                    <input type="file" id="input-file" accept={extensions} name="upload-file" style="display: none;"
                        onchange={onchange}
                    />
                </div>
                <div class="input-text-message">
                    { self.view_required_msg(ctx, &my_index) }
                </div>
            </div>
        }
    }

    pub(super) fn view_required_msg(&self, ctx: &Context<Self>, id: &BigUint) -> Html {
        let txt = ctx.props().txt.txt.clone();
        if self.required_msg.contains(id) {
            html! {
                <div class="input-required-message">
                    { text!(txt, ctx.props().language, REQUIRED_MSG) }
                </div>
            }
        } else {
            html! {}
        }
    }

    pub(super) fn view_explanation_msg(ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();
        if let Some(example_message) = &ctx.props().example_message {
            html! {
                <div class="host-network-group-input-input-notice">
                    { text!(txt, ctx.props().language, example_message)}
                </div>
            }
        } else {
            html! {}
        }
    }
}

pub fn view_asterisk(required: bool) -> Html {
    if required {
        html! {
            <div class="input-required-asterisk">
                { "*" }
            </div>
        }
    } else {
        html! {}
    }
}

fn validate_extensions(extensions: &[String]) -> Vec<String> {
    extensions
        .iter()
        .filter_map(|ext| {
            if !ext.contains(' ') && ext.chars().all(|c| c.is_alphanumeric() || c == '.') {
                Some(if ext.starts_with('.') {
                    ext.to_string()
                } else {
                    format!(".{ext}")
                })
            } else {
                None
            }
        })
        .collect()
}
