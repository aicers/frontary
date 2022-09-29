use super::{
    component::{Message, Model},
    DEFAULT_NUM_PAGES,
};
use crate::{
    list::{Column, DataType, Kind, ListItem},
    text, CheckBox, CheckStatus, InputType, MoreAction, Pages, SelectMini, SelectMiniKind, Sort,
    SortStatus, ViewString, WholeList, NBSP,
};
use htmlescape::decode_html;
use json_gettext::get_text;
use std::collections::HashMap;
use std::rc::Rc;
use yew::{html, Component, Context, Html};

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    #[allow(clippy::too_many_lines)]
    pub(super) fn view_head(&self, ctx: &Context<Self>) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let varied_width: u32 = ctx.props().display_info.width_full
            - ctx
                .props()
                .display_info
                .width_cols
                .iter()
                .filter_map(Clone::clone)
                .collect::<Vec<u32>>()
                .iter()
                .sum::<u32>();
        let onclick_all = ctx.link().callback(|_| Message::CheckAll);
        let onclick_all_second = ctx.link().callback(|_| Message::CheckAllSecond);

        let check_status = self.check_status(ctx);
        html! {
            <tr class="list-whole-head">
                <td class="list-whole-head-check">
                    <div onclick={onclick_all}>
                        <CheckBox
                            status={check_status}
                        />
                    </div>
                </td>
                {
                    if ctx.props().kind == Kind::LayeredFirst {
                        let check_status_second = self.check_status_second.try_borrow().map_or(CheckStatus::Unchecked, |s| *s);
                        html! {
                            <td class="list-whole-head-check">
                                <div onclick={onclick_all_second}>
                                    <CheckBox
                                        status={check_status_second}
                                    />
                                </div>
                            </td>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    for ctx.props().display_info.titles.iter().enumerate().map(|(index, title)| {
                        let width = ctx.props().display_info.width_cols.get(index).map_or_else(
                            String::new,
                            |w| w.map_or_else(|| format!("width: {}px;", varied_width), |w| format!("width: {}px;", w),
                        ));
                        let height = ctx.props().display_info.height_cols.get(index).map_or_else(
                            String::new,
                            |w| w.map_or_else(String::new, |w| format!("height: {}px;", w),
                        ));
                        let style = format!("{} {}", width, height);
                        let style_inner = if ctx.props().kind == Kind::LayeredFirst && !self.expand_list.is_empty() {
                            let width_inner = ctx.props().display_info.width_cols.get(index).map_or_else(
                                String::new,
                                |w| w.map_or_else(|| format!("width: {}px;", varied_width - 20), |w| format!("width: {}px;", w - 20)),
                            );
                            let height_inner = ctx.props().display_info.height_cols.get(index).map_or_else(
                                String::new,
                                |w| w.map_or_else(String::new, |w| format!("height: {}px;", w))
                            );
                            format!("{} {}", width_inner, height_inner)
                        } else {
                            style.clone()
                        };

                        let onclick_sort = |index: usize| ctx.link().callback(move |_| Message::ClickSort(index));
                        html! {
                            <td class="list-whole-head-title" style={style}  onclick={onclick_sort(index)}>
                                <table>
                                    <tr>
                                        <td class="list-whole-head-title-inner-text" style={style_inner}>
                                            { text!(txt, ctx.props().language, title) }
                                        </td>
                                        {
                                            if ctx.props().kind == Kind::LayeredFirst && !self.expand_list.is_empty()
                                                || ctx.props().kind == Kind::Flat || ctx.props().kind == Kind::LayeredSecond {
                                                let sort_status = if ctx.props().kind == Kind::LayeredFirst {
                                                    self.sort_second_layer
                                                } else {
                                                    self.sort
                                                };
                                                let sort_status = sort_status.map_or(SortStatus::Unsorted, |s| if s.index == index {
                                                    s.status
                                                } else {
                                                    SortStatus::Unsorted
                                                });
                                                html! {
                                                    <td class="list-whole-head-title-inner-sort">
                                                        <div class="list-whole-head-title-sort">
                                                            <Sort status={sort_status} />
                                                        </div>
                                                    </td>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                    </tr>
                                </table>
                            </td>
                        }
                    })
                }
                <td class="list-whole-head-last-column">
                </td>
            </tr>
        }
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn view_list(&self, ctx: &Context<Self>) -> Html {
        let (start, end) = self.item_range(ctx);
        html! {
            for (start..=end).map(|index| {
                if let Some(key) = self.sorted_keys.get(index - 1) {
                    if let Some(item) = ctx.props().data.get(key) {
                        let onclick_item = |key: String| ctx.link().callback(move |_| Message::CheckItem(key.clone()));
                        let check_status = if self.checked.contains(key) {
                            CheckStatus::Checked
                        } else {
                            CheckStatus::Unchecked
                        };
                        let more_action_list = Rc::new(vec![
                            ViewString::Key("Edit".to_string()),
                            ViewString::Key("Delete".to_string()),
                        ]);
                        let value_candidates = Rc::new(vec![
                            MoreAction::Edit,
                            MoreAction::Delete,
                        ]);

                        html! {
                            <>
                            {
                                match ctx.props().kind {
                                    Kind::LayeredFirst => {
                                        if ctx.props().data_type == DataType::Customer {
                                            let cols = ctx.props().display_info.titles.len().to_string();
                                            let style = if self.expand_list.contains(key) {
                                                "background-image: url('/img/collapse-list.png');"
                                            } else {
                                                "background-image: url('/img/expand-list.png');"
                                            };
                                            let onclick_expandible = |key: String| ctx.link().callback(move |_| Message::ClickExpandible(key.clone()));

                                            html! {
                                                <tr class="list-whole-first-layer">
                                                    <td class="list-whole-list-first-check">
                                                        <div onclick={onclick_item(key.clone())}>
                                                            <CheckBox
                                                                status={check_status}
                                                            />
                                                        </div>
                                                    </td>
                                                    <td class="list-whole-list-first-expand">
                                                        <div class="list-whole-list-first-expand" style={style} onclick={onclick_expandible(key.clone())}>
                                                        </div>
                                                    </td>
                                                    <td colspan={cols} class="list-whole-list-first-layer">
                                                        { item.columns.first().map_or_else(|| html! {}, |n| Self::view_column(ctx, index, n)) }
                                                        { decode_html(NBSP.repeat(3).as_str()).expect("safely-selected character") }
                                                        <font class="list-whole-list-first-layer-light">
                                                            { item.columns.get(1).map_or_else(|| html! {}, |d| Self::view_column(ctx, index, d)) }
                                                        </font>
                                                    </td>
                                                    <td class="list-whole-list-first-layer-more-action">
                                                        <div class="list-whole-list-flat-more-action">
                                                            <SelectMini::<MoreAction, Self>
                                                                txt={ctx.props().txt.clone()}
                                                                language={ctx.props().language}
                                                                parent_message={Message::DoMoreAction(key.clone())}
                                                                id={format!("more-action-alpha-{}", key)}
                                                                active={true}
                                                                list={Rc::clone(&more_action_list)}
                                                                candidate_values={Rc::clone(&value_candidates)}
                                                                selected_value={Rc::clone(&self.more_action)}
                                                                selected_value_cache={self.more_action.try_borrow().ok().and_then(|x| *x)}
                                                                align_left={false}
                                                                list_top={28}
                                                                kind={SelectMiniKind::MoreAction}
                                                            />
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        } else {
                                            html! {}
                                        }

                                    },
                                    Kind::Flat | Kind::LayeredSecond => {
                                        html! {
                                            <tr class="list-whole-list-flat">
                                                {
                                                    if ctx.props().kind == Kind::LayeredSecond {
                                                        html! {
                                                            <td></td>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                <td class="list-whole-list-flat-check">
                                                    <div onclick={onclick_item(key.clone())}>
                                                        <CheckBox
                                                            status={check_status}
                                                        />
                                                    </div>
                                                </td>
                                                {
                                                    for item.columns.iter().enumerate().map(|(index, col)| html! {
                                                        <td class="list-whole-list-flat">
                                                            { Self::view_column(ctx, index, col) }
                                                        </td>
                                                    })
                                                }
                                                <td class="list-whole-list-flat-more-action">
                                                    <div class="list-whole-list-flat-more-action">
                                                        <SelectMini::<MoreAction, Self>
                                                            txt={ctx.props().txt.clone()}
                                                            language={ctx.props().language}
                                                            parent_message={Message::DoMoreAction(key.clone())}
                                                            id={format!("more-action-beta-{}", key)}
                                                            active={true}
                                                            list={Rc::clone(&more_action_list)}
                                                            candidate_values={Rc::clone(&value_candidates)}
                                                            selected_value={Rc::clone(&self.more_action)}
                                                            selected_value_cache={self.more_action.try_borrow().ok().and_then(|x| *x)}
                                                            align_left={false}
                                                            list_top={28}
                                                            kind={SelectMiniKind::MoreAction}
                                                        />
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }
                                }
                            }
                            {
                                if ctx.props().kind == Kind::LayeredFirst && ctx.props().data_type == DataType::Customer && self.expand_list.contains(key) {
                                    if let Some(pages_info) = self.pages_info_second.get(key) {
                                        let data = Rc::new(item.sub_items.iter().enumerate().map(|(index, item)| {
                                            // HIGHLIGHT: use the first item as the key
                                            let key = if let Some(Column::Text(txt)) = item.first() {
                                                txt.to_string()
                                            } else {
                                                index.to_string()
                                            };
                                            (
                                                key,
                                                ListItem {
                                                    columns: item.clone(),
                                                    sub_items: Vec::<_>::new(),
                                                    creation_time: None,
                                                },
                                            )
                                        }).collect::<HashMap<String, ListItem>>());
                                        let id = format!("customer-{}", key.clone());
                                        if let (
                                            Some(add_title),
                                            Some(edit_title),
                                            Some(width),
                                            Some(height),
                                            Some(input_type),
                                            Some(second_data),
                                        ) = (
                                            ctx.props().input_second_add_title,
                                            ctx.props().input_second_edit_title,
                                            ctx.props().input_second_width,
                                            ctx.props().input_second_height,
                                            ctx.props().input_second_type.as_ref(),
                                            ctx.props().input_second_data.as_ref(),
                                        ) {
                                            html! {
                                                <WholeList::<T>
                                                    txt={ctx.props().txt.clone()}
                                                    language={ctx.props().language}
                                                    id={id}
                                                    title={""}
                                                    title_second={ctx.props().title_second}
                                                    kind={Kind::LayeredSecond}
                                                    data_type={ctx.props().data_type}
                                                    data={Rc::clone(&data)}
                                                    display_info={Rc::clone(&ctx.props().display_info)}
                                                    sort={self.sort_second_layer}
                                                    pages_info={Rc::clone(pages_info)}
                                                    check_status_second={Rc::clone(&self.check_status_second)}

                                                    input_ids={Rc::clone(&ctx.props().input_ids)}
                                                    input_second_keys={Rc::clone(&ctx.props().input_second_keys)}
                                                    input_data={ctx.props().input_data.clone()}
                                                    input_add_title={add_title}
                                                    input_edit_title={edit_title}
                                                    input_width={width}
                                                    input_height={height}
                                                    input_type={input_type.clone()}
                                                    input_second_data={Some(second_data.clone())}
                                                    messages={ctx.props().messages.clone()}
                                                />
                                            }
                                        } else {
                                            html! {}
                                        }
                                    } else {
                                        html! {}
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            </>
                        }
                    } else {
                        html! {}
                    }
                } else {
                    html! {}
                }
            })
        }
    }

    pub(super) fn view_pages(&self, ctx: &Context<Self>, out_table: bool) -> Html {
        let cols = ctx.props().display_info.titles.len();
        let txt = ctx.props().txt.txt.clone();

        if out_table {
            let msg = format!(
                "{}{} {} {}{}",
                self.checked.len(),
                text!(txt, ctx.props().language, "(items of)"),
                text!(txt, ctx.props().language, "chosen"),
                text!(txt, ctx.props().language, &ctx.props().title)
                    .to_string()
                    .to_lowercase(),
                text!(txt, ctx.props().language, "(s)"),
            );
            html! {
                <tr>
                    <td colspan={(cols + 3).to_string()} class="list-whole-list-pages">
                        <div class="list-whole-list-pages-inner">
                            <Pages::<Self>
                                txt={ctx.props().txt.clone()}
                                language={ctx.props().language}
                                parent_message={Message::MovePage}
                                pages_info={Rc::clone(&ctx.props().pages_info)}
                                num_pages={DEFAULT_NUM_PAGES}
                            />
                            { self.view_delete_checked(ctx, msg) }
                        </div>
                    </td>
                </tr>
            }
        } else {
            let add_text = match ctx.props().kind {
                Kind::LayeredSecond => {
                    if let Some(title) = ctx.props().title_second {
                        let text_key = if ctx.props().data_type == DataType::Customer {
                            "Add a network".to_string()
                        } else {
                            format!("Add a(n) {}", title.to_lowercase())
                        };
                        text!(txt, ctx.props().language, text_key).to_string()
                    } else {
                        text!(txt, ctx.props().language, "Add an item").to_string()
                    }
                }
                _ => String::new(), // unreachable
            };

            let onclick_add_second = ctx.link().callback(|_| Message::InputAdd);
            let msg = format!(
                "{}{} {} {}{}",
                self.checked.len(),
                text!(txt, ctx.props().language, "(items of)"),
                text!(txt, ctx.props().language, "chosen"),
                text!(
                    txt,
                    ctx.props().language,
                    ctx.props().title_second.unwrap_or("item")
                )
                .to_string()
                .to_lowercase(),
                text!(txt, ctx.props().language, "(s)"),
            );

            html! {
                <tr>
                    <td></td>
                    <td></td>
                    <td colspan={cols.to_string()} class="list-whole-list-second-pages">
                        <div class="list-whole-list-pages-inner">
                            <Pages::<Self>
                                txt={ctx.props().txt.clone()}
                                language={ctx.props().language}
                                parent_message={Message::MovePage}
                                pages_info={Rc::clone(&ctx.props().pages_info)}
                                num_pages={DEFAULT_NUM_PAGES}
                            />
                            <div class="list-whole-list-second-add" onclick={onclick_add_second}>
                                { add_text }
                            </div>
                            { self.view_delete_checked(ctx, msg) }
                        </div>
                    </td>
                    <td></td>
                </tr>
            }
        }
    }

    fn view_column(ctx: &Context<Self>, index: usize, col: &Column) -> Html {
        let txt = ctx.props().txt.txt.clone();
        match col {
            Column::Text(elem) => match elem {
                ViewString::Key(key) => {
                    html! { text!(txt, ctx.props().language, key) }
                }
                ViewString::Raw(raw) => {
                    html! { raw }
                }
            },
            Column::Unsigned32(elem) => {
                html! {
                    { elem.map_or_else(String::new, |v| v.to_string()) }
                }
            }
            Column::HostNetworkGroup(elem) => {
                html! {
                    for elem.iter().map(|elem| html! {
                        <>
                            { elem.clone() } <br/>
                        </>
                    })
                }
            }
            Column::KeyValueList(list) => {
                let mut list = list.values().map(Clone::clone).collect::<Vec<String>>();
                list.sort_unstable();
                view_list_sep_dot(&list)
            }
            Column::Tag(tags) => {
                let mut list = tags
                    .iter()
                    .filter_map(|t| {
                        ctx.props().input_type.get(index).and_then(|x| {
                            if let InputType::Tag(_, tag_values) = &**x {
                                tag_values.get(t).cloned()
                            } else {
                                None
                            }
                        })
                    })
                    .collect::<Vec<String>>();
                list.sort_unstable();
                html! {
                    <div class="list-whole-tag">
                    {
                        for list.iter().map(|item| html! {
                            <div class="list-whole-tag-item">
                                { item.clone() }
                            </div>
                        })
                    }
                    </div>
                }
            }
        }
    }

    pub(super) fn view_delete_checked(&self, ctx: &Context<Self>, msg: String) -> Html {
        if self.check_status(ctx) == CheckStatus::Unchecked {
            html! {}
        } else {
            let onclick_delete = ctx.link().callback(|_| Message::DeleteChecked);
            let onclick_cancel = ctx.link().callback(|_| Message::CancelChecked);
            html! {
                <div class="list-whole-delete-checked">
                    <div class="list-whole-delete-checked-text">
                        { msg }
                    </div>
                    <table>
                        <tr>
                            <td class="list-whole-delete-checked-trash">
                                <div class="list-whole-delete-checked-trash" onclick={onclick_delete}>
                                    <img src="/img/delete-trash-white.png" class="list-whole-delete-trash-white" />
                                </div>
                            </td>
                            <td class="list-whole-delete-checked-close">
                                <div class="list-whole-delete-checked-close" onclick={onclick_cancel}>
                                    <img src="/img/close-white.png" class="list-whole-close-white" />
                                </div>
                            </td>
                        </tr>
                    </table>
                </div>
            }
        }
    }
}

fn view_list_sep_dot(list: &[String]) -> Html {
    html! {
        for list.iter().enumerate().map(|(index, item)| html! {
            <>
                { item.clone() }
                {
                    if index < list.len() - 1 {
                        html! {
                            <font class="list-whole-list-multiple-dot">
                                { " • " }
                            </font>
                        }
                    } else {
                        html! {}
                    }
                }
            </>
        })
    }
}
