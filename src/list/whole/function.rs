use super::{component::SortListKind, Message, Model, SortColumn, ViewInputStatus};
use crate::{
    list::{DataType, Kind},
    {CheckStatus, PagesInfo, SortStatus},
};
use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashSet;
use std::rc::Rc;
use yew::{Component, Context};

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub(super) fn toggle_view_input_status(&mut self, view: ViewInputStatus) {
        match self.view_input_status {
            ViewInputStatus::Add | ViewInputStatus::Edit => {
                self.view_input_status = ViewInputStatus::None;
            }
            ViewInputStatus::None => self.view_input_status = view,
        }
    }

    pub(super) fn item_range(&self, ctx: &Context<Self>) -> (usize, usize) {
        if let Ok(info) = ctx.props().pages_info.try_borrow() {
            let num = if ctx.props().kind == Kind::LayeredSecond {
                ctx.props().num_per_page_second
            } else {
                ctx.props().num_per_page
            };
            (
                (info.current - 1) * num + 1,
                std::cmp::min(info.current * num, self.sorted_keys.len()),
            )
        } else {
            (1, 1)
        }
    }

    pub(super) fn initiate_pages_info(&mut self, ctx: &Context<Self>) {
        if ctx.props().kind != Kind::LayeredSecond {
            if let Ok(mut info) = ctx.props().pages_info.try_borrow_mut() {
                let total = if ctx.props().data.is_empty() {
                    0 // HIGHLIGHT: end and total have 0 with no data
                } else {
                    (ctx.props().data.len() - 1) / ctx.props().num_per_page + 1
                };
                *info = PagesInfo {
                    current: 1,
                    start: 1,
                    end: std::cmp::min(ctx.props().num_pages, total),
                    total,
                };
                self.pages_info = Some(*info);
            }

            if ctx.props().kind == Kind::LayeredFirst {
                for (key, item) in ctx.props().data.iter() {
                    let total = if item.sub_items.is_empty() {
                        0
                    } else {
                        (item.sub_items.len() - 1) / ctx.props().num_per_page_second + 1
                    };
                    self.pages_info_second.insert(
                        key.clone(),
                        Rc::new(RefCell::new(PagesInfo {
                            current: 1,
                            start: 1,
                            end: std::cmp::min(ctx.props().num_pages, total),
                            total,
                        })),
                    );
                }
            }
        }
    }

    pub(super) fn update_pages_info(&mut self, ctx: &Context<Self>) {
        if let Ok(mut info) = ctx.props().pages_info.try_borrow_mut() {
            if ctx.props().data.is_empty() {
                *info = PagesInfo {
                    // HIGHLIGHT: this represents no data
                    current: 1,
                    start: 1,
                    end: 0,
                    total: 0,
                };
            } else {
                let total = (ctx.props().data.len() - 1) / ctx.props().num_per_page + 1;
                // Add the first item in both first and second layers
                if info.total == 0 && total == 1 {
                    *info = PagesInfo {
                        current: 1,
                        total: 1,
                        start: 1,
                        end: 1,
                    };
                } else {
                    let current =
                        std::cmp::min(self.pages_info.map_or(info.current, |p| p.current), total);
                    let start =
                        std::cmp::min(self.pages_info.map_or(info.start, |p| p.start), total);
                    let end = std::cmp::min(self.pages_info.map_or(info.end, |p| p.end), total);

                    *info = PagesInfo {
                        current,
                        total,
                        start,
                        end,
                    };
                }
            }
        }
    }

    pub(super) fn update_pages_info_second(&mut self, ctx: &Context<Self>) {
        if ctx.props().kind == Kind::LayeredFirst {
            // in case an item in first layer added
            for key in ctx.props().data.keys() {
                if let Vacant(entry) = self.pages_info_second.entry(key.clone()) {
                    entry.insert(Rc::new(RefCell::new(PagesInfo {
                        current: 1,
                        start: 1,
                        end: 0,
                        total: 0,
                    })));
                }
            }
            // in case an item in first layer deleted
            let data = Rc::clone(&ctx.props().data);
            self.pages_info_second
                .retain(|key, _| data.contains_key(key));
        }
    }

    pub(super) fn update_parent_check_status(&mut self, ctx: &Context<Self>) {
        if ctx.props().kind == Kind::LayeredSecond {
            if let Ok(mut status) = self.check_status_second.try_borrow_mut() {
                *status = self.check_status(ctx);
            }

            if let Some(parent) = ctx.link().get_parent() {
                parent
                    .clone()
                    .downcast::<Self>()
                    .send_message(Message::Render);
            }
        }
    }

    pub(super) fn check_all(&mut self, ctx: &Context<Self>, check: bool) {
        let (start, end) = self.item_range(ctx);
        for index in start..=end {
            if let Some(key) = self.sorted_keys.get(index - 1) {
                if self.checked.contains(key) {
                    if !check {
                        self.checked.remove(key);
                    }
                } else if check {
                    self.checked.insert(key.clone());
                }
            }
        }
    }

    pub(super) fn sort_keys(&mut self, ctx: &Context<Self>) -> (HashSet<String>, HashSet<String>) {
        // return: (added, deleted)
        let previous = self
            .sorted_keys
            .iter()
            .map(Clone::clone)
            .collect::<HashSet<String>>();

        let (index, asc) = self.sort.map_or((None, true), |s| {
            (Some(s.index), s.status == SortStatus::Ascending)
        });

        let mut keys: Vec<(String, String, Option<DateTime<Utc>>)> = ctx
            .props()
            .data
            .iter()
            .map(|(key, item)| {
                index.map_or((key.clone(), key.clone(), item.creation_time), |index| {
                    (
                        key.clone(),
                        item.columns
                            .get(index)
                            .map_or_else(String::new, ToString::to_string),
                        item.creation_time,
                    )
                })
            })
            .collect();

        // First step: the latest item first whether a sort column is designated or not
        keys.sort_unstable_by(|a, b| {
            if let (Some(a_time), Some(b_time)) = (a.2, b.2) {
                b_time.cmp(&a_time)
            } else {
                b.0.cmp(&a.0)
            }
        });
        // Second step: if a sort column is designated, sort items by the column
        if index.is_some() {
            if asc {
                keys.sort_by(|a, b| a.1.cmp(&b.1));
            } else {
                keys.sort_unstable_by(|a, b| b.1.cmp(&a.1));
            }
        }

        self.sorted_keys = keys.into_iter().map(|(k, _, _)| k).collect();
        let current = self
            .sorted_keys
            .iter()
            .map(Clone::clone)
            .collect::<HashSet<String>>();

        let added = current
            .iter()
            .filter_map(|k| previous.get(k).map_or(Some(k.clone()), |_| None))
            .collect::<HashSet<String>>();

        let deleted = previous
            .iter()
            .filter_map(|p| current.get(p).map_or(Some(p.clone()), |_| None))
            .collect::<HashSet<String>>();

        (added, deleted)
    }

    pub(super) fn update_checked(&mut self, ctx: &Context<Self>, added: &HashSet<String>) {
        let prev = self.checked.clone();
        self.checked.clear();
        let (start, end) = self.item_range(ctx);
        for index in start..=end {
            if let Some(key) = self.sorted_keys.get(index - 1) {
                if prev.contains(key) && !added.contains(key) {
                    self.checked.insert(key.clone());
                }
            }
        }
    }

    pub(super) fn reset_sort_second_layer(&mut self, ctx: &Context<Self>) {
        if ctx.props().kind == Kind::LayeredFirst {
            self.sort_second_layer = match ctx.props().data_type {
                DataType::Customer => Some(SortColumn {
                    index: 0,
                    status: SortStatus::Ascending,
                }),
                DataType::Network
                | DataType::Account
                | DataType::Node
                | DataType::TrustedDomain
                | DataType::TiDb => None,
            };
        }
    }

    pub(super) fn check_status(&self, ctx: &Context<Self>) -> CheckStatus {
        let (start, end) = self.item_range(ctx);
        let len = (start..=end)
            .filter_map(|index| {
                self.sorted_keys.get(index - 1).and_then(|key| {
                    if self.checked.contains(key) {
                        Some(true)
                    } else {
                        None
                    }
                })
            })
            .count();

        if len == 0 {
            CheckStatus::Unchecked
        } else if len == (end - start + 1) {
            CheckStatus::Checked
        } else {
            CheckStatus::Indeterminate
        }
    }

    pub(super) fn clear_input_data(ctx: &Context<Self>) {
        match ctx.props().kind {
            Kind::LayeredFirst | Kind::Flat => {
                for input in &ctx.props().input_data {
                    if let Ok(mut input) = input.try_borrow_mut() {
                        input.clear();
                    }
                }
                if let Ok(mut input_ids) = ctx.props().input_ids.try_borrow_mut() {
                    input_ids.clear();
                }
            }
            Kind::LayeredSecond => {
                if let Some(input) = ctx.props().input_second_data.as_ref() {
                    for input in input.iter() {
                        if let Ok(mut input) = input.try_borrow_mut() {
                            input.clear();
                        }
                    }
                }
            }
        }
    }

    pub(super) fn set_sort_list_kind(&mut self, ctx: &Context<Self>) {
        if ctx.props().kind != Kind::LayeredSecond {
            if let Ok(mut kind) = self.sort_list_kind.try_borrow_mut() {
                if let Some(sort) = self.sort {
                    if sort.index == 0 {
                        match sort.status {
                            SortStatus::Ascending => *kind = Some(SortListKind::Ascending),
                            SortStatus::Descending => *kind = Some(SortListKind::Descending),
                            SortStatus::Unsorted => {
                                *kind = None; // unreachable
                            }
                        }
                    } else {
                        *kind = None;
                    }
                } else {
                    *kind = Some(SortListKind::LatestFirst);
                }
            }
        }
    }
}
