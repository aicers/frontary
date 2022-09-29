use super::{Message, Model, MIN_POP_HEIGHT};
use crate::{
    text, window_inner_height, CheckBox, CheckStatus, EndpointKind, Item, SelectComplexKind,
    SelectMini, SelectMiniKind, SelectionExtraInfo, ViewString, NBSP,
};
use htmlescape::decode_html;
use json_gettext::get_text;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::{events::InputEvent, html, Context, Html};

impl Model {
    pub(super) fn view_pop(&self, ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let style_all = match self.check_status(ctx, false) {
            CheckStatus::Checked => "background-image: url('/img/radio-opener-checked.png');",
            _ => "background-image: url('/img/radio-opener-unchecked.png');",
        };
        let style = format!("width: {}px;", ctx.props().pop_width);
        let style_pop = format!(
            "widht: {}px; height: {}px;",
            ctx.props().pop_width,
            std::cmp::max(MIN_POP_HEIGHT, window_inner_height())
        );
        let style_head_title = format!("width: {}px;", ctx.props().pop_width - 34);
        let style_list = if self.view_list {
            "background-image: url('/img/collapse-contents.png');"
        } else {
            "background-image: url('/img/expand-contents.png');"
        };
        let style_input = if self.view_input {
            "background-image: url('/img/collapse-contents.png');"
        } else {
            "background-image: url('/img/expand-contents.png');"
        };
        let class_input_head = if self.view_list {
            "complex-select-pop-input-head-bottom"
        } else {
            "complex-select-pop-input-head"
        };

        let onclick_input = ctx.link().callback(|_| Message::ToggleInput);
        let onclick_list = ctx.link().callback(|_| Message::ToggleList);
        let onclick_all = ctx.link().callback(|_| Message::ClickAll);
        let onclick_close = ctx.link().callback(|_| Message::Close);

        html! {
            <div id={ctx.props().id.clone()} class="complex-select-pop" style={style_pop}>
                <div class="complex-select-pop-head">
                    <table>
                        <tr style={style.clone()}>
                            <td class="complex-select-pop-head-text" style={style_head_title}>
                                { text!(txt, ctx.props().language, &ctx.props().title) }
                                { decode_html(NBSP).expect("safely-selected character") }
                                <font style="color: #B5131A;">
                                    { "(" } { Self::selected_len(ctx) } { ")" }
                                </font>
                            </td>
                            <td class="complex-select-pop-head-close" onclick={onclick_close}>
                                <div class="complex-select-pop-head-close-icon">
                                </div>
                            </td>
                        </tr>
                    </table>
                </div>
                // All for the entire
                <div class="complex-select-pop-all" style={style.clone()} >
                    <div class="complex-select-pop-all-text">
                        { text!(txt, ctx.props().language, "Select All") }
                    </div>
                    <div class="complex-select-pop-all-button" style={style_all} onclick={onclick_all}>
                    </div>
                </div>
                <div class="complex-select-pop-list-head">
                    <table>
                        <tr onclick={onclick_list}>
                            <td class="complex-select-pop-head-1st">
                                { text!(txt, ctx.props().language, "Choose ones (in the list)") }
                            </td>
                            <td class="complex-select-pop-head-2nd" style={style_list}>
                            </td>
                        </tr>
                    </table>
                </div>
                {
                    if self.view_list {
                        self.view_list(ctx)
                    } else {
                        html! {}
                    }
                }
                <div class={class_input_head}>
                    <table>
                        <tr onclick={onclick_input}>
                            <td class="complex-select-pop-head-1st">
                                { text!(txt, ctx.props().language, "Input yourself") }
                            </td>
                            <td class="complex-select-pop-head-2nd" style={style_input}>
                            </td>
                        </tr>
                    </table>
                </div>
                {
                    if self.view_input {
                        self.view_input(ctx)
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

    fn view_list(&self, ctx: &Context<Self>) -> Html {
        let style = format!("width: {}px;", ctx.props().pop_width);
        let style_width_search = format!("width: {}px", ctx.props().pop_width - 48);
        let style_pop_list = format!(
            "width: {}px; height: {}px",
            ctx.props().pop_width,
            std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 202
        );
        let style_pop_list_list = format!(
            "width: {}px; height: {}px",
            ctx.props().pop_width,
            std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 244
        );
        let style_pop_list_list_items = format!(
            "width: {}px; height: {}px",
            ctx.props().pop_width,
            std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 286
        );
        let txt = ctx.props().txt.txt.clone();
        let search_notice = text!(txt, ctx.props().language, "Search").to_string();
        let oninput_search = ctx.link().callback(|e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputSearch(input.value())
                })
        });
        let check_status = if self.search_result.is_none() {
            self.check_status(ctx, false)
        } else {
            self.check_status(ctx, true)
        };

        html! {
            <div class="complex-select-pop-list" style={style_pop_list}>
                <div class="complex-select-pop-list-search" style={style}>
                    <input type="text" class="complex-select-search"
                        placeholder={search_notice}
                        style={style_width_search}
                        oninput={oninput_search}
                        value={self.search_text.clone()}
                    />
                </div>
                <div class="complex-select-pop-list-list" style={style_pop_list_list}>
                {
                    match ctx.props().kind {
                        SelectComplexKind::NetworkIp => {
                            let onclick_all = ctx.link().callback(move |_| Message::ClickAllBelow);
                            html! {
                                <div class="complex-select-pop-list-list-all">
                                    <table class="complex-select-pop-list-list-all">
                                        <tr style="position: relative;">
                                            <td class="complex-select-pop-list-list-all-checkbox">
                                                // All for the below list
                                                <div onclick={onclick_all}>
                                                    <CheckBox status={check_status} />
                                                </div>
                                            </td>
                                            <td class="complex-select-pop-list-list-all-item">
                                                { text!(txt, ctx.props().language, "Select All") }
                                            </td>
                                            <td>
                                            { self.view_direction(ctx) }
                                            </td>
                                        </tr>
                                    </table>
                                </div>
                            }
                        },
                        SelectComplexKind::Basic => html! {},
                    }
                }
                    <div class="complex-select-pop-list-list-items" style={style_pop_list_list_items}>
                    {
                        if let Ok(list) = ctx.props().list.try_borrow() {
                            if let Some(search) = self.search_result.as_ref() {
                                html! {
                                    for search.iter().map(|&index| {
                                        if let Some(item) = list.get(index) {
                                            self.view_list_item(ctx, item)
                                        } else {
                                            html! {}
                                        }
                                    })
                                }
                            } else {
                                html! {
                                    for list.iter().map(|item| self.view_list_item(ctx, item))
                                }
                            }
                        } else {
                            html! {}
                        }
                    }
                    </div>
                </div>
            </div>
        }
    }

    fn view_direction(&self, ctx: &Context<Self>) -> Html {
        let direction_list = Rc::new(vec![
            ViewString::Key("Set the selected to both".to_string()),
            ViewString::Key("Set the selected to sources".to_string()),
            ViewString::Key("Set the selected to destinations".to_string()),
        ]);
        let value_candidates = Rc::new(vec![
            EndpointKind::Both,
            EndpointKind::Source,
            EndpointKind::Destination,
        ]);
        let check_status = if self.search_result.is_some() {
            self.check_status(ctx, true)
        } else {
            self.check_status(ctx, false)
        };
        let active = match check_status {
            CheckStatus::Checked | CheckStatus::Indeterminate => true,
            CheckStatus::Unchecked => false,
        };
        html! {
            <SelectMini::<EndpointKind, Self>
                txt={ctx.props().txt.clone()}
                language={ctx.props().language}
                parent_message={Message::SetDirection}
                active={active}
                deactive_class_suffix={Some("-deactive".to_string())}
                id="assign-direction"
                list={direction_list}
                candidate_values={value_candidates}
                selected_value={self.direction.clone()}
                selected_value_cache={self.direction.try_borrow().ok().and_then(|x| *x)}
                align_left={false}
                list_top={22}
                kind={SelectMiniKind::DirectionAll}
            />
        }
    }

    fn view_list_item(&self, ctx: &Context<Self>, item: &Item) -> Html {
        let (key, checked) = if item.networks().is_some() {
            (
                item.id.clone(),
                self.direction_items
                    .get(&item.id)
                    .map_or(CheckStatus::Unchecked, |extra| {
                        if let Ok(extra) = extra.try_borrow() {
                            match *extra {
                                Some(SelectionExtraInfo::Network(EndpointKind::Both)) => {
                                    CheckStatus::Checked
                                }
                                None => CheckStatus::Unchecked,
                                _ => CheckStatus::Indeterminate,
                            }
                        } else {
                            CheckStatus::Unchecked
                        }
                    }),
            )
        } else {
            (String::new(), CheckStatus::Unchecked) // Item::KeyString -> unreachable
        };
        let onclick_item = |key: String| {
            ctx.link()
                .callback(move |_| Message::ClickItem(key.clone()))
        };
        let style_item_width = match ctx.props().kind {
            SelectComplexKind::NetworkIp => "width: 209px;",
            SelectComplexKind::Basic => "width: 279px",
        };

        html! {
            <table>
                <tr>
                    <td class="complex-select-pop-list-list-items-checkbox">
                        <div onclick={onclick_item(key)}>
                            <CheckBox status={checked} />
                        </div>
                    </td>
                    <td class="complex-select-pop-list-list-items-item" style={style_item_width}>
                    {
                        if let Some(networks)=item.networks() {
                                html! {
                                    <>
                                        { item.value().to_string() } <br/>
                                        <div class="complex-select-pop-list-networks">
                                        {
                                            for networks.hosts.iter().map(|host| html! {
                                                <>
                                                    { host } <br/>
                                                </>
                                            })
                                        }
                                        {
                                            for networks.networks.iter().map(|nt| html! {
                                                <>
                                                    { nt } <br/>
                                                </>
                                            })
                                        }
                                        {
                                            for networks.ranges.iter().map(|r| html! {
                                                <>
                                                    { r.start.clone() } { " ~ " } { r.end.clone() } <br/>
                                                </>
                                            })
                                        }
                                        </div>
                                    </>
                                }
                        } else {
                            html!{}
                        }
                    }
                    </td>
                    {
                            if item.networks.is_some(){
                                html! {
                                    <td class="complex-select-pop-list-list-items-direction">
                                        { self.view_network_ip_item_direction(ctx, item.id(), checked == CheckStatus::Checked || checked == CheckStatus::Indeterminate) }
                                    </td>
                                }
                            } else {
                                html! {}
                            }
                    }
                </tr>
            </table>
        }
    }

    fn view_network_ip_item_direction(
        &self,
        ctx: &Context<Self>,
        id: &String,
        checked: bool,
    ) -> Html {
        if checked {
            let src_dst_list = Rc::new(vec![
                ViewString::Key("Both (Directions)".to_string()),
                ViewString::Key("SRC".to_string()),
                ViewString::Key("DST".to_string()),
            ]);
            let value_candidates = Rc::new(vec![
                SelectionExtraInfo::Network(EndpointKind::Both),
                SelectionExtraInfo::Network(EndpointKind::Source),
                SelectionExtraInfo::Network(EndpointKind::Destination),
            ]);
            if let Some(selected) = self.direction_items.get(id) {
                html! {
                    <SelectMini::<SelectionExtraInfo, Self>
                        txt={ctx.props().txt.clone()}
                        language={ctx.props().language}
                        parent_message={Message::SetDirectionItem}
                        id={format!("assign-item-direction-{}", id.clone())}
                        list={Rc::clone(&src_dst_list)}
                        candidate_values={Rc::clone(&value_candidates)}
                        selected_value={Rc::clone(selected)}
                        selected_value_cache={selected.try_borrow().ok().and_then(|x| *x)}
                        align_left={false}
                        list_top={28}
                        top_width={Some(70)}
                        list_min_width={Some(70)}
                        kind={SelectMiniKind::DirectionItem}
                    />
                }
            } else {
                html! {}
            }
        } else {
            html! {}
        }
    }

    fn view_input(&self, ctx: &Context<Self>) -> Html {
        let style_pop_input = format!(
            "width: {}px; height: {}px",
            ctx.props().pop_width,
            std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 196,
        );
        let style_pop_input_list = format!("width: {}px; height: {}px", ctx.props().pop_width, {
            if self.input_wrong_msg.is_some() {
                std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 226 - 40
            } else {
                std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 226 - 15
            }
        });
        let style_width_input = format!("width: {}px", ctx.props().pop_width - 86);
        let style_msg = format!("width: {}px; height: {}px;", ctx.props().pop_width, {
            if self.input_wrong_msg.is_some() {
                40
            } else {
                15
            }
        });
        let txt = ctx.props().txt.txt.clone();
        let input_notice = text!(txt, ctx.props().language, "Network/IP Details").to_string();
        let oninput_input = ctx.link().callback(|e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    Message::InputInput(input.value())
                })
        });
        let onclick_add = ctx.link().callback(|_| Message::ClickAddInput);
        let onkeyup = ctx.link().batch_callback(move |e: KeyboardEvent| {
            (e.key() == "Enter").then_some(Message::ClickAddInput)
        });

        html! {
            <div class="complex-select-pop-input" style={style_pop_input}>
                <div class="complex-select-pop-input-input">
                    <div class="complex-select-pop-input-input-text">
                        <input type="text" class="complex-select-pop-input"
                            placeholder={input_notice}
                            style={style_width_input}
                            oninput={oninput_input}
                            onkeyup={onkeyup}
                            value={self.input_text.clone()}
                        />
                    </div>
                    <div class="complex-select-pop-input-input-plus" onclick={onclick_add}>
                    </div>
                </div>
                <div class="complex-select-pop-input-input-message" style={style_msg}>
                {
                    if let Some(msg) = self.input_wrong_msg.as_ref() {
                        html! { text!(txt, ctx.props().language, msg) }
                    } else {
                        html! {}
                    }
                }
                </div>
                <div class="complex-select-pop-input-list" style={style_pop_input_list}>
                    { Self::view_input_list(ctx) }
                </div>
            </div>
        }
    }

    fn view_input_list(ctx: &Context<Self>) -> Html {
        if let Ok(custom) = ctx.props().selected.custom.try_borrow_mut() {
            let mut keys = custom.keys().collect::<Vec<&String>>();
            keys.sort_unstable();
            if keys.is_empty() {
                html! {}
            } else {
                html! {
                    <table>
                    {
                        for keys.iter().map(|&key| {
                            if let Some(value) = custom.get(key) {
                                html! {
                                    match ctx.props().kind {
                                        SelectComplexKind::NetworkIp => {
                                            let style_ip = format!("float: left; width: {}px;", ctx.props().pop_width - 150);
                                            let src_dst_list = Rc::new(vec![
                                                ViewString::Key("Both (Directions)".to_string()),
                                                ViewString::Key("SRC".to_string()),
                                                ViewString::Key("DST".to_string()),
                                            ]);
                                            let value_candidates = Rc::new(vec![
                                                SelectionExtraInfo::Network(EndpointKind::Both),
                                                SelectionExtraInfo::Network(EndpointKind::Source),
                                                SelectionExtraInfo::Network(EndpointKind::Destination),
                                            ]);
                                            let onclick_del = |key: String| ctx.link().callback(move |_| Message::DeleteInputItem(key.clone()));

                                            html! {
                                                <>
                                                    <tr>
                                                        <td class="complex-select-pop-input-list-networks">
                                                            <div style={style_ip} class="complex-select-pop-input-list-text">
                                                                { key }
                                                            </div>
                                                            <div class="complex-select-pop-input-list-delete" onclick={onclick_del(key.clone())}>
                                                            </div>
                                                        </td>
                                                        <td class="complex-select-pop-input-list-direction">
                                                            <SelectMini::<SelectionExtraInfo, Self>
                                                                txt={ctx.props().txt.clone()}
                                                                language={ctx.props().language}
                                                                parent_message={Message::Render}
                                                                id={format!("assign-input-direction-{}", key.clone())}
                                                                list={src_dst_list}
                                                                candidate_values={value_candidates}
                                                                default_value={Some(SelectionExtraInfo::Network(EndpointKind::Both))}
                                                                selected_value={value.clone()}
                                                                selected_value_cache={value.try_borrow().ok().and_then(|x| *x)}
                                                                align_left={false}
                                                                list_top={28}
                                                                top_width={Some(70)}
                                                                list_min_width={Some(70)}
                                                                kind={SelectMiniKind::DirectionItem}
                                                                top_bg_color={"#F6F6F6".to_string()}
                                                            />
                                                        </td>
                                                    </tr>
                                                    <tr>
                                                        <td colspan="2" style="height: 8px;">
                                                        </td>
                                                    </tr>
                                                </>
                                            }
                                        }
                                        SelectComplexKind::Basic => html! {}
                                    }
                                }
                            } else {
                                html! {}
                            }
                        })
                    }
                    </table>
                }
            }
        } else {
            html! {}
        }
    }
}
