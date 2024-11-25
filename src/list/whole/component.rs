use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use json_gettext::get_text;
use yew::{html, virtual_dom::AttrValue, Component, Context, Html, Properties};

use super::{MessageType, DEFAULT_NUM_PAGES, DEFAULT_NUM_PER_PAGE};
use crate::{
    input::InputSecondId,
    language::Language,
    list::{DataType, DisplayInfo, Kind, ListItem},
    text, CheckStatus, Input, InputConfig, InputItem, InputTag, MoreAction, PagesInfo, SelectMini,
    SelectMiniKind, SortStatus, Texts, ViewString,
};

const DEFAULT_TABLE_WIDTH: u32 = 100;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SortColumn {
    pub index: usize,
    pub status: SortStatus,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum ViewInputStatus {
    Add,
    Edit,
    None,
}

#[derive(Clone, PartialEq)]
pub struct Model<T> {
    data_cache: HashMap<String, ListItem>, // to check `data` being changed
    id_cache: String,                      // to check `id` being changed
    // `pages_info` is not just a cache. This is altered by this `Model`.
    pub(super) pages_info: Option<PagesInfo>, // to check `pages_info` being changed
    // `check_status_second` is not just a cache. This is altered by this `Model`.
    pub(super) check_status_second: Rc<RefCell<CheckStatus>>,

    pub(super) sort: Option<SortColumn>,
    pub(super) sort_second_layer: Option<SortColumn>,
    pub(super) sorted_keys: Vec<String>,

    pub(super) pages_info_second: HashMap<String, Rc<RefCell<PagesInfo>>>,
    pub(super) expand_list: HashSet<String>,
    pub(super) checked: HashSet<String>,

    pub(super) view_input_status: ViewInputStatus,
    pub(super) more_action: Rc<RefCell<Option<MoreAction>>>,
    pub(super) sort_list_kind: Rc<RefCell<Option<SortListKind>>>,

    phantom: PhantomData<T>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    ClickExpandible(String),
    ClickSort(usize),
    CheckItem(String),
    CheckAll,
    CheckAllSecond,
    Render,
    MovePage,
    ResetCheckSecond,

    InputEscape, // click the escape key in the add window
    InputAdd,    // click the add button
    Add,
    Edit, // key
    Delete(String),
    AddSecond,
    EditSecond,
    DeleteSecond,

    ExtraMessage(MessageType),

    ClearChecked,
    ClearOtherLayerChecked,
    DeleteChecked,
    CancelChecked,

    DoMoreAction(String),
    SortList,
    SetSecondSortDefault,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortListKind {
    LatestFirst,
    Ascending,
    Descending,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,

    // id:: 1st layer: customer/network, 2nd layer: customer-{item's id}
    pub id: AttrValue,
    pub title: &'static str,
    #[prop_or(None)]
    pub title_second: Option<&'static str>,
    pub kind: Kind,
    #[prop_or(None)]
    pub data_type: Option<DataType>, // None means default
    pub data: Rc<HashMap<String, ListItem>>, // (key of a row, item of the row)
    #[prop_or(None)]
    pub sort: Option<SortColumn>,
    pub display_info: Rc<DisplayInfo>,
    #[prop_or(DEFAULT_NUM_PER_PAGE)]
    pub num_per_page: usize,
    #[prop_or(DEFAULT_NUM_PER_PAGE)]
    pub num_per_page_second: usize,
    #[prop_or(DEFAULT_NUM_PAGES)]
    pub num_pages: usize,
    #[prop_or(Rc::new(RefCell::new(PagesInfo::default())))]
    pub pages_info: Rc<RefCell<PagesInfo>>,
    #[prop_or(Rc::new(RefCell::new(CheckStatus::Unchecked)))]
    pub check_status_second: Rc<RefCell<CheckStatus>>, // check status of the second layer
    #[prop_or(None)]
    pub check_status_second_cache: Option<CheckStatus>,

    // for add/edit/delete
    pub input_ids: Rc<RefCell<Vec<String>>>, // Vec for deleting multiple items
    #[prop_or(Rc::new(RefCell::new(None)))]
    pub input_second_keys: Rc<RefCell<Option<Vec<String>>>>, // String: key

    pub input_data: Vec<Rc<RefCell<InputItem>>>,
    #[prop_or(None)]
    pub input_data_tag: Option<Rc<RefCell<InputTag>>>,
    pub input_add_title: &'static str,
    pub input_edit_title: &'static str,
    pub input_width: u32,
    pub input_height: u32,
    pub input_conf: Vec<Rc<InputConfig>>,

    #[prop_or(None)]
    pub input_second_data: Option<Vec<Rc<RefCell<InputItem>>>>,
    #[prop_or(None)]
    pub input_second_add_title: Option<&'static str>,
    #[prop_or(None)]
    pub input_second_edit_title: Option<&'static str>,

    #[prop_or(None)]
    pub input_second_width: Option<u32>,
    #[prop_or(None)]
    pub input_second_height: Option<u32>,
    #[prop_or(None)]
    pub input_second_type: Option<Vec<Rc<InputConfig>>>,

    #[prop_or(None)]
    pub br_separator: Option<&'static str>,

    pub messages: HashMap<MessageType, T::Message>,
}

impl<T> Component for Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let mut s = Self {
            data_cache: (*ctx.props().data).clone(),
            id_cache: ctx.props().id.as_ref().into(),
            pages_info: ctx.props().pages_info.try_borrow().ok().map(|info| *info),
            check_status_second: ctx.props().check_status_second.clone(),

            sort: ctx.props().sort,
            sort_second_layer: None,
            sorted_keys: Vec::new(),

            pages_info_second: HashMap::new(),
            expand_list: HashSet::new(),
            checked: HashSet::new(),

            view_input_status: ViewInputStatus::None,
            more_action: Rc::new(RefCell::new(None)),
            sort_list_kind: Rc::new(RefCell::new(None)),

            phantom: PhantomData,
        };
        s.initiate_pages_info(ctx);
        s.reset_sort_second_layer(ctx);
        s.set_sort_list_kind(ctx);
        s.sort_keys(ctx);
        s
    }

    #[allow(clippy::too_many_lines)]
    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        // even if the page or sort changes in Flat or LayeredFirst, this `change` is not called. Instead `update` is called.
        let data_changed = self.data_cache != *ctx.props().data;
        let id_changed = self.id_cache != ctx.props().id.as_ref();

        let sort_changed = self.sort != ctx.props().sort;
        if self.data_cache.len() < ctx.props().data.len() {
            // if an item is added, sort items by latest to display the added one at the top.
            self.initiate_pages_info(ctx); // go to the first page
            self.sort = None;
            self.set_sort_list_kind(ctx);
        }

        match ctx.props().kind {
            Kind::Flat => {
                // id_changed: in the case when the menu changes like customer -> network
                if id_changed {
                    self.initiate_pages_info(ctx);
                    self.sort_keys(ctx);
                } else if data_changed {
                    // when add/edit/delete a item
                    let (added, deleted) = self.sort_keys(ctx);
                    self.update_pages_info(ctx);

                    // modify self.checked according to the result of add or delete
                    if !added.is_empty() || !deleted.is_empty() {
                        self.update_checked(ctx, &added);
                    }
                } else if let (Ok(mut pages_info), Some(pre_pages_info)) =
                    (ctx.props().pages_info.try_borrow_mut(), self.pages_info)
                {
                    *pages_info = pre_pages_info;
                }
            }
            Kind::LayeredFirst => {
                if id_changed {
                    self.initiate_pages_info(ctx);
                    self.sort_keys(ctx);
                    self.reset_sort_second_layer(ctx);
                } else if data_changed {
                    let (added, deleted) = self.sort_keys(ctx);
                    self.update_pages_info(ctx);
                    self.update_pages_info_second(ctx);

                    if !added.is_empty() || !deleted.is_empty() {
                        self.update_checked(ctx, &added);
                    }
                }
                self.set_first_layer_input_id(ctx);
            }
            Kind::LayeredSecond => {
                // In Flat or LayeredFirst, `pages_info` is not changed but updated by a user's click.
                // In LayerdSecond, `pages_info` can be changed when its parent page changes as well as updated.
                let page_changed = if let (Some(prev), Ok(info)) =
                    (self.pages_info, ctx.props().pages_info.try_borrow())
                {
                    prev != *info
                } else {
                    false
                };

                if id_changed {
                    self.sort = ctx.props().sort;
                    self.pages_info = ctx.props().pages_info.try_borrow().ok().map(|info| *info);
                    self.sort_keys(ctx);
                    self.checked.clear();
                } else if data_changed {
                    self.sort = Some(SortColumn {
                        index: 0,
                        status: SortStatus::Ascending,
                    });
                    if let Some(parent) = ctx.link().get_parent() {
                        parent
                            .clone()
                            .downcast::<Self>()
                            .send_message(Message::SetSecondSortDefault);
                    }

                    let (added, deleted) = self.sort_keys(ctx);
                    self.update_pages_info(ctx);
                    self.pages_info = ctx.props().pages_info.try_borrow().ok().map(|info| *info);

                    if added.is_empty() && deleted.is_empty() {
                        self.check_status_second = ctx.props().check_status_second.clone();
                        if let Ok(status) = self.check_status_second.try_borrow() {
                            if *status == CheckStatus::Checked {
                                let (start, end) = self.item_range(ctx);
                                for index in start..=end {
                                    if let Some(key) = self.sorted_keys.get(index - 1) {
                                        if !self.checked.contains(key) {
                                            self.checked.insert(key.clone());
                                        }
                                    }
                                }
                            } else if *status == CheckStatus::Unchecked {
                                self.checked.clear();
                            }
                        }
                    } else {
                        self.update_checked(ctx, &added);
                        self.update_parent_check_status(ctx);
                    }
                } else if page_changed {
                    self.pages_info = ctx.props().pages_info.try_borrow().ok().map(|info| *info);
                    self.checked.clear();
                } else if sort_changed {
                    // HIGHLIGHT: Clicking sort buttons calls `update` NOT `change` in Flat and LayeredFirst, which doesn't need to update `self.sort` in both.
                    // On the other hand, self.sort should be updated since `LayeredFirst` calls `LayeredSecond`'s `change`.
                    self.sort = ctx.props().sort;
                    self.pages_info = ctx.props().pages_info.try_borrow().ok().map(|info| *info);
                    self.sort_keys(ctx);
                    self.checked.clear();
                }

                if let Some(CheckStatus::Checked) = ctx.props().check_status_second_cache {
                    if let Ok(info) = ctx.props().pages_info.try_borrow() {
                        if info.total == 0 {
                            self.update_parent_check_status(ctx);
                        } else {
                            self.check_all(ctx, true);
                        }
                    }
                } else if let Some(CheckStatus::Unchecked) = ctx.props().check_status_second_cache {
                    self.check_all(ctx, false);
                }
            }
        }

        if id_changed {
            self.id_cache = ctx.props().id.as_ref().into();
        }
        if data_changed {
            self.data_cache.clone_from(&(*ctx.props().data));
        }

        true
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::ClickExpandible(key) => {
                if self.expand_list.contains(&key) {
                    self.expand_list.remove(&key);
                    if let Ok(mut id) = ctx.props().input_ids.try_borrow_mut() {
                        *id = Vec::new();
                    }
                } else {
                    let (start, end) = self.item_range(ctx);
                    for index in start..=end {
                        if let Some(key) = self.sorted_keys.get(index - 1) {
                            self.expand_list.remove(key);
                        }
                    }
                    self.expand_list.insert(key.clone());
                    if let Ok(mut id) = ctx.props().input_ids.try_borrow_mut() {
                        *id = vec![key];
                    }
                };
                self.checked.clear();
                if let Ok(mut second) = self.check_status_second.try_borrow_mut() {
                    *second = CheckStatus::Unchecked;
                }
            }
            Message::SortList => {
                let sort = if let Ok(sort_input) = self.sort_list_kind.try_borrow() {
                    match *sort_input {
                        Some(SortListKind::LatestFirst) => {
                            self.sort = None;
                            true
                        }
                        Some(SortListKind::Ascending) => {
                            self.sort = Some(SortColumn {
                                index: 0,
                                status: SortStatus::Ascending,
                            });
                            true
                        }
                        Some(SortListKind::Descending) => {
                            self.sort = Some(SortColumn {
                                index: 0,
                                status: SortStatus::Descending,
                            });
                            true
                        }
                        None => false,
                    }
                } else {
                    false
                };
                if sort {
                    self.checked.clear();
                    self.sort_keys(ctx);
                    if ctx.props().kind == Kind::LayeredFirst {
                        self.set_first_layer_input_id(ctx);
                    }
                }
            }
            Message::ClickSort(index) => {
                self.checked.clear();

                let current_sort = match ctx.props().kind {
                    Kind::Flat => self.sort,
                    Kind::LayeredFirst => self.sort_second_layer,
                    Kind::LayeredSecond => return false, // unreachable
                };
                let next_sort = current_sort.map_or(
                    Some(SortColumn {
                        index,
                        status: SortStatus::Ascending,
                    }),
                    |sort| {
                        let status = if index == sort.index {
                            match sort.status {
                                SortStatus::Ascending => SortStatus::Descending,
                                SortStatus::Descending | SortStatus::Unsorted => {
                                    SortStatus::Ascending
                                }
                            }
                        } else {
                            SortStatus::Ascending
                        };
                        Some(SortColumn { index, status })
                    },
                );
                match ctx.props().kind {
                    Kind::Flat => {
                        self.sort = next_sort;
                        self.set_sort_list_kind(ctx);
                        self.sort_keys(ctx);
                    }
                    Kind::LayeredFirst => {
                        self.sort_second_layer = next_sort;
                        ctx.link().send_message(Message::ResetCheckSecond);
                        return false;
                    }
                    Kind::LayeredSecond => return false, // unreachable
                };
            }
            Message::CheckItem(key) => {
                if self.check_status(ctx) == CheckStatus::Unchecked {
                    ctx.link().send_message(Message::ClearOtherLayerChecked);
                }
                if self.checked.contains(&key) {
                    self.checked.remove(&key);
                } else {
                    self.checked.insert(key);
                }
                if ctx.props().kind == Kind::LayeredSecond {
                    self.update_parent_check_status(ctx);
                    // HIGHLIGHT: should return true
                }
            }
            Message::CheckAll => {
                // This can occur except in LayeredSecond
                let check_status = self.check_status(ctx);
                let clear = match check_status {
                    CheckStatus::Unchecked => {
                        self.check_all(ctx, true);
                        true
                    }
                    CheckStatus::Indeterminate => {
                        self.check_all(ctx, true);
                        false
                    }
                    CheckStatus::Checked => {
                        self.check_all(ctx, false);
                        false
                    }
                };
                if clear {
                    ctx.link().send_message(Message::ClearOtherLayerChecked);
                }
            }
            Message::CheckAllSecond => {
                // This can occur in LayeredFirst only
                let (start, end) = self.item_range(ctx);
                if (start..=end)
                    .filter_map(|index| {
                        self.sorted_keys.get(index - 1).and_then(|key| {
                            if self.expand_list.contains(key) {
                                Some(true)
                            } else {
                                None
                            }
                        })
                    })
                    .count()
                    == 0
                {
                    return false;
                }

                let clear = if let Ok(mut status) = self.check_status_second.try_borrow_mut() {
                    match *status {
                        CheckStatus::Unchecked => {
                            *status = CheckStatus::Checked;
                            true
                        }
                        CheckStatus::Indeterminate => {
                            *status = CheckStatus::Checked;
                            false
                        }
                        CheckStatus::Checked => {
                            *status = CheckStatus::Unchecked;
                            false
                        }
                    }
                } else {
                    false
                };

                if clear {
                    ctx.link().send_message(Message::ClearChecked);
                }
            }
            Message::Render => (),
            Message::MovePage => {
                self.pages_info = ctx.props().pages_info.try_borrow().ok().map(|info| *info);
                ctx.link().send_message(Message::ClearChecked);
                if ctx.props().kind == Kind::LayeredFirst {
                    self.set_first_layer_input_id(ctx);
                    ctx.link().send_message(Message::ResetCheckSecond);
                }
                return false;
            }
            Message::ClearOtherLayerChecked => match ctx.props().kind {
                Kind::LayeredFirst => {
                    if let Ok(mut second) = self.check_status_second.try_borrow_mut() {
                        *second = CheckStatus::Unchecked;
                    }
                }
                Kind::LayeredSecond => {
                    if let Some(parent) = ctx.link().get_parent() {
                        parent
                            .clone()
                            .downcast::<Self>()
                            .send_message(Message::ClearChecked);
                    }
                }
                Kind::Flat => (),
            },
            Message::ClearChecked => {
                self.checked.clear();
                if ctx.props().kind == Kind::LayeredSecond {
                    if let Some(parent) = ctx.link().get_parent() {
                        parent
                            .clone()
                            .downcast::<Self>()
                            .send_message(Message::ResetCheckSecond);
                    }
                }
            }
            Message::ResetCheckSecond => {
                if let Ok(mut status) = self.check_status_second.try_borrow_mut() {
                    *status = CheckStatus::Unchecked;
                }
            }
            Message::InputEscape => {
                self.view_input_status = ViewInputStatus::None;
            }
            Message::InputAdd => {
                self.toggle_view_input_status(ViewInputStatus::Add);
                Self::clear_input_data(ctx);
            }
            Message::ExtraMessage(message) => {
                if let (Some(parent), Some(msg)) =
                    (ctx.link().get_parent(), ctx.props().messages.get(&message))
                {
                    parent.clone().downcast::<T>().send_message(msg.clone());
                }
            }
            Message::Add => {
                self.toggle_view_input_status(ViewInputStatus::Add);
                if let (Some(parent), Some(msg)) = (
                    ctx.link().get_parent(),
                    ctx.props().messages.get(&MessageType::Add),
                ) {
                    parent.clone().downcast::<T>().send_message(msg.clone());
                }
                return false;
            }
            Message::Edit => {
                self.toggle_view_input_status(ViewInputStatus::Edit);
                if let (Some(parent), Some(msg)) = (
                    ctx.link().get_parent(),
                    ctx.props().messages.get(&MessageType::Edit),
                ) {
                    parent.clone().downcast::<T>().send_message(msg.clone());
                }
                // HIGHLIGHT: If a tag is modified, input_data is modified at the same time.
                // After that, users click "Save". Therefore input_data doesn't seem changed
                // from the point of WholeList, which doesn't rerender itself as a result.
                // In order to avoid this, rerender whenever Edit happens.
            }
            Message::Delete(key) => {
                if let (Some(parent), Some(msg), Ok(mut ids)) = (
                    ctx.link().get_parent(),
                    ctx.props().messages.get(&MessageType::Delete),
                    ctx.props().input_ids.try_borrow_mut(),
                ) {
                    *ids = vec![key];
                    parent.clone().downcast::<T>().send_message(msg.clone());
                }
                return false;
            }
            Message::AddSecond => {
                self.toggle_view_input_status(ViewInputStatus::Add);
                if let Ok(mut second) = ctx.props().input_second_keys.try_borrow_mut() {
                    *second = None;
                }
                if let (Some(parent), Some(msg)) = (
                    ctx.link().get_parent(),
                    ctx.props().messages.get(&MessageType::AddSecond),
                ) {
                    if let Some(parent) = parent.clone().downcast::<Self>().get_parent() {
                        parent.clone().downcast::<T>().send_message(msg.clone());
                    }
                }
                return false;
            }
            Message::EditSecond => {
                self.toggle_view_input_status(ViewInputStatus::Edit);
                if let (Some(parent), Some(msg)) = (
                    ctx.link().get_parent(),
                    ctx.props().messages.get(&MessageType::EditSecond),
                ) {
                    if let Some(parent) = parent.clone().downcast::<Self>().get_parent() {
                        parent.clone().downcast::<T>().send_message(msg.clone());
                    }
                }
                return false;
            }
            Message::DeleteSecond => {
                if let (Some(parent), Some(msg)) = (
                    ctx.link().get_parent(),
                    ctx.props().messages.get(&MessageType::DeleteSecond),
                ) {
                    if let Some(parent) = parent.clone().downcast::<Self>().get_parent() {
                        parent.clone().downcast::<T>().send_message(msg.clone());
                    }
                }
                return false;
            }
            Message::DoMoreAction(key) => {
                if let Ok(mut action) = self.more_action.try_borrow_mut() {
                    match *action {
                        Some(MoreAction::Delete) => {
                            match ctx.props().kind {
                                Kind::LayeredSecond => {
                                    if let Ok(mut second) =
                                        ctx.props().input_second_keys.try_borrow_mut()
                                    {
                                        *second = Some(vec![key]);
                                    }
                                    ctx.link().send_message(Message::DeleteSecond);
                                }
                                Kind::LayeredFirst | Kind::Flat => {
                                    if let Ok(mut second) =
                                        ctx.props().input_second_keys.try_borrow_mut()
                                    {
                                        *second = None;
                                    }
                                    ctx.link().send_message(Message::Delete(key));
                                }
                            }
                            *action = None;
                            return false;
                        }
                        Some(MoreAction::Edit) => {
                            if let Some(current) = ctx.props().data.get(&key) {
                                match ctx.props().kind {
                                    Kind::LayeredSecond => {
                                        if let Some(input) = ctx.props().input_second_data.as_ref()
                                        {
                                            for (index, item) in input.iter().enumerate() {
                                                if let (Some(list), Ok(mut item)) = (
                                                    current.columns.get(index),
                                                    item.try_borrow_mut(),
                                                ) {
                                                    *item = list.into();
                                                }
                                            }
                                        }
                                    }
                                    Kind::LayeredFirst | Kind::Flat => {
                                        if let Ok(mut id) = ctx.props().input_ids.try_borrow_mut() {
                                            let mut index: usize = 0;
                                            for item in &ctx.props().input_data {
                                                if let (Some(list), Ok(mut item)) = (
                                                    current.columns.get(index),
                                                    item.try_borrow_mut(),
                                                ) {
                                                    // HIGHLIGHT: Currently, the only exception is InputItem::Password. But so might be some more.
                                                    if let InputItem::Password(_) = *item {
                                                        continue;
                                                    }
                                                    *item = list.into();
                                                    index += 1;
                                                }
                                            }
                                            *id = vec![key.clone()];
                                        }
                                    }
                                }
                            }
                            if let Ok(mut second) = ctx.props().input_second_keys.try_borrow_mut() {
                                *second = Some(vec![key]);
                            }
                            *action = None;
                            self.view_input_status = ViewInputStatus::Edit;
                        }
                        _ => (),
                    }
                }
            }
            Message::CancelChecked => {
                ctx.link().send_message(Message::ClearChecked);
                return false;
            }
            Message::DeleteChecked => match ctx.props().kind {
                Kind::LayeredSecond => {
                    if let Ok(mut second) = ctx.props().input_second_keys.try_borrow_mut() {
                        let keys = self.checked.iter().cloned().collect::<Vec<String>>();
                        *second = Some(keys);
                    }
                    ctx.link().send_message(Message::DeleteSecond);
                    return false;
                }
                Kind::LayeredFirst | Kind::Flat => {
                    let send_msg = if let Ok(mut ids) = ctx.props().input_ids.try_borrow_mut() {
                        *ids = self.checked.iter().cloned().collect::<Vec<String>>();
                        true
                    } else {
                        false
                    };
                    if send_msg {
                        if let (Some(parent), Some(msg)) = (
                            ctx.link().get_parent(),
                            ctx.props().messages.get(&MessageType::Delete),
                        ) {
                            parent.clone().downcast::<T>().send_message(msg.clone());
                        }
                    }
                    return false;
                }
            },
            Message::SetSecondSortDefault => {
                self.reset_sort_second_layer(ctx);
            }
        }
        true
    }

    #[allow(clippy::too_many_lines)]
    fn view(&self, ctx: &Context<Self>) -> Html {
        let style_full = if cfg!(feature = "pumpkin-dark") {
            String::new()
        } else {
            format!("width: {}px;", ctx.props().display_info.width_full)
        };
        let style_view = if cfg!(feature = "pumpkin-dark") {
            format!("width: {DEFAULT_TABLE_WIDTH}%;")
        } else {
            format!("width: {}px;", ctx.props().display_info.width_view)
        };

        let txt = ctx.props().txt.txt.clone();
        let onclick_add = ctx.link().callback(|_| Message::InputAdd);
        let input_id = ctx
            .props()
            .input_ids
            .try_borrow()
            .map_or(None, |ids| ids.first().cloned());
        let sort_column = ctx
            .props()
            .display_info
            .titles
            .first()
            .map_or_else(String::new, |t| {
                get_text!(txt, ctx.props().language.tag(), t)
                    .map_or_else(String::new, |t| t.to_string())
            });
        let sort_list_kind_list = Rc::new(vec![
            ViewString::Key("Latest".to_string()),
            ViewString::Raw(format!(
                "{} ({})",
                get_text!(txt, ctx.props().language.tag(), "A ➝ Z")
                    .map_or("A ➝ Z".to_string(), |t| t.to_string()),
                &sort_column
            )),
            ViewString::Raw(format!(
                "{} ({})",
                get_text!(txt, ctx.props().language.tag(), "Z ➝ A")
                    .map_or("Z ➝ A".to_string(), |t| t.to_string()),
                &sort_column
            )),
        ]);
        let value_candidates = Rc::new(vec![
            SortListKind::LatestFirst,
            SortListKind::Ascending,
            SortListKind::Descending,
        ]);
        let list_top = if cfg!(feature = "pumpkin-dark") {
            42
        } else {
            38
        };

        match ctx.props().kind {
            Kind::LayeredFirst | Kind::Flat => {
                html! {
                    <>
                        <div class="list-title">
                            { text!(txt, ctx.props().language, &ctx.props().title) }
                        </div>
                        <div class="list-add" onclick={onclick_add}>
                            if cfg!(feature = "pumpkin-dark") {
                                <img src="/frontary/clumit-list-add.svg" class="list-add" />
                            } else {
                                <img src="/frontary/list-add.png" class="list-add" />
                            }
                            { text!(txt, ctx.props().language, "Add") }
                        </div>
                        <div class="list-sort-recently">
                            <SelectMini::<SortListKind, Self>
                                txt={ctx.props().txt.clone()}
                                language={ctx.props().language}
                                parent_message={Message::SortList}
                                id={"sort-list".to_string()}
                                active={true}
                                list={Rc::clone(&sort_list_kind_list)}
                                candidate_values={Rc::clone(&value_candidates)}
                                selected_value={Rc::clone(&self.sort_list_kind)}
                                selected_value_cache={self.sort_list_kind.try_borrow().ok().and_then(|k| *k)}
                                align_left={false}
                                {list_top}
                                kind={SelectMiniKind::SortList}
                            />
                        </div>
                        <div class="list-table" style={style_view}>
                            <table class="list-table" style={style_full}>
                                { self.view_head(ctx) }
                                { self.view_list(ctx) }
                                { self.view_pages(ctx, true) }
                            </table>
                        </div>

                        {
                            if self.view_input_status == ViewInputStatus::None {
                                html! {}
                            } else {
                                let (msg, title) = match self.view_input_status {
                                    ViewInputStatus::Add => (Message::Add, ctx.props().input_add_title),
                                    ViewInputStatus::Edit => (Message::Edit, ctx.props().input_edit_title),
                                    ViewInputStatus::None => unreachable!(),
                                };
                                let messages = if ctx.props().data_type == Some(DataType::Network) {
                                    let mut messages: HashMap<MessageType, Message> = HashMap::new();
                                    messages.insert(MessageType::AddTag, Message::ExtraMessage(MessageType::AddTag));
                                    messages.insert(MessageType::EditTag, Message::ExtraMessage(MessageType::EditTag));
                                    messages.insert(MessageType::DeleteTag, Message::ExtraMessage(MessageType::DeleteTag));
                                    Some(messages)
                                } else {
                                    None
                                };
                                let tag = ctx.props().input_data_tag.clone();
                                html! {
                                    <Input<Self>
                                        txt={ctx.props().txt.clone()}
                                        language={ctx.props().language}
                                        data={Rc::clone(&ctx.props().data)}
                                        title={title}
                                        width={ctx.props().input_width}
                                        height={ctx.props().input_height}
                                        input_conf={ctx.props().input_conf.clone()}
                                        input_id={input_id}
                                        input_data={ctx.props().input_data.clone()}
                                        input_data_tag={tag}
                                        action_message={msg}
                                        escape_message={Message::InputEscape}
                                        extra_messages={messages}
                                    />
                                }
                            }
                        }
                    </>
                }
            }
            Kind::LayeredSecond => html! {
                <>
                    { self.view_list(ctx) }
                    { self.view_pages(ctx, false) }
                    {
                        if self.view_input_status == ViewInputStatus::None {
                            html! {}
                        } else {
                            let second_id = if let Ok(seconds) = ctx.props().input_second_keys.try_borrow() {
                                (*seconds).clone().and_then(|s| s.first().cloned())
                            } else {
                                None
                            };
                            let (second_id, msg, title) = match self.view_input_status {
                                ViewInputStatus::Add => (Some(InputSecondId::Add), Message::AddSecond, ctx.props().input_add_title),
                                ViewInputStatus::Edit => (
                                    second_id.map(InputSecondId::Edit),
                                    Message::EditSecond,
                                    ctx.props().input_edit_title
                                ),
                                ViewInputStatus::None => unreachable!(),
                            };
                            if let Some(data) = ctx.props().input_second_data.as_ref() {
                                let tag = ctx.props().input_data_tag.clone();
                                html! {
                                    <Input<Self>
                                        txt={ctx.props().txt.clone()}
                                        language={ctx.props().language}
                                        data={Rc::clone(&ctx.props().data)}
                                        title={title}
                                        width={ctx.props().input_width}
                                        height={ctx.props().input_height}
                                        input_conf={ctx.props().input_conf.clone()}
                                        input_id={input_id}
                                        input_second_id={second_id}

                                        input_data={data.clone()}
                                        input_data_tag={tag}
                                        action_message={msg}
                                        escape_message={Message::InputEscape}
                                    />
                                }
                            } else {
                                html! {}
                            }
                        }
                    }
                </>
            },
        }
    }
}
