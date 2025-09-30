use std::rc::Rc;

use htmlescape::decode_html;
use json_gettext::get_text;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::{Context, Html, events::InputEvent, html};

use super::{MIN_POP_HEIGHT, Message, Model};
use crate::{
    CheckStatus, Checkbox, EndpointKind, NBSP, NetworkItem, SelectComplexKind, SelectMini,
    SelectMiniKind, SelectionExtraInfo, Theme, ViewString, select::complex::ItemKind, text,
    window_inner_height,
};

impl Model {
    #[allow(clippy::too_many_lines)]
    pub(super) fn view_pop(&self, ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let (predefined_selected_opt, custom_selected_count) = ctx.props().selected.len();
        let predefined_selected_count = predefined_selected_opt.unwrap_or_default();
        let postfix = if cfg!(feature = "pumpkin") {
            if predefined_selected_count == 0 && custom_selected_count == 0 {
                "unchecked"
            } else {
                "checked"
            }
        } else {
            match self.check_status(ctx, false) {
                CheckStatus::Checked => "checked",
                _ => "unchecked",
            }
        };
        let theme = ctx.props().theme;
        let ext = if cfg!(feature = "pumpkin") {
            "svg"
        } else {
            "png"
        };
        let radio_opener = Theme::path(&theme, &format!("radio-opener-{postfix}.{ext}"));
        let style_all = format!("background-image: url('{radio_opener}');");
        let style = format!("width: {}px;", ctx.props().pop_width);
        let style_pop = if cfg!(feature = "pumpkin") {
            format!("width: {}px;", ctx.props().pop_width)
        } else {
            format!(
                "width: {}px; height: {}px;",
                ctx.props().pop_width,
                std::cmp::max(MIN_POP_HEIGHT, window_inner_height())
            )
        };
        let style_head_title = format!("width: {}px;", ctx.props().pop_width - 34);
        let theme = ctx.props().theme;
        let get_expand_collapse_style = |visible: bool| {
            let filename = if visible {
                "collapse-contents"
            } else {
                "expand-contents"
            };
            let ext = if cfg!(feature = "pumpkin") {
                "svg"
            } else {
                "png"
            };
            let url = Theme::path(&theme, &format!("{filename}.{ext}"));
            format!("background-image: url('{url}');")
        };
        let style_list = get_expand_collapse_style(self.view_list);
        let style_input = get_expand_collapse_style(self.view_input);
        let class_input_head = if cfg!(feature = "pumpkin") || !self.view_list {
            "complex-select-pop-input-head"
        } else {
            "complex-select-pop-input-head-bottom"
        };

        let onclick_input = ctx.link().callback(|_| Message::ToggleInput);
        let onclick_list = ctx.link().callback(|_| Message::ToggleList);
        let onclick_all = ctx.link().callback(|_| Message::ClickAll);
        let onclick_close = ctx.link().callback(|_| Message::Close);

        html! {
            <div id={ctx.props().id.clone()} class="complex-select-pop" style={style_pop}>
                <div class="complex-select-pop-head">
                    {
                        if cfg!(feature = "pumpkin") {
                            html! {
                                <>
                                    <div class="complex-select-pop-head-text">
                                        { text!(txt, ctx.props().language, &ctx.props().title) }
                                        { decode_html(NBSP).expect("safely-selected character") }
                                    </div>
                                    <div class="complex-select-pop-head-close" onclick={onclick_close}>
                                        <div class="complex-select-pop-head-close-icon"></div>
                                    </div>
                                </>
                            }
                        }
                        else {
                            html! {
                                <table>
                                    <tr style={style.clone()}>
                                        <td class="complex-select-pop-head-text" style={style_head_title}>
                                            { text!(txt, ctx.props().language, &ctx.props().title) }
                                            { decode_html(NBSP).expect("safely-selected character") }
                                            <span class="complex-select-pop-head-color">
                                                { "(" } { Self::selected_len(ctx) } { ")" }
                                            </span>
                                        </td>
                                        <td class="complex-select-pop-head-close" onclick={onclick_close}>
                                            <div class="complex-select-pop-head-close-icon"></div>
                                        </td>
                                    </tr>
                                </table>
                            }
                        }
                    }
                </div>
                {
                    if cfg!(feature = "pumpkin") {
                        let (style_width_input, style_msg, input_notice, oninput_input, onclick_add,
                                onkeyup) = self.input_props(ctx);
                        let input_group_min_height = match self.input_wrong_msg {
                            None => 116,
                            Some("The input already exists.") => 124,
                            Some(_) => 144,
                        };
                        let list_group_style = match (self.view_list, self.view_input) {
                            (true, _) => "flex: 1 1 0;",
                            (false, _) => "flex: 0 0 auto;",
                        };
                        let input_group_flex = match (self.view_list, self.view_input) {
                            (true, false) => "flex: 0 0 auto;",
                            _ => "flex: 1 1 0;",
                        };
                        let input_group_style = format!(
                            "{input_group_flex} min-height: {input_group_min_height}px;"
                        );
                        let list_toggle_key = if self.view_list { "Hide" } else { "Show" };
                        let input_toggle_key = if self.view_input { "Hide" } else { "Show" };
                        html! {
                                <div class="complex-select-pop-container">
                                <div class="complex-select-pop-list-group" style={list_group_style}>
                                    <div class="complex-select-pop-list-text">
                                        <div class="complex-select-pop-list-head" onclick={onclick_list}>
                                            <div class="complex-select-pop-head-1st">
                                                { text!(txt, ctx.props().language, "Saved Network/IPs") }
                                                { format!(" ({})", predefined_selected_count) }
                                            </div>
                                            <div class="complex-select-pop-head-2nd">
                                                <div class="complex-select-pop-head-2nd-text">
                                                    { text!(txt, ctx.props().language, list_toggle_key) }
                                                </div>
                                                <div class="complex-select-pop-head-2nd-icon" style={style_list}></div>
                                            </div>
                                        </div>
                                        <div class="complex-select-pop-subtext">
                                            { text!(txt, ctx.props().language, "Choose from network/IPs previously saved in your environment.") }
                                        </div>
                                    </div>
                                    {
                                        if self.view_list {
                                            self.view_list(ctx)
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>
                                <div class="complex-select-pop-divider"></div>
                                <div class="complex-select-pop-input-group" style={input_group_style}>
                                    <div class="complex-select-pop-input-group-inner">
                                        <div class="complex-select-pop-input-text">
                                            <div class={class_input_head} onclick={onclick_input}>
                                                <div class="complex-select-pop-head-1st">
                                                    { text!(txt, ctx.props().language, "Custom Network/IPs") }
                                                    { format!(" ({})", custom_selected_count) }
                                                </div>
                                                <div class="complex-select-pop-head-2nd">
                                                    <div class="complex-select-pop-head-2nd-text">
                                                        { text!(txt, ctx.props().language, input_toggle_key) }
                                                    </div>
                                                    <div class="complex-select-pop-head-2nd-icon" style={style_input}>
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="complex-select-pop-subtext">
                                                { text!(txt, ctx.props().language, "Specify a custom network/IP.") }
                                            </div>
                                        </div>
                                        <div class="complex-select-pop-input-input-group">
                                            <div class="complex-select-pop-input-input">
                                                <div class="complex-select-pop-input-input-text">
                                                    <input type="text" class="complex-select-pop-input"
                                                        placeholder={input_notice}
                                                        style={format!(
                                                            "{} {}",
                                                            style_width_input,
                                                            if self.input_wrong_msg.is_some() {
                                                                "border-radius: 8px; border: 2px solid var(--Red-60);"
                                                            } else {
                                                                ""
                                                            }
                                                        )}
                                                        oninput={oninput_input}
                                                        onkeyup={onkeyup}
                                                        value={self.input_text.clone()}
                                                    />
                                                </div>
                                                <div class="complex-select-pop-input-input-plus" onclick={onclick_add}></div>
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
                                        </div>
                                    </div>
                                    {
                                        if self.view_input {
                                            html! {
                                                <div class="complex-select-pop-input-container">
                                                    {self.view_input(ctx)}
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>
                            </div>
                        }
                    }
                    else {
                        html!{
                            <>
                                // All for the entire
                                <div class="complex-select-pop-all" style={style.clone()}>
                                    <div class="complex-select-pop-all-text">
                                        { text!(txt, ctx.props().language, "Select All") }
                                    </div>
                                    <div class="complex-select-pop-all-button" style={style_all} onclick={onclick_all}></div>
                                </div>
                                <div class="complex-select-pop-list-head">
                                    <table>
                                        <tr onclick={onclick_list}>
                                            <td class="complex-select-pop-head-1st">
                                                { text!(txt, ctx.props().language, "Choose ones (in the list)") }
                                            </td>
                                            <td class="complex-select-pop-head-2nd" style={style_list}></td>
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
                                            <td class="complex-select-pop-head-2nd" style={style_input}></td>
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
                            </>
                        }
                    }
                }
            </div>
        }
    }

    fn view_list(&self, ctx: &Context<Self>) -> Html {
        let (
            style,
            style_width_search,
            style_pop_list,
            style_pop_list_list,
            style_pop_list_list_items,
        ) = if cfg!(feature = "pumpkin") {
            (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            )
        } else {
            (
                format!("width: {}px;", ctx.props().pop_width),
                format!("width: {}px", ctx.props().pop_width - 48),
                format!(
                    "width: {}px; height: {}px",
                    ctx.props().pop_width,
                    std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 202
                ),
                format!(
                    "width: {}px; height: {}px",
                    ctx.props().pop_width,
                    std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 244
                ),
                format!(
                    "width: {}px; height: {}px",
                    ctx.props().pop_width,
                    std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 286
                ),
            )
        };
        let txt = ctx.props().txt.txt.clone();

        html! {
            <div class="complex-select-pop-list" style={style_pop_list}>
                {
                    if let Ok(list) = ctx.props().list.try_borrow() {
                        if cfg!(feature = "pumpkin") && list.is_empty() {
                            html! {
                                <div class="complex-select-pop-list-empty">
                                    { text!(txt, ctx.props().language, "No registered networks.") }
                                </div>
                            }
                        } else {
                            self.view_registered_list(
                                ctx,
                                style,
                                style_width_search,
                                style_pop_list_list,
                                style_pop_list_list_items,
                            )
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }

    #[allow(clippy::too_many_lines)]
    fn view_registered_list(
        &self,
        ctx: &Context<Self>,
        style: String,
        style_width_search: String,
        style_pop_list_list: String,
        style_pop_list_list_items: String,
    ) -> Html {
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
        let theme = ctx.props().theme;

        html! {
            <>
                <div class="complex-select-pop-list-search" style={style}>
                    <input type="text" class="complex-select-search"
                        placeholder={search_notice}
                        style={style_width_search}
                        oninput={oninput_search}
                        value={self.search_text.clone()}
                    />
                </div>
                {
                    if cfg!(feature = "pumpkin") {
                        html! {
                            {
                                match ctx.props().kind {
                                    SelectComplexKind::NetworkIp => {
                                        let onclick_all = ctx
                                            .link()
                                            .callback(|_| Message::ClickAllBelow(ItemKind::Registered));
                                        html! {
                                            <>
                                                <div class="complex-select-pop-list-select-all-group">
                                                    <div class="complex-select-pop-list-select-all-checkbox">
                                                        <div onclick={onclick_all}>
                                                            <Checkbox status={check_status} {theme} />
                                                        </div>
                                                        <div class="complex-select-pop-list-select-all-text">
                                                            { text!(txt, ctx.props().language, "Select below") }
                                                        </div>
                                                    </div>
                                                    { self.view_direction(ctx, ItemKind::Registered) }
                                                </div>
                                                <div class="complex-select-pop-list-divider">
                                                </div>
                                                <div class="complex-select-pop-list-container">
                                                    { self.view_registered_list_items(ctx) }
                                                </div>
                                            </>
                                        }
                                    },
                                    SelectComplexKind::Basic => html! {},
                                }
                            }
                        }
                    }
                    else {
                        html! {
                            <div class="complex-select-pop-list-list" style={style_pop_list_list}>
                                {
                                    match ctx.props().kind {
                                        SelectComplexKind::NetworkIp => {
                                            let onclick_all = ctx
                                                .link()
                                                .callback(|_| Message::ClickAllBelow(ItemKind::Registered));
                                            html! {
                                                <div class="complex-select-pop-list-list-all">
                                                    <table class="complex-select-pop-list-list-all">
                                                        <tr style="position: relative;">
                                                            <td class="complex-select-pop-list-list-all-checkbox">
                                                                // All for the below list
                                                                <div onclick={onclick_all}>
                                                                    <Checkbox status={check_status} {theme} />
                                                                </div>
                                                            </td>
                                                            <td class="complex-select-pop-list-list-all-item">
                                                                { text!(txt, ctx.props().language, "Select All") }
                                                            </td>
                                                            <td>
                                                                { self.view_direction(ctx, ItemKind::Registered) }
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
                                        if cfg!(feature = "pumpkin") {
                                            html! {
                                                <div class="complex-select-pop-list-list-items-inner">
                                                    { self.view_registered_list_items(ctx) }
                                                </div>
                                            }
                                        } else {
                                            self.view_registered_list_items(ctx)
                                        }
                                    }
                                </div>
                            </div>
                        }
                    }
                }
            </>
        }
    }

    fn view_registered_list_items(&self, ctx: &Context<Self>) -> Html {
        if let Ok(list) = ctx.props().list.try_borrow() {
            if let Some(search) = self.search_result.as_ref() {
                html! {
                    for search.iter().filter_map(|&index| list.get(index)).map(|item| self.view_list_item(ctx, item))
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

    fn view_direction(&self, ctx: &Context<Self>, origin: ItemKind) -> Html {
        let (parent_message, id, active) = match origin {
            ItemKind::Registered => {
                let check_status = if self.search_result.is_some() {
                    self.check_status(ctx, true)
                } else {
                    self.check_status(ctx, false)
                };
                let active = matches!(
                    check_status,
                    CheckStatus::Checked | CheckStatus::Indeterminate
                );
                (Message::SetDirection, "assign-direction", active)
            }
            ItemKind::Custom => {
                let check_status = Self::check_custom_status(ctx);
                let active = matches!(
                    check_status,
                    CheckStatus::Checked | CheckStatus::Indeterminate
                );
                (
                    Message::SetDirectionItem(ItemKind::Custom),
                    "assign-direction-custom",
                    active,
                )
            }
        };
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
        let theme = ctx.props().theme;
        html! {
            <SelectMini::<EndpointKind, Self>
                txt={ctx.props().txt.clone()}
                language={ctx.props().language}
                parent_message={parent_message}
                active={active}
                deactive_class_suffix={Some("-deactive".to_string())}
                id={id.to_string()}
                list={direction_list}
                candidate_values={value_candidates}
                selected_value={self.direction.clone()}
                selected_value_cache={self.direction.try_borrow().ok().and_then(|x| *x)}
                align_left={false}
                list_top={40}
                kind={SelectMiniKind::DirectionAll}
                {theme}
            />
        }
    }

    #[allow(clippy::too_many_lines)]
    fn view_list_item(&self, ctx: &Context<Self>, item: &NetworkItem) -> Html {
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
                .callback(move |_| Message::ClickItem(key.clone(), ItemKind::Registered))
        };
        let style_item_width = match (ctx.props().kind, cfg!(feature = "pumpkin")) {
            (SelectComplexKind::NetworkIp, true) => "",
            (SelectComplexKind::NetworkIp, false) => "width: 209px;",
            (SelectComplexKind::Basic, _) => "width: 279px;",
        };
        let theme = ctx.props().theme;
        if cfg!(feature = "pumpkin") {
            html! {
                <div class="complex-select-pop-list-item">
                    <div class="complex-select-pop-list-item-checkbox">
                        <div onclick={onclick_item(key)}>
                            <Checkbox status={checked} {theme} />
                        </div>
                        <div class="complex-select-pop-list-item-checkbox-text">
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
                                                            { r.start.clone() } { " - " } { r.end.clone() } <br/>
                                                        </>
                                                    })
                                                }
                                                </div>
                                            </>
                                        }
                                }
                                else {
                                    html!{}
                                }
                            }
                        </div>
                    </div>
                    { self.view_network_ip_item_direction(ctx, item.id(), checked == CheckStatus::Checked || checked == CheckStatus::Indeterminate) }
                </div>
            }
        } else {
            html! {
                <table>
                    <tr>
                        <td class="complex-select-pop-list-list-items-checkbox">
                            <div onclick={onclick_item(key)}>
                                <Checkbox status={checked} />
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
                                                        { r.start.clone() } { " - " } { r.end.clone() } <br/>
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
    }

    fn view_network_ip_item_direction(
        &self,
        ctx: &Context<Self>,
        id: &String,
        checked: bool,
    ) -> Html {
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
        let top_width = if cfg!(feature = "pumpkin") { 94 } else { 70 };
        let theme = ctx.props().theme;
        if let Some(selected) = self.direction_items.get(id) {
            html! {
            <SelectMini::<SelectionExtraInfo, Self>
                txt={ctx.props().txt.clone()}
                language={ctx.props().language}
                parent_message={Message::SetDirectionItem(ItemKind::Registered)}
                active={checked}
                deactive_class_suffix={Some("-deactive".to_string())}
                id={format!("assign-item-direction-{}", id.clone())}
                list={Rc::clone(&src_dst_list)}
                candidate_values={Rc::clone(&value_candidates)}
                selected_value={Rc::clone(selected)}
                selected_value_cache={selected.try_borrow().ok().and_then(|x| *x)}
                align_left={false}
                list_top={28}
                {top_width}
                list_min_width={Some(70)}
                kind={SelectMiniKind::DirectionItem}
                {theme}
            />
            }
        } else {
            html! {}
        }
    }

    #[allow(clippy::too_many_lines)]
    fn view_input(&self, ctx: &Context<Self>) -> Html {
        let style_pop_input = format!(
            "width: {}px; height: {}px",
            ctx.props().pop_width,
            std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 196,
        );
        let style_pop_input_list = format!(
            "width: {}px; height: {}px",
            ctx.props().pop_width,
            if self.input_wrong_msg.is_some() {
                std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 226 - 40
            } else {
                std::cmp::max(MIN_POP_HEIGHT, window_inner_height()) - 226 - 15
            }
        );
        let txt = ctx.props().txt.txt.clone();

        if cfg!(feature = "pumpkin") {
            let custom_keys = ctx
                .props()
                .selected
                .custom
                .try_borrow()
                .map(|custom| custom.keys().cloned().collect::<Vec<String>>())
                .unwrap_or_default();

            let custom_is_empty = custom_keys.is_empty();

            html! {
                <>
                    {
                        if custom_is_empty {
                            html! {}
                        } else {
                            let check_status = Self::check_custom_status(ctx);
                            let onclick_all_custom = ctx
                                .link()
                                .callback(|_| Message::ClickAllBelow(ItemKind::Custom));
                            let theme = ctx.props().theme;
                            html! {
                                <>
                                    <div class="complex-select-pop-list-select-all-group">
                                        <div class="complex-select-pop-list-select-all-checkbox">
                                            <div onclick={onclick_all_custom}>
                                                <Checkbox status={check_status} {theme} />
                                            </div>
                                            <div class="complex-select-pop-list-select-all-text">
                                                { text!(txt, ctx.props().language, "Select below") }
                                            </div>
                                        </div>
                                        { self.view_direction(ctx, ItemKind::Custom) }
                                    </div>
                                    <div class="complex-select-pop-list-divider"></div>
                                </>
                            }
                        }
                    }

                    if custom_is_empty {
                        <div class="complex-select-pop-input-empty">
                            { text!(txt, ctx.props().language, "No custom network/IPs added.") }
                        </div>

                    } else {
                        <div class="complex-select-pop-input-list">
                            { Self::view_input_list(ctx) }
                        </div>
                    }
                </>
            }
        } else {
            let (style_width_input, style_msg, input_notice, oninput_input, onclick_add, onkeyup) =
                self.input_props(ctx);

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
    }

    #[allow(clippy::too_many_lines)]
    fn view_input_list(ctx: &Context<Self>) -> Html {
        if let Ok(custom) = ctx.props().selected.custom.try_borrow_mut() {
            let mut keys = custom.keys().collect::<Vec<&String>>();
            keys.sort_unstable();
            if keys.is_empty() {
                html! {}
            } else {
                html! {
                    <>
                    {
                        for keys.iter().map(|&key| {
                            if let Some(value) = custom.get(key) {
                                html! {
                                    match ctx.props().kind {
                                        SelectComplexKind::NetworkIp => {
                                            let style_ip = if cfg!(feature = "pumpkin") {
                                                String::new()
                                            } else {
                                                format!("float: left; width: {}px;", ctx.props().pop_width - 150)
                                            };
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
                                            let (top_bg_color, top_width, list_top) = if cfg!(feature = "pumpkin") {
                                                ("rgba(97, 105, 116, 0.24)", 94, 42)
                                            } else {
                                                ("#F6F6F6", 70, 28)
                                            };
                                            let theme = ctx.props().theme;
                                            if cfg!(feature = "pumpkin") {
                                                let theme = ctx.props().theme;
                                                let checked = if let Ok(v) = value.try_borrow() {
                                                    match *v {
                                                        Some(SelectionExtraInfo::Network(EndpointKind::Both)) => CheckStatus::Checked,
                                                        Some(
                                                            SelectionExtraInfo::Network(_)
                                                                | SelectionExtraInfo::Basic,
                                                        ) => CheckStatus::Indeterminate,
                                                        None => CheckStatus::Unchecked,
                                                    }
                                                } else { CheckStatus::Unchecked };
                                                let onclick_custom = |k: String| {
                                                    ctx.link().callback(move |_| {
                                                        Message::ClickItem(
                                                            k.clone(),
                                                            ItemKind::Custom,
                                                        )
                                                    })
                                                };
                                                html! {
                                                    <div class="complex-select-pop-input-list-items">
                                                        <div class="complex-select-pop-input-list-component">
                                                            <div onclick={onclick_custom(key.clone())}>
                                                                <Checkbox status={checked} {theme} />
                                                            </div>
                                                            <div class="complex-select-pop-input-list-text">
                                                                { key }
                                                            </div>
                                                            <div class="complex-select-pop-input-list-delete" onclick={onclick_del(key.clone())}>
                                                            </div>
                                                        </div>
                                                        <div class="complex-select-pop-input-list-direction">
                                                            {
                                                                if let Ok(v) = value.try_borrow()
                                                                    && v.is_some()
                                                                {
                                                                    html! {
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
                                                                        {list_top}
                                                                        top_width={Some(top_width)}
                                                                        list_min_width={Some(70)}
                                                                        kind={SelectMiniKind::DirectionItem}
                                                                        {top_bg_color}
                                                                    />
                                                                }
                                                            } else {
                                                                html! {}
                                                            }
                                                        }
                                                        </div>
                                                    </div>
                                                }
                                            }
                                            else {
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
                                                                    {list_top}
                                                                    top_width={Some(top_width)}
                                                                    list_min_width={Some(70)}
                                                                    kind={SelectMiniKind::DirectionItem}
                                                                    {top_bg_color}
                                                                    {theme}
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
                                        }
                                        SelectComplexKind::Basic => html! {}
                                    }
                                }
                            } else {
                                html! {}
                            }
                        })
                    }
                    </>
                }
            }
        } else {
            html! {}
        }
    }

    fn input_props(
        &self,
        ctx: &Context<Self>,
    ) -> (
        String,
        String,
        String,
        yew::Callback<InputEvent>,
        yew::Callback<yew::MouseEvent>,
        yew::Callback<KeyboardEvent>,
    ) {
        let txt = ctx.props().txt.txt.clone();
        let style_width_input = if cfg!(feature = "pumpkin") {
            format!("width: {}px;", ctx.props().pop_width - 96)
        } else {
            format!("width: {}px", ctx.props().pop_width - 86)
        };
        let style_msg = if cfg!(feature = "pumpkin") {
            format!(
                "width: {}px; height: {}px;",
                354,
                if let Some(msg) = &self.input_wrong_msg {
                    if *msg == "The input already exists." {
                        24
                    } else {
                        44
                    }
                } else {
                    16
                }
            )
        } else {
            format!(
                "width: {}px; height: {}px;",
                ctx.props().pop_width,
                if self.input_wrong_msg.is_some() {
                    40
                } else {
                    15
                }
            )
        };
        let input_notice = if cfg!(feature = "pumpkin") {
            text!(
                txt,
                ctx.props().language,
                "Enter an IP (e.g., 192.168.0.1/24)"
            )
        } else {
            text!(txt, ctx.props().language, "Network/IP Details")
        }
        .to_string();
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
        (
            style_width_input,
            style_msg,
            input_notice,
            oninput_input,
            onclick_add,
            onkeyup,
        )
    }
}
