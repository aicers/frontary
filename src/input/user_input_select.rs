use super::{
    component::{Message, Model},
    user_input::view_asterisk,
    InputItem,
};
use crate::{
    text, InputEssential, Item, SelectSearchable, SelectSearchableKind, VecSelect, ViewString,
};
use json_gettext::get_text;
use std::{cell::RefCell, cmp::Ordering, collections::HashMap, rc::Rc};
use yew::{html, Component, Context, Html};

const PADDING_SUM: u32 = 66; // left + right paddings
const SELECT_NIC_WIDTH: u32 = 130;

pub(super) type VecSelectListMap = HashMap<Vec<String>, Vec<(String, ViewString)>>;

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
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

    #[allow(clippy::too_many_arguments)]
    pub(super) fn view_vec_select(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        ess_list: &[InputEssential],
        last_multi: bool,
        width: Option<u32>,
        width_list: &[u32],
        max_width_list: &[u32],
        max_height_list: &[u32],
        list: &[VecSelectListMap],
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
        group: bool,
    ) -> Html {
        let title = ess_list
            .iter()
            .map(|ess| ess.title.to_string())
            .collect::<Vec<String>>();
        let empty_msg = ess_list
            .iter()
            .map(|ess| ess.notice.to_string())
            .collect::<Vec<String>>();
        let required = ess_list
            .iter()
            .map(|ess| ess.required)
            .collect::<Vec<bool>>();
        let kind_last = if last_multi {
            SelectSearchableKind::Multi
        } else {
            SelectSearchableKind::Single
        };
        let txt = ctx.props().txt.txt.clone();
        let list = list
            .iter()
            .map(|h| {
                h.iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            Rc::new(RefCell::new(
                                v.iter()
                                    .map(|(a, b)| Item {
                                        id: a.clone(),
                                        value: b.clone(),
                                    })
                                    .collect::<Vec<Item>>(),
                            )),
                        )
                    })
                    .collect::<HashMap<Vec<String>, Rc<RefCell<Vec<Item>>>>>()
            })
            .collect::<Vec<_>>();
        let Some(selected) = self.vec_select_buffer.get(&(base_index + layer_index)) else {
            return html! {};
        };
        let parent_message = selected
            .iter()
            .enumerate()
            .map(|(index, _)| {
                Message::InputVecSelect(base_index + layer_index, index, input_data.clone())
            })
            .collect::<Vec<_>>();
        let class_item = if group { "" } else { "input-select-vector" };
        let class_vec = if self.required_msg.get(&(base_index + layer_index)).is_some() {
            "input-select-vector-vec-required"
        } else {
            "input-select-vector-vec"
        };
        let style = if let Some(width) = width {
            format!("width: {width}px;")
        } else {
            "width: 100%;".to_string()
        };

        html! {
            <div class={class_item} style={style}>
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
                <div class={class_vec}>
                    <VecSelect<Self>
                        txt={ctx.props().txt.clone()}
                        language={ctx.props().language}
                        id={format!("VecSelect-{layer_index}-{base_index}")}
                        title={title}
                        kind_last={kind_last}
                        empty_msg={empty_msg}
                        top_width={width_list.to_vec()}
                        max_width={max_width_list.to_vec()}
                        max_height={max_height_list.to_vec()}
                        allow_empty={required}
                        list={Rc::new(list)}
                        selected={selected.clone()}
                        parent_message={parent_message}
                    />
                </div>
                { self.view_required_msg(ctx, base_index + layer_index) }
            </div>
        }
    }
}
