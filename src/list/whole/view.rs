use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;

use htmlescape::decode_html;
use json_gettext::get_text;
use yew::{Component, Context, Html, classes, html, virtual_dom::AttrValue};

use super::{
    DEFAULT_NUM_PAGES,
    component::{Message, Model},
};
use crate::{
    CheckStatus, Checkbox, InputConfig, MoreAction, NBSP, Pages, SelectMini, SelectMiniKind, Sort,
    SortStatus, ViewString, WholeList,
    list::{ColWidths, Column, DataType, Kind, ListItem, ModalDisplay},
    text,
};

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    #[allow(clippy::too_many_lines)]
    pub(super) fn view_head(&self, ctx: &Context<Self>) -> Html {
        let onclick_all = ctx.link().callback(|_| Message::CheckAll);
        let onclick_all_second = ctx.link().callback(|_| Message::CheckAllSecond);
        let check_status = self.check_status(ctx);
        let mut colspan = 0;
        let rowspan = ctx.props().display_info.widths.len().to_string();

        html! {
            <>
                <tr class="list-whole-head">
                    <td class="list-whole-head-check" rowspan={rowspan.clone()}>
                        <div onclick={onclick_all}>
                            <Checkbox
                                status={check_status}
                            />
                        </div>
                    </td>
                    {
                        if ctx.props().kind == Kind::LayeredFirst {
                            let check_status_second = self.check_status_second.try_borrow().map_or(CheckStatus::Unchecked, |s| *s);
                            colspan += 1;

                            html! {
                                <td class="list-whole-head-check">
                                    <div onclick={onclick_all_second}>
                                        <Checkbox
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
                        if let Some(widths) = ctx.props().display_info.widths.first() {
                            if let ColWidths::Pixel(ws) = widths {
                                colspan += ws.len();
                            }

                            html! {
                                self.view_head_row(ctx, 0, widths)
                            }
                        } else {
                            html! {}
                        }
                    }
                    <td class="list-whole-head-last-column" rowspan={rowspan}>
                    </td>
                </tr>
                {
                    if ctx.props().display_info.widths.len() > 1 {
                        let mut sum_cols: usize = 0;

                        html! {
                            for ctx.props().display_info.widths.iter().enumerate().map(|(row, cols)| {
                                let first = sum_cols;
                                sum_cols += cols.len();
                                let style = if let ColWidths::Pixel(cols) = cols {
                                    format!("width: {}px;", cols.iter().filter_map(|x| *x).sum::<u32>())
                                } else {
                                    "width: 100%;".to_string()
                                };
                                let class = if row + 1 == ctx.props().display_info.widths.len() {
                                    "list-whole-head-last-line"
                                } else {
                                    ""
                                };

                                if row == 0 {
                                    html! {}
                                } else {
                                    html! {
                                        <tr class={class} >
                                            <td colspan={colspan.to_string()} class="list-whole-head-colspan">
                                                <div class="list-whole-head-next-lines">
                                                    <table style={style}>
                                                        <tr class="list-whole-head-next-lines">
                                                            { self.view_head_row(ctx, first, cols) }
                                                        </tr>
                                                    </table>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }
                            })
                        }
                    } else {
                        html! {}
                    }
                }
            </>
        }
    }

    pub(super) fn view_head_row(
        &self,
        ctx: &Context<Self>,
        start: usize,
        widths: &ColWidths,
    ) -> Html {
        let txt = ctx.props().txt.txt.clone();
        let varied_width = Self::varied_width(ctx, widths);
        html! {
            for (0..widths.len()).map(|i| {
                let index = start + i;
                if let Some(title) = ctx.props().display_info.titles.get(index) {
                    let class_border = if i > 0 {
                        "list-whole-head-title-border"
                    } else {
                        ""
                    };
                    let style = if cfg!(feature = "pumpkin") {
                        format!("width: {}px;", ctx.props().display_info.width_full)
                    } else {
                        Self::style_width_height(ctx, widths, i, varied_width)
                    };
                    let style_inner = format!("width: 100%; height: {}px", ctx.props().display_info.height);
                    let onclick_sort = |index: usize| ctx.link().callback(move |_| Message::ClickSort(index));

                    html! {
                        <td class={classes!("list-whole-head-title", class_border)} style={style} onclick={onclick_sort(index)}>
                            <table style={style_inner}>
                                <tr>
                                    <td class="list-whole-head-title-inner-text">
                                        { text!(txt, ctx.props().language, *title) }
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
                } else {
                    html! {}
                }
            })
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
                                        if ctx.props().data_type == Some(DataType::Customer) {
                                            let cols = ctx.props().display_info.titles.len().to_string();
                                            let (prefix, extension) = if cfg!(feature = "pumpkin") {
                                                ("pumpkin/", "svg")
                                            } else {
                                                ("", "png")
                                            };
                                            let expand_collapse_img = if self.expand_list.contains(key) {
                                                format!("collapse-list.{extension}")
                                            } else {
                                                format!("expand-list.{extension}")
                                            };
                                            let style = format!("background-image: url('/frontary/{prefix}{expand_collapse_img}');");
                                            let onclick_expandible = |key: String| ctx.link().callback(move |_| Message::ClickExpandible(key.clone()));
                                            let list_top = if cfg!(feature = "pumpkin") {
                                                34
                                            } else {
                                                28
                                            };
                                            html! {
                                                <tr class="list-whole-first-layer">
                                                    <td class="list-whole-list-first-check">
                                                        <div onclick={onclick_item(key.clone())}>
                                                            <Checkbox
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
                                                        <span class="list-whole-list-first-layer-light">
                                                            { item.columns.get(1).map_or_else(|| html! {}, |d| Self::view_column(ctx, index, d)) }
                                                        </span>
                                                    </td>
                                                    <td class="list-whole-list-first-layer-more-action">
                                                        <div class="list-whole-list-flat-more-action">
                                                            <SelectMini::<MoreAction, Self>
                                                                txt={ctx.props().txt.clone()}
                                                                language={ctx.props().language}
                                                                parent_message={Message::DoMoreAction(key.clone())}
                                                                id={format!("more-action-alpha-{key}")}
                                                                active={true}
                                                                list={Rc::clone(&more_action_list)}
                                                                candidate_values={Rc::clone(&value_candidates)}
                                                                selected_value={Rc::clone(&self.more_action)}
                                                                selected_value_cache={self.more_action.try_borrow().ok().and_then(|x| *x)}
                                                                align_left={false}
                                                                list_top={list_top}
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
                                        let mut colspan = 0;
                                        let class = if ctx.props().display_info.widths.len() > 1 {
                                            "list-whole-list-flat-border"
                                        } else {
                                            "list-whole-list-flat"
                                        };
                                        let rowspan = ctx.props().display_info.widths.len().to_string();
                                        let list_top = if cfg!(feature = "pumpkin") {
                                            34
                                        } else {
                                            28
                                        };

                                        html! {
                                            <>
                                                <tr class={class}>
                                                    {
                                                        if ctx.props().kind == Kind::LayeredSecond {
                                                            html! {
                                                                <td class="list-whole-list-layered-second"></td>
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                    <td class="list-whole-list-flat-check" rowspan={rowspan.clone()}>
                                                        <div onclick={onclick_item(key.clone())}>
                                                            <Checkbox
                                                                status={check_status}
                                                            />
                                                        </div>
                                                    </td>
                                                    {
                                                        if let Some(widths) = ctx.props().display_info.widths.first() {
                                                            if let ColWidths::Pixel(ws) = widths {
                                                                colspan += ws.len();
                                                            }

                                                            html! {
                                                                self.view_column_row(ctx, item.columns.as_ref(), 0, widths, ctx.props().display_info.widths.len() > 1)
                                                            }
                                                        } else {
                                                            html! {}
                                                        }
                                                    }
                                                    <td class="list-whole-list-flat-more-action" rowspan={rowspan}>
                                                        <div class="list-whole-list-flat-more-action">
                                                            <SelectMini::<MoreAction, Self>
                                                                txt={ctx.props().txt.clone()}
                                                                language={ctx.props().language}
                                                                parent_message={Message::DoMoreAction(key.clone())}
                                                                id={format!("more-action-beta-{key}")}
                                                                active={true}
                                                                list={Rc::clone(&more_action_list)}
                                                                candidate_values={Rc::clone(&value_candidates)}
                                                                selected_value={Rc::clone(&self.more_action)}
                                                                selected_value_cache={self.more_action.try_borrow().ok().and_then(|x| *x)}
                                                                align_left={false}
                                                                {list_top}
                                                                kind={SelectMiniKind::MoreAction}
                                                            />
                                                        </div>
                                                    </td>
                                                </tr>
                                                {
                                                    if ctx.props().display_info.widths.len() > 1 {
                                                        let mut sum_cols: usize = 0;

                                                        html! {
                                                            for ctx.props().display_info.widths.iter().enumerate().map(|(row, cols)| {
                                                                let first = sum_cols;
                                                                sum_cols += cols.len();
                                                                let style = if let ColWidths::Pixel(cols) = cols {
                                                                    format!("width: {}px;", cols.iter().filter_map(|x| *x).sum::<u32>())
                                                                } else {
                                                                    "width: 100%;".to_string()
                                                                };

                                                                if row == 0 {
                                                                    html! {}
                                                                } else {
                                                                    let height = format!("height: {}px;", ctx.props().display_info.height);

                                                                    html! {
                                                                        <tr>
                                                                            <td colspan={colspan.to_string()} class="list-whole-list-colspan">
                                                                                <div class="list-whole-column-next-lines">
                                                                                    <table style={style}>
                                                                                        <tr style={height}>
                                                                                            { self.view_column_row(ctx, item.columns.as_ref(), first, cols, true) }
                                                                                        </tr>
                                                                                    </table>
                                                                                </div>
                                                                            </td>
                                                                        </tr>
                                                                    }
                                                                }
                                                            })
                                                        }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </>
                                        }
                                    }
                                }
                            }
                            {
                                if ctx.props().kind == Kind::LayeredFirst && ctx.props().data_type == Some(DataType::Customer) && self.expand_list.contains(key) {
                                    if let Some(pages_info) = self.pages_info_second.get(key) {
                                        let data = Rc::new(item.sub_items.iter().enumerate().map(|(index, item)| {
                                            // HIGHLIGHT: use the first item as the key
                                            let key = if let Some(Column::Text(txt)) = item.first() {
                                                txt.text.to_string()
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
                                            Some(input_conf),
                                            Some(second_data),
                                            Ok(check_status_second),
                                        ) = (
                                            ctx.props().input_second_add_title,
                                            ctx.props().input_second_edit_title,
                                            ctx.props().input_second_width,
                                            ctx.props().input_second_height,
                                            ctx.props().input_second_type.as_ref(),
                                            ctx.props().input_second_data.as_ref(),
                                            self.check_status_second.try_borrow(),
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
                                                    check_status_second_cache={Some(*check_status_second)}

                                                    input_ids={Rc::clone(&ctx.props().input_ids)}
                                                    input_second_keys={Rc::clone(&ctx.props().input_second_keys)}
                                                    input_data={ctx.props().input_data.clone()}
                                                    input_add_title={add_title}
                                                    input_edit_title={edit_title}
                                                    input_width={width}
                                                    input_height={height}
                                                    input_conf={input_conf.clone()}
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

    #[must_use]
    fn varied_width(ctx: &Context<Self>, widths: &ColWidths) -> Option<u32> {
        match widths {
            ColWidths::Pixel(widths) => Some(
                ctx.props().display_info.width_full - widths.iter().filter_map(|x| *x).sum::<u32>(),
            ),
            ColWidths::Ratio(_) => None,
        }
    }

    #[must_use]
    fn style_width_height(
        ctx: &Context<Self>,
        widths: &ColWidths,
        index: usize,
        varied_width: Option<u32>,
    ) -> String {
        let width = match widths {
            ColWidths::Pixel(ws) => ws.get(index).map_or_else(String::new, |w| {
                w.map_or_else(
                    || {
                        varied_width
                            .as_ref()
                            .map_or_else(String::new, |v| format!("width: {v}px;"))
                    },
                    |w| format!("width: {w}px;"),
                )
            }),
            ColWidths::Ratio(ws) => ws.get(index).map_or_else(String::new, |w| {
                w.map_or_else(String::new, |w| {
                    format!("width: {:.0}%;", (w * 100.0).trunc())
                })
            }),
        };
        let height = format!("height: {}px;", ctx.props().display_info.height);
        format!("{width} {height}")
    }

    pub(super) fn view_column_row(
        &self,
        ctx: &Context<Self>,
        columns: &[Column],
        start: usize,
        widths: &ColWidths,
        border: bool,
    ) -> Html {
        let varied_width = Self::varied_width(ctx, widths);

        html! {
            for (0..widths.len()).map(|i| {
                let index = start + i;
                if let Some(col) = columns.get(index) {
                    let class_border = if start == 0 {
                        "list-whole-column-next-lines-first"
                    } else if i > 0 {
                        "list-whole-column-next-lines"
                    } else {
                        ""
                    };
                    let style = Self::style_width_height(ctx, widths, i, varied_width);
                    let onclick_close = {
                        ctx.link().callback(move |_| Message::CloseModal)
                    };
                    html! {
                        <td class={classes!("list-whole-list-flat", if border { class_border } else { "" })} style={style}>
                            { Self::view_column(ctx, index, col) }
                            {
                                if let Some(modal) = &self.modal {
                                    let modal_content = Some(Html::from_html_unchecked(AttrValue::from_str(&modal.1).expect("AttrValue never returns Err.")));
                                    html! {
                                        <div class="cell-modal-container">
                                            <div class="cell-modal-wrapper">
                                                <div class="cell-modal-header">
                                                    <span>{&modal.0}</span>
                                                    <div onclick={onclick_close}>
                                                        <div class="complex-select-pop-head-close-icon"></div>
                                                    </div>
                                                </div>
                                                <div class="cell-modal-body">
                                                    { modal_content }
                                                </div>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </td>
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
                                pages_info_cache={self.pages_info}
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
                        let text_key = if ctx.props().data_type == Some(DataType::Customer) {
                            "Add a network".to_string()
                        } else {
                            format!("Add a(n) {}", title.to_lowercase())
                        };
                        text!(txt, ctx.props().language, text_key).to_string()
                    } else {
                        text!(txt, ctx.props().language, "Add an item").to_string()
                    }
                }
                _ => unreachable!(),
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
                <tr class="list-whloe-list-pages-outer">
                    <td class="list-whole-list-second-page-checkbox"></td>
                    <td class="list-whole-list-second-page-caret-down"></td>
                    <td colspan={cols.to_string()} class="list-whole-list-second-pages">
                        <div class="list-whole-list-pages-inner">
                            <Pages::<Self>
                                txt={ctx.props().txt.clone()}
                                language={ctx.props().language}
                                parent_message={Message::MovePage}
                                pages_info={Rc::clone(&ctx.props().pages_info)}
                                pages_info_cache={self.pages_info}
                                num_pages={DEFAULT_NUM_PAGES}
                            />
                            <div class="list-whole-list-second-add" onclick={onclick_add_second}>
                                { add_text }
                            </div>
                            { self.view_delete_checked(ctx, msg) }
                        </div>
                    </td>
                    <td class="list-whole-list-second-page-last-column"></td>
                </tr>
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn view_column(ctx: &Context<Self>, index: usize, col: &Column) -> Html {
        let txt = ctx.props().txt.txt.clone();
        match col {
            Column::Text(elem) => {
                if let Some(display) = &elem.display {
                    let v_node = Html::from_html_unchecked(
                        AttrValue::from_str(display).expect("AttrValue never returns Err."),
                    );
                    html! {
                        <div>
                            { v_node }
                        </div>
                    }
                } else {
                    html! {
                        elem.text.to_string_txt(&txt, ctx.props().language)
                    }
                }
            }
            Column::HostNetworkGroup(elem) => {
                html! {
                    for elem.host_network_group.iter().map(|elem| html! {
                        <>
                            { elem.clone() } <br/>
                        </>
                    })
                }
            }
            Column::SelectSingle(elem) => {
                let Some((_, value)) = elem.selected.as_ref() else {
                    return html! {};
                };
                if let Some(display) = &elem.display {
                    let v_node = Html::from_html_unchecked(
                        AttrValue::from_str(display).expect("AttrValue never returns Err."),
                    );
                    html! {
                        <div>
                            { v_node }
                        </div>
                    }
                } else {
                    html! {
                        value.to_string_txt(&txt, ctx.props().language)
                    }
                }
            }
            Column::SelectMultiple(list) => {
                let mut list = list
                    .selected
                    .values()
                    .map(|v| v.to_string_txt(&txt, ctx.props().language))
                    .collect::<Vec<String>>();
                list.sort_unstable();
                view_list_sep_dot(&list, false)
            }
            Column::Tag(tags) => {
                let mut list = tags
                    .tags
                    .iter()
                    .filter_map(|t| {
                        ctx.props().input_conf.get(index).and_then(|x| {
                            if let InputConfig::Tag(config) = &**x {
                                config.name_map.get(t).cloned()
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
            Column::Unsigned32(_)
            | Column::Unsigned8(_)
            | Column::Float64(_)
            | Column::Percentage(_)
            | Column::Comparison(_) => html! { col.to_string() },
            Column::Nic(nics) => {
                html! {
                    for nics.nics.iter().map(|n| html! {
                        <>
                            { n.name.clone() } {": (ip/mask) "} { n.interface.clone() } { " (gateway) " } { n.gateway.clone() } <br/>
                        </>
                    })
                }
            }
            Column::File(elem) => html! {
                elem.filename.clone()
            },
            Column::VecSelect(list) => {
                let list = list
                    .selected
                    .iter()
                    .map(|s| {
                        s.values()
                            .map(|v| v.to_string_txt(&txt, ctx.props().language))
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .collect::<Vec<_>>();
                view_list_sep_dot(&list, false)
            }
            Column::Group(group) => {
                let Some(input_conf) = ctx.props().input_conf.get(index) else {
                    return html! {};
                };
                let InputConfig::Group(config) = &**input_conf else {
                    return html! {};
                };

                html! {
                    <table class="list-whole-group">
                        <tr>
                        {
                            for config.items.iter().map(|t| html! {
                                <th class="list-whole-group-heading">
                                    { text!(txt, ctx.props().language, t.title()) }
                                </th>
                            })
                        }
                        </tr>
                        {
                            for group.groups.iter().map(|g|
                                html! {
                                    <tr>
                                    {
                                        for g.iter().map(|c|
                                            match c {
                                                Column::Text(..)
                                                | Column::HostNetworkGroup(..)
                                                | Column::SelectSingle(..)
                                                | Column::SelectMultiple(..)
                                                | Column::Unsigned32(..)
                                                | Column::Unsigned8(..)
                                                | Column::Float64(..)
                                                | Column::Percentage(..)
                                                | Column::Comparison(..)
                                                | Column::VecSelect(..)
                                                | Column::File(..)  => html! {
                                                    <td class="list-whole-group">
                                                        { Self::view_column(ctx, index, c) }
                                                    </td>
                                                },
                                                Column::Tag(..)
                                                | Column::Nic(..)
                                                | Column::Group(..)
                                                | Column::Checkbox(..)
                                                | Column::Radio(..) => {
                                                    panic!("Column Group does not support some items such as Tag, Nic, Group, Checkbox, and Radio.")
                                                }
                                            }

                                        )
                                    }
                                    </tr>
                                }
                            )
                        }
                    </table>
                }
            }
            Column::Checkbox(elem) => {
                if elem.modal.is_empty() {
                    if elem.display.is_empty() {
                        html! {
                            "-"
                        }
                    } else {
                        html! {
                            {
                                for elem.display.iter().map(|display| {
                                    html! {
                                        <div>
                                            { display }
                                        </div>
                                    }
                                })
                            }
                        }
                    }
                } else {
                    Self::to_unchecked_html(ctx, &elem.display, &elem.modal)
                }
            }
            Column::Radio(elem) => {
                if elem.display.is_empty() {
                    html! {
                        elem.selected.to_string_txt(&txt, ctx.props().language)
                    }
                } else {
                    Self::to_unchecked_html(ctx, &elem.display, &elem.modal)
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
                                    <img src={if cfg!(feature = "pumpkin") { "/frontary/pumpkin/delete-trash.svg" } else { "/frontary/delete-trash-white.png" }} class="list-whole-delete-trash-white" />
                                </div>
                            </td>
                            <td class="list-whole-delete-checked-close">
                                <div class="list-whole-delete-checked-close" onclick={onclick_cancel}>
                                    <img src={if cfg! (feature = "pumpkin") {"/frontary/pumpkin/close-white.svg"} else {"/frontary/close-white.png"} } class="list-whole-close-white" />
                                </div>
                            </td>
                        </tr>
                    </table>
                </div>
            }
        }
    }

    fn to_unchecked_html(ctx: &Context<Self>, display: &[String], modal: &[ModalDisplay]) -> Html {
        html! {
            for display.iter().enumerate().map(|(index, d)| {
                let modal_data = modal.get(index).map(|modal| (modal.title.clone(), modal.content.clone()));
                let onclick_button = {
                    ctx.link()
                        .callback(move |_| Message::ClickButton(modal_data.clone()))
                };
                let v_node = Html::from_html_unchecked(AttrValue::from_str(d).expect("AttrValue never returns Err."));
                html! {
                    <div onclick={onclick_button.clone()}>
                        { v_node }
                    </div>
                }
            })
        }
    }
}

fn view_list_sep_dot(list: &[String], br: bool) -> Html {
    html! {
        for list.iter().enumerate().map(|(index, item)| html! {
            <>
                { item.clone() }
                {
                    if index < list.len() - 1 {
                        html! {
                            <>
                                <span class="list-whole-list-multiple-sep">
                                    { " • " }
                                </span>
                                {
                                    if br {
                                        html! {
                                            <> <br/> </>
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
                }
            </>
        })
    }
}
