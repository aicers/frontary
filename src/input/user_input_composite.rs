use std::cell::RefCell;
use std::rc::Rc;

use json_gettext::get_text;
use num_bigint::BigUint;
use yew::{Component, Context, Html, classes, html};

use super::{
    CheckStatus, CheckboxChildrenConfig, ChildrenPosition, Essential as InputEssential,
    InputConfig, InputItem, cal_index,
    component::{Message, Model},
    user_input::view_asterisk,
};
use crate::{Checkbox, Radio, ViewString, text};

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub(super) fn view_radio(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        options: &[ViewString],
        children_group: &[Option<Vec<Rc<InputConfig>>>],
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        depth: u32,
    ) -> Html {
        let list = Rc::new(options.to_vec());
        let candidates = Rc::new(
            list.iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );
        let txt = ctx.props().txt.txt.clone();
        let my_index = cal_index(base_index, layer_index);

        if let Some(buffer_option) = self.radio_buffer.get(&(my_index)) {
            let checked_index = buffer_option.try_borrow().ok().and_then(|buffer_option| {
                options
                    .iter()
                    .position(|x| x.to_string() == buffer_option.as_str())
            });

            let checked_children_group_data = input_data.try_borrow().ok().and_then(|input_data| {
                if let (Some(checked_index), InputItem::Radio(data)) = (checked_index, &*input_data)
                {
                    data.children_group().get(checked_index).cloned()
                } else {
                    None
                }
            });

            let (children, children_data) =
                if let (Some(checked_index), Some(checked_children_group_data)) =
                    (checked_index, checked_children_group_data)
                {
                    (
                        children_group.get(checked_index).and_then(Clone::clone),
                        checked_children_group_data,
                    )
                } else {
                    (None, Vec::new())
                };

            let (class_child, class_line) = ("input-checkbox-child", "input-checkbox-link-line");

            html! {
                <div class="input-radio-outer">
                    <div class="input-radio">
                        if cfg!(feature = "debug") {
                            { format!("({}:{}={})", base_index.map_or_else(String::new, ToString::to_string), layer_index, my_index) }
                        }
                        <div class="input-radio-title">
                            { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                        </div>
                        <div class="input-radio-radio">
                            <Radio::<Self>
                                txt={ctx.props().txt.clone()}
                                language={ctx.props().language}
                                parent_message={Some(Message::InputRadio(my_index.clone(), input_data.clone()))}
                                list={Rc::clone(&list)}
                                candidate_values={Rc::clone(&candidates)}
                                selected_value={Rc::clone(buffer_option)}
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
                    {
                        if let (Some(children), Some(checked_index)) = (children, checked_index) {
                            if children_data.is_empty() {
                                html! {}
                            } else {
                                html! {
                                    <div class="input-checkbox-children">
                                    {
                                        for children.iter().enumerate().map(|(sub_index, child)|
                                            if let Some(child_data) = children_data.get(sub_index) {
                                                self.view_child(ctx, child, child_data, &cal_index(Some(&my_index), checked_index), sub_index, depth, class_child, class_line)
                                            } else {
                                                html! {}
                                            }
                                        )
                                    }
                                    </div>
                                }
                            }
                        } else {
                            html! {}
                        }
                    }
                    <div class="input-radio-message">
                        { self.view_required_msg(ctx, &my_index) }
                    </div>
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
        language: bool,
        always: Option<CheckStatus>,
        children_config: Option<&CheckboxChildrenConfig>,
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
        both_border: Option<bool>,
        depth: u32,
    ) -> Html {
        let my_index = cal_index(base_index, layer_index);
        let my_index_clone = my_index.clone();
        let txt = ctx.props().txt.txt.clone();
        let input_data_msg = input_data.clone();
        let onclick = ctx.link().callback(move |_| {
            Message::ClickCheckbox(my_index_clone.clone(), input_data_msg.clone())
        });
        let checked = if let Ok(data) = input_data.try_borrow() {
            if let InputItem::Checkbox(data) = (*data).clone() {
                Some(data.status())
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
        let (class_align, class_me, class_child) = children_config.map_or(
            (
                "input-checkbox-children-nextline",
                "input-checkbox-me-nextline",
                "input-checkbox-child",
            ),
            |cc| match cc.position {
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

        let title = if language {
            get_text!(txt, ctx.props().language.tag(), ess.title())
                .map_or(ess.title.clone(), |text| text.to_string())
        } else {
            ess.title.clone()
        };

        if let Some(checked) = checked {
            html! {
                <div class={class}>
                    <div class={class_align}>
                        if cfg!(feature = "debug") {
                            { format!("({}:{}={})", base_index.map_or_else(String::new, ToString::to_string), layer_index, my_index.clone()) }
                        }
                        {
                            if always == Some(CheckStatus::Checked) || always == Some(CheckStatus::Unchecked) {
                                html! {
                                    <div class={classes!("input-checkbox-me", class_me)}>
                                        <Checkbox
                                            status={checked}
                                            {always}
                                        />
                                        <div class="input-checkbox-me-title">
                                            { title }{ view_asterisk(ess.required) }
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class={classes!("input-checkbox-me", class_me)}>
                                        <div class="input-checkbox-me-checkbox" onclick={onclick}>
                                            <Checkbox
                                                status={checked}
                                            />
                                        </div>
                                        <div class="input-checkbox-me-title">
                                            { title }{ view_asterisk(ess.required) }
                                        </div>
                                    </div>
                                }
                            }
                        }
                        <div class="input-checkbox-children">
                        {
                            if checked == CheckStatus::Unchecked {
                                html! {}
                            } else if let (Some(children_config), Ok(input_data)) = (children_config, input_data.try_borrow()) {
                                html! {
                                    for children_config.children.iter().enumerate().map(|(sub_index, child)| {
                                        let child_data = if let InputItem::Checkbox(data) = input_data.clone() {
                                            data.children().get(sub_index).cloned()
                                        } else {
                                            None
                                        };
                                        let class_line = if children_config.position == ChildrenPosition::Right {
                                            if sub_index == 0 {
                                                "input-checkbox-link-line-right"
                                            } else {
                                                "input-checkbox-link-line"
                                            }
                                        } else {
                                            "input-checkbox-link-line"
                                        };
                                        if let Some(child_data) = child_data {
                                            self.view_child(ctx, child, &child_data, &my_index, sub_index, depth, class_child, class_line)
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

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines)]
    pub(super) fn view_child(
        &self,
        ctx: &Context<Self>,
        child: &Rc<InputConfig>,
        child_data: &Rc<RefCell<InputItem>>,
        base_index: &BigUint,
        layer_index: usize,
        depth: u32,
        class_child: &'static str,
        class_line: &'static str,
    ) -> Html {
        match &**child {
            InputConfig::Text(config) => {
                html! {
                    <div class={class_child}>
                        // TODO: issue #111
                        <div class={class_line}>
                        </div>
                        { self.view_text(ctx, &config.ess, config.length, config.width, child_data, Some(base_index), layer_index, false, false, config.immutable, config.validation_rule.as_ref()) }
                    </div>
                }
            }
            InputConfig::DomainName(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_domain_name(ctx, &config.ess, config.width, child_data, Some(base_index), layer_index, false) }
                    </div>
                }
            }
            InputConfig::HostNetworkGroup(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_host_network_group(ctx, &config.ess, config.kind, config.num, config.width, child_data, Some(base_index), layer_index) }
                    </div>
                }
            }
            InputConfig::SelectSingle(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_select_searchable(ctx, false, &config.ess, config.width, &config.options, child_data, Some(base_index), layer_index, depth, false) }
                    </div>
                }
            }
            InputConfig::SelectMultiple(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_select_nic_or(ctx, config.options.as_ref(), config.nic_index, &config.ess, child_data, Some(base_index), layer_index, depth) }
                    </div>
                }
            }
            InputConfig::Unsigned32(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_unsigned_32(ctx, &config.ess, config.min, config.max, config.width, child_data, Some(base_index), layer_index, false, false) }
                    </div>
                }
            }
            InputConfig::Unsigned8(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_unsigned_8(ctx, &config.ess, config.min, config.max, config.width, child_data, Some(base_index), layer_index, false, false) }
                    </div>
                }
            }
            InputConfig::Float64(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_float_64(ctx, &config.ess, config.step, config.width, child_data, Some(base_index), layer_index, false, false) }
                    </div>
                }
            }
            InputConfig::Percentage(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_percentage(ctx, &config.ess, config.min, config.max, config.num_decimals, config.width, child_data, Some(base_index), layer_index, false) }
                    </div>
                }
            }
            InputConfig::Group(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_group(ctx, &config.ess, config.all_in_one_row, &config.widths, &config.items, child_data, Some(base_index), layer_index) }
                    </div>
                }
            }
            InputConfig::Checkbox(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_checkbox(ctx, &config.ess, config.language, config.always, config.children.as_ref(), child_data, Some(base_index), layer_index, None, depth + 1) }
                    </div>
                }
            }
            InputConfig::Radio(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_radio(ctx, &config.ess, &config.options, &config.children_group, child_data, Some(base_index), layer_index, depth + 1) }
                    </div>
                }
            }
            InputConfig::Password(_)
            | InputConfig::Tag(_)
            | InputConfig::VecSelect(_)
            | InputConfig::Nic(_)
            | InputConfig::File(_)
            | InputConfig::Comparison(_) => {
                panic!(
                    "Checkbox does not support Password, Tag, Nic, File, VecSelect, and Comparison for children."
                )
            }
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
        items_conf: &[Rc<InputConfig>],
        input_data: &Rc<RefCell<InputItem>>,
        base_index: Option<&BigUint>,
        layer_index: usize,
    ) -> Html {
        let this_index = cal_index(base_index, layer_index); // == my_index
        let this_index_clone = this_index.clone();
        let input_data_clone = input_data.clone();
        let input_data_clone_ref = &(input_data.clone());
        let items_conf_clone = items_conf.to_vec();
        let items_conf_clone_ref = &(items_conf_clone.clone());

        let Ok(input_data) = input_data.try_borrow() else {
            return html! {};
        };
        let InputItem::Group(input_data) = &*input_data else {
            return html! {};
        };
        let txt = ctx.props().txt.txt.clone();
        let onclick_add = ctx.link().callback(move |_| {
            Message::InputGroupAdd(
                this_index_clone.clone(),
                input_data_clone.clone(),
                items_conf_clone.clone(),
            )
        });
        let display_titles =
            !(items_conf.len() == 1 && items_conf.first().is_some_and(|x| x.title().is_empty()));
        let required = ess.required;
        let add_message = if cfg!(feature = "pumpkin") {
            "Add another condition"
        } else {
            "Add"
        };
        let input_add_class = if cfg!(feature = "pumpkin") {
            "input-group-add-start"
        } else {
            "input-group-add"
        };
        let input_group = if cfg!(feature = "pumpkin") {
            "input-group-one-col"
        } else {
            "input-group"
        };
        let input_contents_item_title = if display_titles {
            "input-contents-item-title"
        } else {
            "input-contents-item-title-no-margin"
        };
        let asterisk = if cfg!(feature = "pumpkin") {
            html! {}
        } else {
            view_asterisk(ess.required)
        };
        let style_for = |index: usize| {
            widths
                .get(index)
                .and_then(|w| w.map(|v| format!("width: {v}px;")))
                .unwrap_or_default()
        };

        html! {
            <div class="input-item">
                <div class={input_contents_item_title}>
                    { text!(txt, ctx.props().language, ess.title()) }
                    { asterisk }
                </div>
                if !cfg!(feature="pumpkin") {
                    <div class="input-text-message">
                        { self.view_required_msg(ctx, &this_index.clone()) }
                    </div>
                }
                <div class={input_group}>
                    <div>
                        <table class="input-group">
                            if cfg!(feature = "debug") {
                                { format!("({}:{}={})", base_index.map_or_else(String::new, ToString::to_string), layer_index, this_index.to_string()) }
                            }
                            if !cfg!(feature = "pumpkin") {
                                if display_titles {
                                    <tr>
                                        <th class={classes!("input-group-heading" ,"input-group-empty-header")}>
                                        </th>
                                        {
                                            for items_conf.iter().enumerate().map(|(col_index, each)| {
                                                let style = style_for(col_index);
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
                                }
                            }
                            {
                                for input_data.iter().enumerate().map(move |(row_index, row)| {
                                    let input_data_callback = input_data_clone_ref.clone();
                                    let items_conf_callback = items_conf_clone_ref.clone();
                                    let this_index_clone = this_index.clone();
                                    let onclick_delete = ctx.link().callback(move |_| {
                                        Message::InputGroupDelete(
                                            this_index_clone.clone(),
                                            row_index,
                                            input_data_callback.clone(),
                                            items_conf_callback.clone(),
                                            required,
                                        )
                                    });
                                    let row_rep_index = cal_index(Some(&this_index), row_index);
                                    let row_has_error = row.iter().enumerate().any(|(i, _)| {
                                        self.required_msg.contains(&cal_index(Some(&row_rep_index), i))
                                    });
                                    let line_class = match (row_index == 0, row_has_error) {
                                        (true, true) => "group-list-link-line-top first-row long",
                                        (true, false) => "group-list-link-line-top first-row",
                                        (false, true) => "group-list-link-line-top long",
                                        (false, false) => "group-list-link-line-top",
                                    };
                                    let delete_cell_class = if row_index == 0 {
                                        classes!("input-trash-can-delete", "first-row")
                                    } else {
                                        classes!("input-trash-can-delete")
                                    };

                                    if one_row {
                                        html! {
                                            <tr>
                                                {
                                                    if cfg!(feature = "pumpkin") {
                                                        html! {
                                                            <td class="group-list-link-cell">
                                                                <div class={line_class}></div>
                                                            </td>
                                                        }
                                                    } else {
                                                        html! {
                                                            <div class="group-list-link-line-top"></div>
                                                        }
                                                    }
                                                }
                                                {
                                                    for row.iter().zip(items_conf.iter()).enumerate().map(|(col_index, (each_item, each_conf))| {
                                                        let txt = Rc::clone(&ctx.props().txt.txt);
                                                        let should_render_title = cfg!(feature = "pumpkin") && row_index == 0;
                                                        let style = style_for(col_index);
                                                        html! {
                                                            <td class="input-group" {style}>
                                                                <div class="input-group-item-outer">
                                                                {
                                                                    if should_render_title {
                                                                        html! {
                                                                            <div class="input-group-heading">
                                                                                { text!(txt, ctx.props().language, each_conf.title()) }
                                                                                { view_asterisk(each_conf.required()) }
                                                                            </div>
                                                                        }
                                                                    } else {
                                                                        html! {}
                                                                    }
                                                                }
                                                                {
                                                                    match &**each_conf {
                                                                        InputConfig::Text(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_text(ctx, &ess, config.length, config.width, each_item,
                                                                                Some(&row_rep_index), col_index, false, true, config.immutable, config.validation_rule.as_ref())
                                                                        }
                                                                        InputConfig::HostNetworkGroup(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_host_network_group(ctx, &config.ess, config.kind, config.num, config.width, each_item,
                                                                                Some(&row_rep_index), col_index)
                                                                        }
                                                                        InputConfig::SelectSingle(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_select_searchable(ctx, false, &ess, config.width, &config.options, each_item,
                                                                                Some(&row_rep_index), col_index, 1, true)
                                                                        }
                                                                        InputConfig::SelectMultiple(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_select_nic_or(ctx, config.options.as_ref(), config.nic_index, &ess, each_item,
                                                                                Some(&row_rep_index), col_index, 1)
                                                                            }
                                                                        InputConfig::Unsigned32(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_unsigned_32(ctx, &ess, config.min, config.max, config.width, each_item,
                                                                                Some(&row_rep_index), col_index, false, true)
                                                                        }
                                                                        InputConfig::Float64(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_float_64(ctx, &ess, config.step, config.width, each_item,
                                                                                Some(&row_rep_index), col_index, false, true)
                                                                        }
                                                                        InputConfig::Percentage(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_percentage(ctx, &ess, config.min, config.max, config.num_decimals, config.width, each_item,
                                                                                Some(&row_rep_index), col_index, false)
                                                                        }
                                                                        InputConfig::Comparison(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_comparison(ctx, &ess, each_item, Some(&row_rep_index), col_index, true)
                                                                        }
                                                                        InputConfig::VecSelect(config) => {
                                                                            self.view_vec_select(ctx, &config.ess, &config.items_ess_list, config.last, config.full_width, &config.widths, &config.max_widths,
                                                                            &config.max_heights, &config.map_list, each_item, Some(&row_rep_index), col_index, true)
                                                                        }
                                                                        _ => {
                                                                            panic!("Input Group does not support some items such as Password, Tag, Nic, File, Group, Checkbox, and Radio.")
                                                                        }
                                                                    }
                                                                }
                                                                </div>
                                                            </td>
                                                        }
                                                    })
                                                }
                                                {
                                                    if cfg!(feature = "pumpkin") {
                                                        html! {
                                                            <td class={delete_cell_class}>
                                                                <div class="input-trash-can-delete-outer">
                                                                    <div class="input-trash-can-delete" onclick={onclick_delete}>
                                                                    </div>
                                                                </div>
                                                            </td>
                                                        }
                                                    }
                                                    else {
                                                        html!{
                                                            <td class="input-group-delete">
                                                                <div class="input-nic-delete-outer">
                                                                    <div class="input-nic-delete" onclick={onclick_delete}>
                                                                    </div>
                                                                </div>
                                                            </td>
                                                        }
                                                    }
                                                }
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
                    {
                        if cfg!(feature = "pumpkin") {
                            html!{}
                        }
                        else {
                            html!{
                                <div class={input_add_class}>
                                    <div class="input-add-item" onclick={onclick_add.clone()}>
                                        { text!(txt, ctx.props().language, add_message) }
                                    </div>
                                </div>
                            }
                        }
                    }
                </div>
                {
                    if cfg!(feature = "pumpkin") {
                        html!{
                            <div class={input_add_class}>
                                <td class="group-list-link-line-bottom"></td>
                                <div class="input-add-item" onclick={onclick_add}>
                                    <img src="/frontary/pumpkin/addition-symbol.svg" />
                                    { text!(txt, ctx.props().language, add_message) }
                                </div>
                            </div>
                        }
                    }
                    else {
                        html!{}
                    }
                }
            </div>
        }
    }
}
