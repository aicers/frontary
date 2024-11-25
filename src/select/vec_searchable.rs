use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use yew::virtual_dom::AttrValue;
use yew::{html, Component, Context, Html, Properties};

use super::searchable::DEFAULT_SIZED_VALUE;
use crate::{language::Language, Item, SelectSearchable, SelectSearchableKind, Texts};

#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    Select(usize),
}

type List = Rc<Vec<HashMap<Vec<String>, Rc<RefCell<Vec<Item>>>>>>;

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,
    pub id: AttrValue,
    pub title: Vec<String>,
    pub kind_last: SelectSearchableKind,
    pub empty_msg: Vec<String>,
    pub top_width: Vec<u32>,
    pub max_width: Vec<u32>,
    pub max_height: Vec<u32>,
    pub list: List,
    pub selected: Vec<Rc<RefCell<Option<HashSet<String>>>>>,
    pub allow_empty: Vec<bool>,
    #[prop_or(Vec::new())]
    pub sized_value: Vec<bool>,
    pub parent_message: Vec<T::Message>,
}

#[derive(Clone, PartialEq)]
pub struct Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    phantom: PhantomData<T>,
    rerender_serial: u64,
}

impl<T> Component for Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            phantom: PhantomData,
            rerender_serial: 0,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Select(index) => {
                self.rerender_serial = self.rerender_serial.wrapping_add(1);
                for i in index + 1..ctx.props().selected.len() {
                    let Some(selected) = ctx.props().selected.get(i) else {
                        continue;
                    };
                    if let Ok(mut selected) = selected.try_borrow_mut() {
                        *selected = Some(HashSet::new());
                    }
                }
                if let (Some(parent), Some(msg)) = (
                    ctx.link().get_parent(),
                    ctx.props().parent_message.get(index),
                ) {
                    parent.clone().downcast::<T>().send_message(msg.clone());
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let keys = ctx
            .props()
            .selected
            .iter()
            .filter_map(|s| {
                s.try_borrow()
                    .ok()
                    .and_then(|s| s.as_ref().and_then(|s| s.iter().next().cloned()))
            })
            .collect::<Vec<String>>();
        let mut key = Vec::<String>::new();

        html! {
            <div class="searchable-select-vector">
            {
                for ctx.props().list.iter().enumerate().map(|(index, l)| {
                    let list = if index == 0 {
                        l.get(&Vec::new())
                    } else if let Some(k) = keys.get(index - 1) {
                        key.push(k.clone());
                        l.get(&key)
                    } else {
                        None
                    };
                    let list = list.map_or_else(|| Rc::new(RefCell::new(Vec::new())), Clone::clone);
                    let sized_value = ctx.props().sized_value.get(index).map_or(DEFAULT_SIZED_VALUE, |v| *v);
                    let (Some(title), Some(empty_msg), Some(top_width), Some(max_width), Some(max_height), Some(selected), Some(allow_empty)) = (
                        ctx.props().title.get(index),
                        ctx.props().empty_msg.get(index),
                        ctx.props().top_width.get(index),
                        ctx.props().max_width.get(index),
                        ctx.props().max_height.get(index),
                        ctx.props().selected.get(index),
                        ctx.props().allow_empty.get(index),
                    ) else {
                        return html! {};
                    };
                    let kind = if index + 1 == ctx.props().list.len() {
                        ctx.props().kind_last
                    } else {
                        SelectSearchableKind::Single
                    };
                    let class = if index + 1 == ctx.props().list.len() {
                        ""
                    } else {
                        "searchable-select-vector-margin"
                    };

                    html! {
                        <div class={class}>
                            <SelectSearchable<Self>
                                txt={ctx.props().txt.clone()}
                                language={ctx.props().language}
                                id={format!("{}-{index}-{}", ctx.props().id.clone(), self.rerender_serial)}
                                kind={kind}
                                title={title.clone()}
                                empty_msg={empty_msg.clone()}
                                top_width={*top_width}
                                max_width={*max_width}
                                max_height={*max_height}
                                list={list}
                                selected={Rc::clone(selected)}
                                allow_empty={*allow_empty}
                                default_all={false}
                                sized_value={sized_value}
                                parent_message={Message::Select(index)}
                            />
                        </div>
                    }
                })
            }
            </div>
        }
    }
}
