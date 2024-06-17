use std::{cell::RefCell, collections::HashMap, rc::Rc, str::FromStr};

use json_gettext::get_text;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::{events::InputEvent, html, Component, Context, Html};

use super::{
    component::{Message, Model},
    user_input::view_asterisk,
    Comparison, ComparisonKind, InputItem, Value as ComparisonValue, ValueKind,
};
use crate::{text, InputEssential, Item, SelectSearchableKind, VecSelect, ViewString};

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    #[allow(clippy::too_many_lines)]
    pub(super) fn view_comparison(
        &self,
        ctx: &Context<Self>,
        ess: &InputEssential,
        input_data: &Rc<RefCell<InputItem>>,
        layer_index: usize,
        base_index: usize,
        group: bool,
    ) -> Html {
        let data_id = layer_index + base_index;
        let Some(value_kind_selected) = self.comparison_value_kind_buffer.get(&data_id) else {
            return html! {};
        };
        let Some(value_cmp_selected) = self.comparison_value_cmp_buffer.get(&data_id) else {
            return html! {};
        };
        let selected = vec![value_kind_selected.clone(), value_cmp_selected.clone()];
        let title = vec!["Type".to_string(), "Comparison".to_string()];
        let empty_msg = vec!["Type".to_string(), "Comparison".to_string()];
        let top_width = vec![90_u32, 110];
        let max_width = vec![200_u32, 240];
        let max_height = vec![300_u32, 300];
        let allow_empty = vec![true, true];
        let sized_value = vec![false, false];
        let mut first: HashMap<Vec<String>, Rc<RefCell<Vec<Item>>>> = HashMap::new();
        first.insert(
            Vec::new(),
            Rc::new(RefCell::new(vec![
                value_kind(ValueKind::String),
                value_kind(ValueKind::Integer),
                value_kind(ValueKind::Float),
            ])),
        );
        let mut second: HashMap<Vec<String>, Rc<RefCell<Vec<Item>>>> = HashMap::new();
        second.insert(
            vec![ValueKind::String.to_string()],
            Rc::new(RefCell::new(vec![
                cmp_kind(ComparisonKind::Contain),
                cmp_kind(ComparisonKind::NotContain),
            ])),
        );
        second.insert(
            vec![ValueKind::Integer.to_string()],
            Rc::new(RefCell::new(vec![
                cmp_kind(ComparisonKind::Less),
                cmp_kind(ComparisonKind::Equal),
                cmp_kind(ComparisonKind::Greater),
                cmp_kind(ComparisonKind::LessOrEqual),
                cmp_kind(ComparisonKind::GreaterOrEqual),
                cmp_kind(ComparisonKind::OpenRange),
                cmp_kind(ComparisonKind::CloseRange),
                cmp_kind(ComparisonKind::LeftOpenRange),
                cmp_kind(ComparisonKind::RightOpenRange),
                cmp_kind(ComparisonKind::NotEqual),
                cmp_kind(ComparisonKind::NotOpenRange),
                cmp_kind(ComparisonKind::NotCloseRange),
                cmp_kind(ComparisonKind::NotLeftOpenRange),
                cmp_kind(ComparisonKind::NotRightOpenRange),
            ])),
        );
        second.insert(
            vec![ValueKind::Float.to_string()],
            Rc::new(RefCell::new(vec![
                cmp_kind(ComparisonKind::Less),
                cmp_kind(ComparisonKind::Equal),
                cmp_kind(ComparisonKind::Greater),
                cmp_kind(ComparisonKind::LessOrEqual),
                cmp_kind(ComparisonKind::GreaterOrEqual),
                cmp_kind(ComparisonKind::OpenRange),
                cmp_kind(ComparisonKind::CloseRange),
                cmp_kind(ComparisonKind::LeftOpenRange),
                cmp_kind(ComparisonKind::RightOpenRange),
                cmp_kind(ComparisonKind::NotEqual),
                cmp_kind(ComparisonKind::NotOpenRange),
                cmp_kind(ComparisonKind::NotCloseRange),
                cmp_kind(ComparisonKind::NotLeftOpenRange),
                cmp_kind(ComparisonKind::NotRightOpenRange),
            ])),
        );
        let list = Rc::new(vec![first, second]);
        let parent_message = vec![
            Message::InputComparisonValueKind(data_id, input_data.clone()),
            Message::InputComparisonComparisionKind(data_id, input_data.clone()),
        ];
        let txt = ctx.props().txt.txt.clone();

        html! {
            <div class="input-comparison-outer">
                {
                    if group {
                        html! {}
                    } else {
                        html! {
                            <div class="input-contents-item-title">
                                { text!(txt, ctx.props().language, ess.title()) }{ view_asterisk(ess.required) }
                            </div>
                        }
                    }
                }
                <div class="input-comparison">
                    <VecSelect<Self>
                        txt={ctx.props().txt.clone()}
                        language={ctx.props().language}
                        id={format!("VecSelect-{layer_index}-{base_index}")}
                        title={title}
                        kind_last={SelectSearchableKind::Single}
                        empty_msg={empty_msg}
                        top_width={top_width}
                        max_width={max_width}
                        max_height={max_height}
                        allow_empty={allow_empty}
                        sized_value={sized_value}
                        list={list}
                        selected={selected}
                        parent_message={parent_message}
                    />
                    { self.view_comparison_value(ctx, input_data, data_id) }
                </div>
                { self.view_required_msg(ctx, data_id) }
            </div>
        }
    }

    fn view_comparison_value(
        &self,
        ctx: &Context<Self>,
        input_data: &Rc<RefCell<InputItem>>,
        data_id: usize,
    ) -> Html {
        let (Some(value_kind), Some(cmp_kind)) =
            (self.comparison_kind(data_id), self.comparison_cmp(data_id))
        else {
            return html! {
                <div class="input-comparison-value">
                </div>
            };
        };

        if cmp_kind.chain_cmp() {
            html! {
                <div class="input-comparison-value">
                    { cmp_statement_head(cmp_kind) }
                    <div class="input-comparison-value-value">
                        { self.view_comparison_value_each(ctx, input_data, data_id, 0, value_kind) }
                    </div>
                    { cmp_statement_symbol(cmp_kind) }
                    <div class="input-comparison-value-value">
                        { self.view_comparison_value_each(ctx, input_data, data_id, 1, value_kind) }
                    </div>
                    { cmp_statement_tail(cmp_kind) }
                </div>
            }
        } else {
            html! {
                <div class="input-comparison-value">
                    { cmp_statement_symbol(cmp_kind) }
                    <div class="input-comparison-value-value">
                        { self.view_comparison_value_each(ctx, input_data, data_id, 0, value_kind) }
                    </div>
                </div>
            }
        }
    }

    fn view_comparison_value_each(
        &self,
        ctx: &Context<Self>,
        input_data: &Rc<RefCell<InputItem>>,
        data_id: usize,
        value_index: usize,
        value_kind: ValueKind,
    ) -> Html {
        let input_data_clone = input_data.clone();
        let oninput = ctx.link().callback(move |e: InputEvent| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .map_or(Message::InputError, |input| {
                    let value = match value_kind {
                        ValueKind::String => Some(ComparisonValue::String(Some(input.value()))),
                        ValueKind::Integer => {
                            if let Ok(value) = input.value().parse::<i64>() {
                                Some(ComparisonValue::Integer(Some(value)))
                            } else {
                                None
                            }
                        }
                        ValueKind::Float => {
                            if let Ok(value) = input.value().parse::<f64>() {
                                Some(ComparisonValue::Float(Some(value)))
                            } else {
                                None
                            }
                        }
                    };
                    if let Some(value) = value {
                        Message::InputComparisonValue(
                            data_id,
                            value_index,
                            value,
                            input_data_clone.clone(),
                        )
                    } else {
                        Message::InvalidInputComparisonValue
                    }
                })
        });
        let value = if let Some((first, second)) = self.comparison_value_buffer.get(&data_id) {
            let value = if value_index == 0 { first } else { second };
            value.try_borrow().map_or_else(
                |_| String::new(),
                |value| {
                    if let Some(value) = &*value {
                        value.to_string()
                    } else {
                        String::new()
                    }
                },
            )
        } else {
            String::new()
        };

        match value_kind {
            ValueKind::String => html! {
                <input type="text"
                    class="frontary-input-text"
                    oninput={oninput}
                    value={value}
                />
            },
            ValueKind::Integer | ValueKind::Float => html! {
                <input type="number"
                    class="input-number"
                    oninput={oninput}
                    value={value}
                />
            },
        }
    }

    pub(super) fn input_comparison_comparison_kind(
        &self,
        data_id: usize,
        input_data: &Rc<RefCell<InputItem>>,
    ) {
        let Some(kind) = self.comparison_kind(data_id) else {
            return;
        };
        let value = match kind {
            ValueKind::String => ComparisonValue::String(None),
            ValueKind::Integer => ComparisonValue::Integer(None),
            ValueKind::Float => ComparisonValue::Float(None),
        };
        self.clear_comparison_value(data_id, input_data);
        let set = if let Some(buf) = self.comparison_cmp(data_id) {
            match buf {
                ComparisonKind::Less => Some(Comparison::Less(value)),
                ComparisonKind::Equal => Some(Comparison::Equal(value)),
                ComparisonKind::Greater => Some(Comparison::Greater(value)),
                ComparisonKind::LessOrEqual => Some(Comparison::LessOrEqual(value)),
                ComparisonKind::GreaterOrEqual => Some(Comparison::GreaterOrEqual(value)),
                ComparisonKind::Contain => Some(Comparison::Contain(value)),
                ComparisonKind::CloseRange => Some(Comparison::CloseRange(value.clone(), value)),
                ComparisonKind::OpenRange => Some(Comparison::OpenRange(value.clone(), value)),
                ComparisonKind::LeftOpenRange => {
                    Some(Comparison::LeftOpenRange(value.clone(), value))
                }
                ComparisonKind::RightOpenRange => {
                    Some(Comparison::RightOpenRange(value.clone(), value))
                }
                ComparisonKind::NotEqual => Some(Comparison::NotEqual(value)),
                ComparisonKind::NotContain => Some(Comparison::NotContain(value)),
                ComparisonKind::NotOpenRange => {
                    Some(Comparison::NotOpenRange(value.clone(), value))
                }
                ComparisonKind::NotCloseRange => {
                    Some(Comparison::NotCloseRange(value.clone(), value))
                }
                ComparisonKind::NotLeftOpenRange => {
                    Some(Comparison::NotLeftOpenRange(value.clone(), value))
                }
                ComparisonKind::NotRightOpenRange => {
                    Some(Comparison::NotRightOpenRange(value.clone(), value))
                }
            }
        } else {
            None
        };

        if let Ok(mut data) = input_data.try_borrow_mut() {
            *data = InputItem::Comparison(set);
        }
    }

    pub(super) fn comparison_kind(&self, data_id: usize) -> Option<ValueKind> {
        self.comparison_value_kind_buffer
            .get(&data_id)?
            .try_borrow()
            .ok()?
            .as_ref()
            .and_then(|kind| {
                kind.iter()
                    .next()
                    .and_then(|first| ValueKind::from_str(first).ok())
            })
    }

    pub(super) fn comparison_cmp(&self, data_id: usize) -> Option<ComparisonKind> {
        self.comparison_value_cmp_buffer
            .get(&data_id)?
            .try_borrow()
            .ok()?
            .as_ref()
            .and_then(|cmp| {
                cmp.iter()
                    .next()
                    .and_then(|first| ComparisonKind::from_str(first).ok())
            })
    }

    pub(super) fn input_comparison_value(
        &mut self,
        data_id: usize,
        value_index: usize,
        value: &ComparisonValue,
        input_data: &Rc<RefCell<InputItem>>,
    ) {
        let Some(cmp) = self.comparison_cmp(data_id) else {
            return;
        };
        let Some((first, second)) = self.comparison_value_buffer.get(&data_id) else {
            return;
        };
        let (Ok(mut first), Ok(mut second)) = (first.try_borrow_mut(), second.try_borrow_mut())
        else {
            return;
        };
        if value_index == 0 {
            *first = Some(value.clone());
        } else {
            *second = Some(value.clone());
        }
        let Ok(mut input_data) = input_data.try_borrow_mut() else {
            return;
        };
        let InputItem::Comparison(input_data) = &mut *input_data else {
            return;
        };
        if cmp.chain_cmp() {
            if let (Some(first), Some(second)) = (&*first, &*second) {
                if let Ok(data) = Comparison::try_new(cmp, first.clone(), Some(second.clone())) {
                    *input_data = Some(data);
                    self.required_msg.remove(&data_id);
                }
            }
        } else if let Some(first) = &*first {
            if let Ok(data) = Comparison::try_new(cmp, first.clone(), None) {
                *input_data = Some(data);
                self.required_msg.remove(&data_id);
            }
        }
    }

    pub(super) fn clear_comparison_value(
        &self,
        data_id: usize,
        input_data: &Rc<RefCell<InputItem>>,
    ) {
        if let Ok(mut data) = input_data.try_borrow_mut() {
            *data = InputItem::Comparison(None);
        }
        let Some((first, second)) = self.comparison_value_buffer.get(&data_id) else {
            return;
        };
        if let Ok(mut first) = first.try_borrow_mut() {
            *first = None;
        }
        if let Ok(mut second) = second.try_borrow_mut() {
            *second = None;
        }
    }
}

