use super::{
    component::{InvalidMessage, Message, Model},
    InputItem,
};
use crate::{
    input::component::Verification, text, HostNetworkHtml, HostNetworkKind, InputEssential, Radio,
    Tag, ViewString,
};
use gloo_file::File;
use json_gettext::get_text;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yew::{events::InputEvent, html, html::TargetCast, Component, Context, Html};

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
    "Your password must not contain consecutive repetitive characters.";
const PASSWD_ADJACENT_MSG: &str =
    "Your password must not contain more than 3 adjacent keyboard characters.";
const PASSWD_REQUIREMENT: &str = if cfg!(feature = "cc-password") {
    "no spaces, more than 8 characters, at least one number/uppercase/lowercase/special characters, no consecutive repetition, and less than 4 adjacent keyboard characters"
} else {
    "no spaces, more than 7 characters, at least one number/uppercase/lowercase/special characters"
};
const FLOAT64_STEP_DEFAULT: f64 = 0.1;
pub(super) const MAX_PER_LAYER: usize = 20;

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
        layer_index: usize,
        base_index: usize,
        autofocus: bool,
        group: bool,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputText(
                        base_index + layer_index,
                        input.value(),
                        input_data_clone.clone(),
                    )
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Text(txt) = &(*input_data) {
                txt.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let class = if self.required_msg.contains(&(base_index + layer_index)) {
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

        html! {
            <div class={class_item}>
                {
                    if group {
                        html! {}
                    } else {
                        html! {
                            <div class="input-contents-item-title">
                                { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                            </div>
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
                                />
                                { Self::view_explanation_msg(ctx)}
                            </>
                        }
                    }
                }
                <div class="input-text-message">
                    { self.view_required_msg(ctx, base_index + layer_index) }
                </div>
                {
                    if self.unique_msg.contains(&(base_index + layer_index)) {
                        html! {
                            <div class="input-contents-item-alert-message">
                                { text!(txt, ctx.props().language, EXISTING_MSG)}
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
        width: Option<u32>,
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
        autofocus: bool,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputPassword(
                        base_index + layer_index,
                        input.value(),
                        input_data_clone.clone(),
                    )
                })
        });
        let oninput_confirm = |index: usize| {
            ctx.link().callback(move |e: InputEvent| {
                e.target()
                    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                    .map_or(Message::InputError, |input| {
                        Message::InputConfirmPassword(index, input.value())
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

        let class = if self.required_msg.contains(&(base_index + layer_index)) {
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
                    { text!(txt, ctx.props().language, ess.title) }
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
                    oninput={oninput_confirm(base_index + layer_index)}
                />
                <div class="input-text-message">
                    { self.view_required_msg(ctx, base_index + layer_index) }
                </div>
                {
                    if let Some(Verification::Invalid(m)) = self.verification.get(&(base_index + layer_index)) {
                        let msg = match m {
                            InvalidMessage::PasswordHasSpace => Some(PASSWD_HAS_SPACE_MSG),
                            InvalidMessage::PasswordHasControlCharacter => Some(PASSWD_HAS_CONTROL_CHARACTER_MSG),
                            InvalidMessage::PasswordNotMatch => Some(PASSWD_NOT_MATCH_MSG),
                            InvalidMessage::PasswordTooShort => Some(PASSWD_TOO_SHORT_MSG),
                            InvalidMessage::PasswordNoLowercaseLetter => Some(PASSWD_NO_LOWER_MSG),
                            InvalidMessage::PasswordNoUppercaseLetter => Some(PASSWD_NO_UPPER_MSG),
                            InvalidMessage::PasswordNoNumber => Some(PASSWD_NO_NUMBER_MSG),
                            InvalidMessage::PasswordNoSymbol => Some(PASSWD_NO_SYMBOL_MSG),
                            InvalidMessage::PasswordHasConsecutiveLetters => Some(PASSWD_CONSECUTIVE_MSG),
                            InvalidMessage::PasswordHasAdjacentLetters => Some(PASSWD_ADJACENT_MSG),
                            _ => None,
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
        layer_index: usize,
        base_index: usize,
        autofocus: bool,
        group: bool,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    if let Ok(value) = input.value().parse::<u32>() {
                        Message::InputUnsigned32(
                            base_index + layer_index,
                            value,
                            input_data_clone.clone(),
                        )
                    } else {
                        Message::InvalidInputUnsigned32
                    }
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Unsigned32(value) = *input_data {
                value
            } else {
                None
            }
        } else {
            None
        };
        let class = if self.required_msg.contains(&(base_index + layer_index)) {
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
                {
                    if group {
                        html! {}
                    } else {
                        html! {
                            <div class="input-contents-item-title">
                                { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
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
                    { self.view_required_msg(ctx, base_index + layer_index) }
                </div>
                {
                    if self.unique_msg.contains(&(base_index + layer_index)) {
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
                    if let Some(Verification::Invalid(InvalidMessage::InvalidInput)) = self.verification.get(&(base_index + layer_index)) {
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
        layer_index: usize,
        base_index: usize,
        autofocus: bool,
        group: bool,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    if let Ok(value) = input.value().parse::<f64>() {
                        Message::InputFloat64(
                            base_index + layer_index,
                            value,
                            input_data_clone.clone(),
                        )
                    } else {
                        Message::InvalidInputFloat64
                    }
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Float64(value) = *input_data {
                value
            } else {
                None
            }
        } else {
            None
        };
        let class = if self.required_msg.contains(&(base_index + layer_index)) {
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
        let step = step.unwrap_or(FLOAT64_STEP_DEFAULT);

        html! {
            <div class={class_item}>
            {
                if group {
                    html! {}
                } else {
                    html! {
                        <div class="input-contents-item-title">
                            { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
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
                                step={step.to_string()}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                oninput={oninput}
                            />
                        }
                    } else {
                        html! {
                            <input type="number" class={class} style={style}
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
                    { self.view_required_msg(ctx, base_index + layer_index) }
                </div>
                {
                    if self.unique_msg.contains(&(base_index + layer_index)) {
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
                    if let Some(Verification::Invalid(InvalidMessage::InvalidInput)) = self.verification.get(&(base_index + layer_index)) {
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
        layer_index: usize,
        base_index: usize,
        autofocus: bool,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    if let Ok(value) = input.value().parse::<f32>() {
                        Message::InputPercentage(
                            base_index + layer_index,
                            value / 100.0,
                            input_data_clone.clone(),
                        )
                    } else {
                        Message::InvalidInputPercentage
                    }
                })
        });
        let placeholder = text!(txt, ctx.props().language, ess.notice).to_string();
        let value = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Percentage(value) = *input_data {
                value.map(|v| v * 100.0)
            } else {
                None
            }
        } else {
            None
        };

        let class = if self.required_msg.contains(&(base_index + layer_index)) {
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
                    { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
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
                    { self.view_required_msg(ctx, base_index + layer_index) }
                </div>
                {
                    if self.unique_msg.contains(&(base_index + layer_index)) {
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
                    if let Some(Verification::Invalid(InvalidMessage::InvalidInput)) = self.verification.get(&(base_index + layer_index)) {
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

    pub(super) fn view_radio(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        options: &[ViewString],
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
    ) -> Html {
        let list = Rc::new(options.to_vec());
        let candidates = Rc::new(
            list.iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );
        let txt = ctx.props().txt.txt.clone();
        if let Some(buffer) = self.radio_buffer.get(&(base_index + layer_index)) {
            html! {
                <div class="input-radio-outer">
                    <div class="input-radio">
                        <div class="input-radio-title">
                            { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                        </div>
                        <div class="input-radio-radio">
                            <Radio::<Self>
                                txt={ctx.props().txt.clone()}
                                language={ctx.props().language}
                                parent_message={Some(Message::InputRadio(base_index + layer_index, input_data.clone()))}
                                list={Rc::clone(&list)}
                                candidate_values={Rc::clone(&candidates)}
                                selected_value={Rc::clone(buffer)}
                            />
                            {
                                if ess.notice.is_empty() {
                                    html! {}
                                } else {
                                    html! {
                                        <div class="input-radio-notice">
                                            { text!(txt, ctx.props().language, ess.notice) }
                                        </div>
                                    }
                                }
                            }
                        </div>
                    </div>
                    <div class="input-radio-message">
                        { self.view_required_msg(ctx, base_index + layer_index) }
                    </div>
                </div>
            }
        } else {
            html! {}
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
        layer_index: usize,
        base_index: usize,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        if let Some(buffer) = self.host_network_buffer.get(&(base_index + layer_index)) {
            html! {
                <div class="input-host-network-group">
                    <div class="input-contents-item-general-title">
                        { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                    </div>
                    <HostNetworkHtml<Self>
                        txt={ctx.props().txt.clone()}
                        language={ctx.props().language}
                        rerender_serial={self.rerender_serial_host_network}
                        kind={kind}
                        num={num}
                        width={width}
                        input_data={Rc::clone(buffer)}
                        input_notice={Some(ess.notice)}
                        parent_message={Some(Message::InputHostNetworkGroup(base_index + layer_index, input_data.clone()))}
                        parent_message_save={Some(Message::RightHostNetworkGroup(base_index + layer_index, input_data.clone()))}
                        parent_message_no_save={Some(Message::WrongHostNetworkGroup(base_index + layer_index))}
                        parent_message_user_input={Some(Message::UserInputHostNetworkGroup(base_index + layer_index))}
                        verify_to_save={self.verify_host_network_group}
                    />
                    { self.view_required_msg(ctx, base_index + layer_index) }
                </div>
            }
        } else {
            html! {}
        }
    }

    pub(super) fn view_tag_group(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        prev_list: &HashMap<String, String>,
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        if let Some(buffer) = self.tag_buffer.get(&(base_index + layer_index)) {
            let prev_list = Rc::new(prev_list.clone());
            html! {
                <div class="input-tag-group">
                    <div class="input-contents-item-general-title">
                        { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                    </div>
                    <Tag<Self>
                        txt={ctx.props().txt.clone()}
                        language={ctx.props().language}
                        prev_list={Rc::clone(&prev_list)}
                        input_data={Rc::clone(buffer)}
                        input_notice={Some(ess.notice)}
                        parent_message={Some(Message::InputTagGroup(base_index + layer_index, input_data.clone()))}
                    />
                    { self.view_required_msg(ctx, base_index + layer_index) }
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
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
    ) -> Html {
        let input_data_clone = input_data.clone();
        let onchange = ctx.link().callback(move |e: Event| {
            let mut result = Vec::new();
            let input: HtmlInputElement = e.target_unchecked_into();
            // TODO: the below `expect` is inevitable? Refer the below example code.
            // if let Some(files) = input.files() {
            //     let files = js_sys::try_iter(&files)
            //         .unwrap()
            //         .unwrap()
            //         .map(|v| web_sys::File::from(v.unwrap()))
            //         .map(File::from);
            //     result.extend(files);
            // }
            if let Some(files) = input.files() {
                let files = js_sys::try_iter(&files).ok().and_then(|x| x);
                if let Some(files) = files {
                    let files = files
                        .map(|v| web_sys::File::from(v.expect("convert to JsValue")))
                        .map(File::from);
                    result.extend(files);
                }
            }
            Message::ChooseFile(base_index + layer_index, result, input_data_clone.clone())
        });
        let txt = ctx.props().txt.txt.clone();
        let file_name = if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::File(name, _) = &(*input_data) {
                name.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        html! {
            <div class="input-item">
                <div class="input-contents-item-title">
                    { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                </div>
                <div class="input-file">
                    <input class="input-file-file-name"
                        value={file_name}
                        readonly={true} />
                    <label for="input-file" class="input-file-choose-file">
                        { text!(txt, ctx.props().language, "Choose a file") }
                    </label>

                    <input type="file" id="input-file" accept=".aice" name="upload-file" style="display: none;"
                        onchange={onchange}
                    />
                </div>
                <div class="input-text-message">
                    { self.view_required_msg(ctx, base_index + layer_index) }
                </div>
            </div>
        }
    }

    pub(super) fn view_required_msg(&self, ctx: &Context<Self>, id: usize) -> Html {
        let txt = ctx.props().txt.txt.clone();
        if self.required_msg.contains(&id) {
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

#[must_use]
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
