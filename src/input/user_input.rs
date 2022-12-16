use super::{
    component::{InvalidMessage, Message, Model},
    InputItem,
};
use crate::{
    input::component::Verification, text, CheckBox, CheckStatus, ChildrenPosition, HostNetworkHtml,
    HostNetworkKind, InputEssential, InputNic, InputType, Item, Radio, SelectSearchable,
    SelectSearchableKind, Tag, ViewString,
};
use gloo_file::File;
use json_gettext::get_text;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yew::{classes, events::InputEvent, html, html::TargetCast, Component, Context, Html};

const PADDING_SUM: u32 = 66; // left + right paddings
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
const INTERFACE_NOTICE: &str = "x.x.x.x/x";
const GATEWAY_NOTICE: &str = "x.x.x.x";
const SELECT_NIC_WIDTH: u32 = 130;
const FLOAT64_STEP_DEFAULT: f64 = 0.1;
pub(super) const MAX_PER_LAYER: usize = 20;

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
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
                            <input type="text" class={class} style={style}
                                value={value}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                autocomplete="off"
                                oninput={oninput}
                                maxlength={length.to_string()}
                            />
                        }
                    } else {
                        html! {
                            <input type="text" class={class} style={style}
                                value={value}
                                placeholder={placeholder}
                                autofocus={autofocus}
                                autocomplete="off"
                                oninput={oninput}
                            />
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

    #[allow(irrefutable_let_patterns)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_select_searchable(
        &self,
        ctx: &Context<Self>,
        multiple: bool,
        ess: &InputEssential,
        width: Option<u32>,
        list: &[(String, ViewString)],
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
        depth: u32,
        group: bool,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let list_clone = Rc::new(list.to_vec());
        let mut list = list
            .iter()
            .map(|(id, value)| Item {
                id: id.clone(),
                value: value.clone(),
                networks: None,
            })
            .collect::<Vec<Item>>();
        list.sort_unstable_by(|a, b| {
            let a_v = a.value.to_string();
            let b_v = b.value.to_string();
            if a_v == b_v {
                a_v.cmp(&b_v)
            } else {
                Ordering::Equal
            }
        });
        let list = Rc::new(RefCell::new(list));

        let top_width = if let Some(width) = width {
            width
        } else if depth > 0 {
            SELECT_NIC_WIDTH
        } else {
            ctx.props().width - PADDING_SUM
        };
        let class_item = if group { "" } else { "input-select-searchable" };
        if let Some(selected) = self
            .select_searchable_buffer
            .get(&(base_index + layer_index))
        {
            html! {
                <div class={class_item}>
                    {
                        if group {
                            html! {}
                        } else {
                            html! {
                                <div class="input-contents-item-general-title">
                                    { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                                </div>
                            }
                        }
                    }
                    {
                        if multiple {
                            html! {
                                <SelectSearchable<Self>
                                    txt={ctx.props().txt.clone()}
                                    language={ctx.props().language}
                                    id={format!("select-searchable-{base_index}-{layer_index}")}
                                    kind={SelectSearchableKind::Multi}
                                    title={ess.title}
                                    empty_msg={ess.notice}
                                    top_width={top_width}
                                    max_height={200}
                                    font="13px 'Spoqa Han Sans Neo'"
                                    list={Rc::clone(&list)}
                                    selected={Rc::clone(selected)}
                                    allow_empty={!ess.required}
                                    parent_message={Some(Message::InputMultipleSelect(base_index + layer_index, input_data.clone(), Rc::clone(&list_clone)))}
                                />
                            }
                        } else {
                            html! {
                                <SelectSearchable<Self>
                                    txt={ctx.props().txt.clone()}
                                    language={ctx.props().language}
                                    id={format!("select-searchable-{base_index}-{layer_index}")}
                                    kind={SelectSearchableKind::Single}
                                    title={ess.title}
                                    empty_msg={ess.notice}
                                    top_width={top_width}
                                    max_height={200}
                                    font="13px 'Spoqa Han Sans Neo'"
                                    list={Rc::clone(&list)}
                                    selected={Rc::clone(selected)}
                                    allow_empty={!ess.required}
                                    parent_message={Some(Message::InputSingleSelect(base_index + layer_index, input_data.clone(), Rc::clone(&list_clone)))}
                                />
                            }
                        }
                    }
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

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_checkbox(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        always: Option<CheckStatus>,
        children: &Option<(ChildrenPosition, Vec<Rc<InputType>>)>,
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
        both_border: Option<bool>,
        depth: u32,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let input_data_msg = input_data.clone();
        let onclick = ctx
            .link()
            .callback(move |_| Message::ClickCheckBox(input_data_msg.clone()));
        let checked = if let Ok(data) = input_data.try_borrow() {
            if let InputItem::CheckBox(checked, _) = (*data).clone() {
                Some(checked)
            } else {
                None
            }
        } else {
            None
        };
        let class = both_border.map_or("input-checkbox", |both| {
            if both {
                "input-checkbox-both"
            } else {
                "input-checkbox-top"
            }
        });
        let (class_align, class_me, class_child) = children.as_ref().map_or(
            (
                "input-checkbox-children-nextline",
                "input-checkbox-me-nextline",
                "input-checkbox-child",
            ),
            |c| match c.0 {
                ChildrenPosition::NextLine => (
                    "input-checkbox-children-nextline",
                    "input-checkbox-me-nextline",
                    "input-checkbox-child",
                ),
                ChildrenPosition::Right => (
                    "input-checkbox-children-right",
                    "input-checkbox-me-right",
                    "input-checkbox-child-right",
                ),
            },
        );

        if let Some(checked) = checked {
            html! {
                <div class={class}>
                    <div class={class_align}>
                    {
                        if always == Some(CheckStatus::Checked) || always == Some(CheckStatus::Unchecked) {
                            html! {
                                <div class={classes!("input-checkbox-me", class_me)}>
                                    <CheckBox
                                        status={checked}
                                        always={always}
                                    />
                                    <div class="input-checkbox-me-title">
                                        { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                                    </div>
                                </div>
                            }
                        } else {
                            html! {
                                <div class={classes!("input-checkbox-me", class_me)}>
                                    <div class="input-checkbox-me-checkbox" onclick={onclick}>
                                        <CheckBox
                                            status={checked}
                                        />
                                    </div>
                                    <div class="input-checkbox-me-title">
                                        { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                                    </div>
                                </div>
                            }
                        }
                    }
                        <div class="input-checkbox-children">
                        {
                            if checked == CheckStatus::Unchecked {
                                html! {}
                            } else if let (Some(children), Ok(input_data)) = (children, input_data.try_borrow()) {
                                html! {
                                    for children.1.iter().enumerate().map(|(sub_index, child)| {
                                        let child_data = if let InputItem::CheckBox(_, childs) = input_data.clone() {
                                            childs.and_then(|childs| childs.get(sub_index).map(Rc::clone))
                                        } else {
                                            None
                                        };
                                        let class_line = if children.0 == ChildrenPosition::Right {
                                            if sub_index == 0 {
                                                "input-checkbox-link-line-right"
                                            } else {
                                                "input-checkbox-link-line"
                                            }
                                        } else {
                                            "input-checkbox-link-line"
                                        };
                                        if let Some(child_data) = child_data {
                                            match &**child {
                                                InputType::CheckBox(ess, always, children) => {
                                                    html! {
                                                        <div class={class_child}>
                                                            <div class={class_line}>
                                                            </div>
                                                            { self.view_checkbox(ctx, ess, *always, children, &child_data, sub_index, (base_index + layer_index) * MAX_PER_LAYER, None, depth + 1) }
                                                        </div>
                                                    }
                                                }
                                                InputType::HostNetworkGroup(ess, kind, num, width) => {
                                                    html! {
                                                        <div class={class_child}>
                                                            <div class={class_line}>
                                                            </div>
                                                            { self.view_host_network_group(ctx, ess, *kind, *num, *width, &child_data, sub_index, (base_index + layer_index) * MAX_PER_LAYER) }
                                                        </div>
                                                    }
                                                }
                                                InputType::Unsigned32(ess, min, max, width) => {
                                                    html! {
                                                        <div class={class_child}>
                                                            <div class={class_line}>
                                                            </div>
                                                            { self.view_unsigned_32(ctx, ess, *min, *max, *width, &child_data, sub_index, (base_index + layer_index) * MAX_PER_LAYER, false, false) }
                                                        </div>
                                                    }
                                                }
                                                InputType::SelectMultiple(ess, list, nics, _, _) => {
                                                    html! {
                                                        <div class={class_child}>
                                                            <div class={class_line}>
                                                            </div>
                                                            { self.view_select_nic_or(ctx, list, *nics, ess, &child_data, sub_index, (base_index + layer_index) * MAX_PER_LAYER, depth) }
                                                        </div>
                                                    }
                                                }
                                                _ => html! {}
                                            }
                                        } else {
                                            html! {}
                                        }
                                    })
                                }
                            } else {
                                html! {}
                            }
                        }
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }

    pub(super) fn view_nic(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let input_data_clone = input_data.clone();

        if let Ok(input_data) = input_data.try_borrow() {
            if let InputItem::Nic(input_data) = &*input_data {
                let num = input_data.len();
                html! {
                    <div class="input-item">
                        <div class="input-contents-item-title">
                            { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                        </div>
                        <table class="input-nic">
                            <tr>
                                <th class={classes!("input-nic-heading", "input-nic-heading-name")}>
                                    { text!(txt, ctx.props().language, "Interface Name") }
                                </th>
                                <th class={classes!("input-nic-heading", "input-nic-border", "input-nic-heading-ip")}>
                                    { text!(txt, ctx.props().language, "IP Address of Interface") }
                                </th>
                                <th class={classes!("input-nic-heading", "input-nic-border", "input-nic-heading-ip")}>
                                    { text!(txt, ctx.props().language, "IP Address of Gateway") }
                                </th>
                                <th class="input-nic-heading-delete">
                                </th>
                                <th class="input-nic-heading-add">
                                </th>
                            </tr>

                        {
                            for input_data.iter().enumerate().map(|(index, d)| {
                                self.view_nic_each(ctx, &input_data_clone, index, layer_index, base_index, index + 1 == num, d)
                            })
                        }
                        </table>
                        { self.view_required_msg(ctx, base_index + layer_index) }
                    </div>
                }
            } else {
                html! {}
            }
        } else {
            html! {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_select_nic_or(
        &self,
        ctx: &Context<Self>,
        list: &Option<Vec<(String, ViewString)>>,
        nics: Option<usize>,
        ess: &InputEssential,
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
        depth: u32,
    ) -> Html {
        match (list, nics) {
            (Some(list), None) => self.view_select_searchable(
                ctx,
                true,
                ess,
                None,
                list,
                input_data,
                layer_index,
                base_index,
                depth,
                false,
            ),
            (None, Some(nics)) => {
                let list = if let Some(nics) = ctx.props().input_data.get(nics) {
                    if let Ok(nics) = nics.try_borrow() {
                        if let InputItem::Nic(nics) = &*nics {
                            Some(
                                nics.iter()
                                    .filter_map(|nics| {
                                        if nics.name.is_empty() {
                                            None
                                        } else {
                                            Some((
                                                nics.name.clone(),
                                                ViewString::Raw(nics.name.clone()),
                                            ))
                                        }
                                    })
                                    .collect::<Vec<(String, ViewString)>>(),
                            )
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                if let Some(list) = list {
                    self.view_select_searchable(
                        ctx,
                        true,
                        ess,
                        None,
                        &list,
                        input_data,
                        layer_index,
                        base_index,
                        depth,
                        false,
                    )
                } else {
                    html! {}
                }
            }
            _ => html! {},
        }
    }

    #[allow(clippy::too_many_lines)]
    #[allow(clippy::too_many_arguments)]
    fn view_nic_each(
        &self,
        ctx: &Context<Self>,
        input_data: &Rc<RefCell<InputItem>>,
        nic_index: usize,
        layer_index: usize,
        base_index: usize,
        is_last: bool,
        nic: &InputNic,
    ) -> Html {
        let input_data_clone_1 = input_data.clone();
        let input_data_clone_2 = input_data.clone();
        let input_data_clone_3 = input_data.clone();
        let input_data_clone_4 = input_data.clone();
        let input_data_clone_5 = input_data.clone();

        let oninput_name = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputNicName(
                        base_index + layer_index,
                        nic_index,
                        input.value(),
                        input_data_clone_1.clone(),
                    )
                })
        });
        let oninput_interface = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputNicInterface(
                        base_index + layer_index,
                        nic_index,
                        input.value(),
                        input_data_clone_2.clone(),
                    )
                })
        });
        let oninput_gateway = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputNicGateway(
                        base_index + layer_index,
                        nic_index,
                        input.value(),
                        input_data_clone_3.clone(),
                    )
                })
        });
        let onclick_delete = ctx.link().callback(move |_| {
            Message::InputNicDelete(
                base_index + layer_index,
                nic_index,
                input_data_clone_4.clone(),
            )
        });
        let onclick_add = ctx.link().callback(move |_| {
            Message::InputNicAdd(base_index, layer_index, input_data_clone_5.clone())
        });
        let txt = ctx.props().txt.txt.clone();
        let name_holder = text!(txt, ctx.props().language, "Name").to_string();

        let (name_msg, interface_msg, gateway_msg) = (
            self.verification_nic
                .get(&((base_index + layer_index) * MAX_PER_LAYER + nic_index, 0)),
            self.verification_nic
                .get(&((base_index + layer_index) * MAX_PER_LAYER + nic_index, 1)),
            self.verification_nic
                .get(&((base_index + layer_index) * MAX_PER_LAYER + nic_index, 2)),
        );
        let name_msg =
            if let Some(Verification::Invalid(InvalidMessage::InterfaceNameRequired)) = name_msg {
                Some("Required")
            } else {
                None
            };
        let interface_msg = if let Some(Verification::Invalid(InvalidMessage::InterfaceRequired)) =
            interface_msg
        {
            Some("Required")
        } else if let Some(Verification::Invalid(InvalidMessage::WrongInterface)) = interface_msg {
            Some("Wrong input")
        } else {
            None
        };
        let gateway_msg =
            if let Some(Verification::Invalid(InvalidMessage::GatewayRequired)) = gateway_msg {
                Some("Required")
            } else if let Some(Verification::Invalid(InvalidMessage::WrongGateway)) = gateway_msg {
                Some("Wrong input")
            } else {
                None
            };
        let msg = name_msg.is_some() || interface_msg.is_some() || gateway_msg.is_some();
        let (class, class_delete) = if is_last {
            ("input-nic-input-last", "input-nic-delete-last")
        } else {
            ("input-nic-input", "input-nic-delete")
        };

        html! {
            <>
                <tr>
                    <td class={class}>
                        <div class="input-nic-input-outer">
                            <div class="input-nic-input-name">
                                <input type="text"
                                    class={classes!("input-nic", "input-nic-name")}
                                    value={nic.name.clone()}
                                    placeholder={name_holder}
                                    oninput={oninput_name}
                                />
                            {
                                if msg {
                                    html! {
                                        <div class="input-nic-msg">
                                            { name_msg.map_or_else(String::new, |m| text!(txt, ctx.props().language, m).to_string()) }
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            </div>
                        </div>
                    </td>
                    <td class={class}>
                        <div class="input-nic-input-outer">
                            <div class="input-nic-input-interface">
                                <input type="text"
                                    class={classes!("input-nic", "input-nic-interface")}
                                    value={nic.interface.clone()}
                                    placeholder={INTERFACE_NOTICE}
                                    oninput={oninput_interface}
                                />
                            {
                                if msg {
                                    html! {
                                        <div class="input-nic-msg">
                                            { interface_msg.map_or_else(String::new, |m| text!(txt, ctx.props().language, m).to_string()) }
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            </div>
                        </div>
                    </td>
                    <td class={class}>
                        <div class="input-nic-input-outer">
                            <div class="input-nic-input-gateway">
                                <input type="text"
                                    class={classes!("input-nic", "input-nic-gateway")}
                                    placeholder={GATEWAY_NOTICE}
                                    value={nic.gateway.clone()}
                                    oninput={oninput_gateway}
                                />
                                {
                                    if msg {
                                        html! {
                                            <div class="input-nic-msg">
                                                { gateway_msg.map_or_else(String::new, |m| text!(txt, ctx.props().language, m).to_string()) }
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </div>
                    </td>
                    <td class={class_delete}>
                        <div class="input-nic-delete-outer">
                            <div class="input-nic-delete" onclick={onclick_delete}>
                            </div>
                        </div>
                    </td>
                    <td class="input-nic-input-add">
                    {
                        if is_last {
                            html! {
                                <div class="input-add-item" onclick={onclick_add}>
                                    { text!(txt, ctx.props().language, "+ Add") }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                    </td>
                </tr>
            </>
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
                        disabled={true} />
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

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines)]
    pub(super) fn view_group(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        one_row: bool,
        widths: &[Option<u32>],
        group_type: &[Rc<InputType>],
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
    ) -> Html {
        let input_data_clone = input_data.clone();
        let input_data_clone_1 = &(input_data.clone());
        let Ok(input_data) = input_data.try_borrow() else {
            return html! {}
        };
        let InputItem::Group(input_data) = &*input_data else {
            return html! {};
        };
        let txt = ctx.props().txt.txt.clone();
        let sub_base_index = (base_index + layer_index) * MAX_PER_LAYER;
        let default = ess.default.clone();
        let onclick_add = ctx.link().callback(move |_| {
            Message::InputGroupAdd(sub_base_index, input_data_clone.clone(), default.clone())
        });

        html! {
            <div class="input-item">
                <div class="input-contents-item-title">
                    { text!(txt, ctx.props().language, ess.title) }{ view_asterisk(ess.required) }
                </div>
                <div class="input-group">
                    <div>
                        <table class="input-group">
                            <tr>
                                {
                                    for group_type.iter().enumerate().map(|(col_index, each)| {
                                        let style = if let Some(Some(width)) = widths.get(col_index) {
                                            format!("width: {}px;", *width)
                                        } else {
                                            String::new()
                                        };
                                        html! {
                                            <th class="input-group-heading" style={style}>
                                                { text!(txt, ctx.props().language, each.title()) }{ view_asterisk(each.required()) }
                                            </th>
                                        }
                                    })
                                }
                                <th class="input-group-heading-delete">
                                </th>
                            </tr>
                            {
                                for input_data.iter().enumerate().map(|(row_index, row)| {
                                    let input_data_clone_1 = input_data_clone_1.clone();
                                    let default = ess.default.clone();
                                    let onclick_delete = ctx.link().callback(move |_| {
                                        Message::InputGroupDelete(
                                            sub_base_index,
                                            row_index,
                                            input_data_clone_1.clone(),
                                            default.clone(),
                                        )
                                    });

                                    if one_row {
                                        html! {
                                            <tr>
                                                {
                                                    for group_type.iter().enumerate().map(|(col_index, each)| {
                                                        let Some(input_data) = row.get(col_index) else {
                                                            return html! {};
                                                        };
                                                        let base_index = (row_index + sub_base_index) * MAX_PER_LAYER;
                                                        html! {
                                                            <td class="input-group">
                                                                <div class="input-group-item-outer">
                                                                {
                                                                    match &**each {
                                                                        InputType::Text(ess, length, width) =>
                                                                            self.view_text(ctx, ess, *length, *width, input_data, col_index, base_index, false, true),
                                                                        InputType::SelectSingle(ess, list, width) => {
                                                                            self.view_select_searchable(ctx, false, ess, *width, list, input_data, col_index, base_index, 1, true)
                                                                        }
                                                                        InputType::Unsigned32(ess, min, max, width) => {
                                                                            self.view_unsigned_32(ctx, ess, *min, *max, *width, input_data, col_index, base_index, false, true)
                                                                        }
                                                                        InputType::Float64(ess, step, width) => {
                                                                            self.view_float_64(ctx, ess, *step, *width, input_data, col_index, base_index, false, true)
                                                                        }
                                                                        _ => html! {}
                                                                    }
                                                                }
                                                                </div>
                                                            </td>
                                                        }
                                                    })
                                                }
                                                <td class="input-group-delete">
                                                    <div class="input-nic-delete-outer">
                                                        <div class="input-nic-delete" onclick={onclick_delete}>
                                                        </div>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    } else {
                                        // TODO: implement in the case of !one_row
                                        html! {}
                                    }
                                })
                            }
                        </table>
                    </div>
                    <div class="input-group-add">
                        <div class="input-add-item" onclick={onclick_add}>
                            { text!(txt, ctx.props().language, "+ Add") }
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    fn view_required_msg(&self, ctx: &Context<Self>, id: usize) -> Html {
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
}

fn view_asterisk(required: bool) -> Html {
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
