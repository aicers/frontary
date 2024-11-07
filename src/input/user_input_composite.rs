use std::cell::RefCell;
use std::rc::Rc;

use json_gettext::get_text;
use yew::{classes, html, Component, Context, Html};

use super::{
    component::{Message, Model},
    user_input::{view_asterisk, MAX_PER_LAYER},
    InputItem,
};
use crate::{
    text, Checkbox, CheckStatus, ChildrenPosition, InputConfig, InputEssential, Radio, ViewString,
};

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
        layer_index: usize,
        base_index: usize,
        depth: u32,
    ) -> Html {
        let list = Rc::new(options.to_vec());
        let candidates = Rc::new(
            list.iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );
        let txt = ctx.props().txt.txt.clone();

        if let Some(buffer_option) = self.radio_buffer.get(&(base_index + layer_index)) {
            let checked_index = buffer_option.try_borrow().ok().and_then(|buffer_option| {
                options
                    .iter()
                    .position(|x| x.to_string() == buffer_option.as_str())
            });

            let checked_children_group_data = input_data.try_borrow().ok().and_then(|input_data| {
                if let (Some(checked_index), InputItem::Radio(_, children_group)) =
                    (checked_index, &*input_data)
                {
                    children_group.get(checked_index).cloned()
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
                        <div class="input-radio-title">
                            { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                        </div>
                        <div class="input-radio-radio">
                            // { format!("{}:{}", base_index, layer_index) }
                            <Radio::<Self>
                                txt={ctx.props().txt.clone()}
                                language={ctx.props().language}
                                parent_message={Some(Message::InputRadio(base_index + layer_index, input_data.clone()))}
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
                        if let Some(children) = children {
                            if children_data.is_empty() {
                                html! {}
                            } else {
                                html! {
                                    <div class="input-checkbox-children">
                                    {
                                        for children.iter().enumerate().map(|(sub_index, child)|
                                            if let Some(child_data) = children_data.get(sub_index) {
                                                self.view_child(ctx, child, child_data, layer_index, base_index, sub_index, depth, class_child, class_line)
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
                        { self.view_required_msg(ctx, base_index + layer_index) }
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
        always: Option<CheckStatus>,
        children: &Option<(ChildrenPosition, Vec<Rc<InputConfig>>)>,
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
        both_border: Option<bool>,
        depth: u32,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let input_data_msg = input_data.clone();
        let onclick = ctx.link().callback(move |_| {
            Message::ClickCheckbox(base_index + layer_index, input_data_msg.clone())
        });
        let checked = if let Ok(data) = input_data.try_borrow() {
            if let InputItem::Checkbox(checked, _) = (*data).clone() {
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

        // Since dynamic titles for checkbox are not included in language files, the below is reuqired.
        let title = get_text!(txt, ctx.props().language.tag(), ess.title())
            .map_or(ess.title.clone(), |text| text.to_string());

        if let Some(checked) = checked {
            html! {
                <div class={class}>
                    <div class={class_align}>
                    {
                        if always == Some(CheckStatus::Checked) || always == Some(CheckStatus::Unchecked) {
                            html! {
                                <div class={classes!("input-checkbox-me", class_me)}>
                                    <Checkbox
                                        status={checked}
                                        always={always}
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
                            } else if let (Some((position, children)), Ok(input_data)) = (children, input_data.try_borrow()) {
                                html! {
                                    for children.iter().enumerate().map(|(sub_index, child)| {
                                        let child_data = if let InputItem::Checkbox(_, childs) = input_data.clone() {
                                            childs.get(sub_index).cloned()
                                        } else {
                                            None
                                        };
                                        let class_line = if *position == ChildrenPosition::Right {
                                            if sub_index == 0 {
                                                "input-checkbox-link-line-right"
                                            } else {
                                                "input-checkbox-link-line"
                                            }
                                        } else {
                                            "input-checkbox-link-line"
                                        };
                                        if let Some(child_data) = child_data {
                                            self.view_child(ctx, child, &child_data, layer_index, base_index, sub_index, depth, class_child, class_line)
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
    pub(super) fn view_child(
        &self,
        ctx: &Context<Self>,
        child: &Rc<InputConfig>,
        child_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
        sub_index: usize,
        depth: u32,
        class_child: &'static str,
        class_line: &'static str,
    ) -> Html {
        match &**child {
            InputConfig::Checkbox(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}> // TODO: remove this empty div
                        </div>
                        { self.view_checkbox(ctx, &config.ess, config.always, &config.children, child_data, sub_index, (base_index + layer_index) * MAX_PER_LAYER, None, depth + 1) }
                    </div>
                }
            }
            InputConfig::Radio(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_radio(ctx, &config.ess, &config.options, &config.children_group, child_data, sub_index, (base_index + layer_index) * MAX_PER_LAYER, depth + 1) }
                    </div>
                }
            }
            InputConfig::HostNetworkGroup(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_host_network_group(ctx, &config.ess, config.kind, config.num, config.width, child_data, sub_index, (base_index + layer_index) * MAX_PER_LAYER) }
                    </div>
                }
            }
            InputConfig::Unsigned32(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_unsigned_32(ctx, &config.ess, config.min, config.max, config.width, child_data, sub_index, (base_index + layer_index) * MAX_PER_LAYER, false, false) }
                    </div>
                }
            }
            InputConfig::SelectMultiple(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_select_nic_or(ctx, &config.options, config.nic_index, &config.ess, child_data, sub_index, (base_index + layer_index) * MAX_PER_LAYER, depth) }
                    </div>
                }
            }
            InputConfig::Text(config) => {
                html! {
                    <div class={class_child}>
                        <div class={class_line}>
                        </div>
                        { self.view_text(ctx, &config.ess, config.length, config.width, child_data, sub_index, base_index, false, false) }
                    </div>
                }
            }
            _ => html! {},
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
        group_type: &[Rc<InputConfig>],
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
    ) -> Html {
        let input_data_clone = input_data.clone();
        let input_data_clone_1 = &(input_data.clone());
        let Ok(input_data) = input_data.try_borrow() else {
            return html! {};
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
                    { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
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
                                                                        InputConfig::Text(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_text(ctx, &ess, config.length, config.width, input_data, col_index, base_index, false, true)
                                                                        }
                                                                        InputConfig::SelectSingle(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_select_searchable(ctx, false, &ess, config.width, &config.options, input_data, col_index, base_index, 1, true)
                                                                        }
                                                                        InputConfig::VecSelect(config) => {
                                                                            self.view_vec_select(ctx, &config.ess, &config.items_ess_list, config.last, config.full_width, &config.widths, &config.max_widths, &config.max_heights, &config.map_list, input_data, col_index, base_index, true)
                                                                        }
                                                                        InputConfig::Unsigned32(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_unsigned_32(ctx, &ess, config.min, config.max, config.width, input_data, col_index, base_index, false, true)
                                                                        }
                                                                        InputConfig::Float64(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_float_64(ctx, &ess, config.step, config.width, input_data, col_index, base_index, false, true)
                                                                        }
                                                                        InputConfig::Comparison(config) => {
                                                                            let mut ess = config.ess.clone();
                                                                            ess.required = false;
                                                                            self.view_comparison(ctx, &ess, input_data, col_index, base_index, true)
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
}
