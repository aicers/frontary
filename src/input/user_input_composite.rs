use super::{
    component::{Message, Model},
    user_input::{view_asterisk, MAX_PER_LAYER},
    InputItem,
};
use crate::{text, CheckBox, CheckStatus, ChildrenPosition, InputEssential, InputType};
use json_gettext::get_text;
use std::cell::RefCell;
use std::rc::Rc;
use yew::{classes, html, Component, Context, Html};

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
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
                                    <CheckBox
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
                                        <CheckBox
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
                            } else if let (Some(children), Ok(input_data)) = (children, input_data.try_borrow()) {
                                html! {
                                    for children.1.iter().enumerate().map(|(sub_index, child)| {
                                        let child_data = if let InputItem::CheckBox(_, childs) = input_data.clone() {
                                            childs.and_then(|childs| childs.get(sub_index).cloned())
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
                                                                        InputType::Text(ess, length, width) =>{
                                                                            let mut ess = ess.clone();
                                                                            ess.required = false;
                                                                            self.view_text(ctx, &ess, *length, *width, input_data, col_index, base_index, false, true)
                                                                        }
                                                                        InputType::SelectSingle(ess, list, width) => {
                                                                            let mut ess = ess.clone();
                                                                            ess.required = false;
                                                                            self.view_select_searchable(ctx, false, &ess, *width, list, input_data, col_index, base_index, 1, true)
                                                                        }
                                                                        InputType::VecSelect(ess, ess_list, last_multi, list, width, width_list, max_width_list, max_height_list) => {
                                                                            self.view_vec_select(ctx, ess, ess_list, *last_multi, *width, width_list, max_width_list, max_height_list, list, input_data, col_index, base_index, true)
                                                                        }
                                                                        InputType::Unsigned32(ess, min, max, width) => {
                                                                            let mut ess = ess.clone();
                                                                            ess.required = false;
                                                                            self.view_unsigned_32(ctx, &ess, *min, *max, *width, input_data, col_index, base_index, false, true)
                                                                        }
                                                                        InputType::Float64(ess, step, width) => {
                                                                            let mut ess = ess.clone();
                                                                            ess.required = false;
                                                                            self.view_float_64(ctx, &ess, *step, *width, input_data, col_index, base_index, false, true)
                                                                        }
                                                                        InputType::Comparison(ess) => {
                                                                            let mut ess = ess.clone();
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