#[inline]
fn value_kind(kind: ValueKind) -> Item {
    Item {
        id: kind.to_string(),
        value: ViewString::Raw(kind.to_string()),
    }
}

#[inline]
fn cmp_kind(kind: ComparisonKind) -> Item {
    Item {
        id: kind.to_string(),
        value: ViewString::Raw(kind.to_string()),
    }
}

#[inline]
fn cmp_statement_head(kind: ComparisonKind) -> &'static str {
    match kind {
        ComparisonKind::NotOpenRange
        | ComparisonKind::NotCloseRange
        | ComparisonKind::NotLeftOpenRange
        | ComparisonKind::NotRightOpenRange => "!(",
        _ => "",
    }
}

#[inline]
fn cmp_statement_tail(kind: ComparisonKind) -> &'static str {
    match kind {
        ComparisonKind::NotOpenRange
        | ComparisonKind::NotCloseRange
        | ComparisonKind::NotLeftOpenRange
        | ComparisonKind::NotRightOpenRange => ")",
        _ => "",
    }
}

#[inline]
fn cmp_statement_symbol(kind: ComparisonKind) -> &'static str {
    match kind {
        ComparisonKind::Less => " x < ",
        ComparisonKind::Equal => " x = ",
        ComparisonKind::Greater => " x > ",
        ComparisonKind::LessOrEqual => " x ≤ ",
        ComparisonKind::GreaterOrEqual => " x ≥ ",
        ComparisonKind::Contain => " x Contains ",
        ComparisonKind::OpenRange | ComparisonKind::NotOpenRange => " < x < ",
        ComparisonKind::CloseRange | ComparisonKind::NotCloseRange => " ≤ x ≤ ",
        ComparisonKind::LeftOpenRange | ComparisonKind::NotLeftOpenRange => " < x ≤ ",
        ComparisonKind::RightOpenRange | ComparisonKind::NotRightOpenRange => " ≤ x < ",
        ComparisonKind::NotEqual => " != ",
        ComparisonKind::NotContain => " !Contains ",
    }
}
