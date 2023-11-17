use super::{
    component::{InvalidMessage, Message, Model},
    user_input::{view_asterisk, MAX_PER_LAYER},
    InputItem,
};
use crate::{input::component::Verification, text, InputEssential, InputNic, ViewString};
use json_gettext::get_text;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::{classes, events::InputEvent, html, Component, Context, Html};

const INTERFACE_NOTICE: &str = "x.x.x.x/x";
const GATEWAY_NOTICE: &str = "x.x.x.x";

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
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
                            { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
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
}
