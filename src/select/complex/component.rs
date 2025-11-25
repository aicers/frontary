use std::cell::RefCell;
use std::collections::{
    HashMap, HashSet,
    hash_map::Entry::{Occupied, Vacant},
};
use std::rc::Rc;

use json_gettext::get_text;
use yew::virtual_dom::AttrValue;
use yew::{Component, Context, Html, Properties, classes, html};

use super::DEFAULT_POP_WIDTH;
use crate::click_outside::toggle_visibility_complex;
use crate::{
    CheckStatus, ComplexSelection, EndpointKind, NetworkGroup, NetworkItem, SelectionExtraInfo,
    Texts, Theme, language::Language, text, validate_host_network,
};

#[cfg(feature = "pumpkin")]
const DEFAULT_FONT: &str = "";
#[cfg(not(feature = "pumpkin"))]
const DEFAULT_FONT: &str = "13px 'Spoqa Han Sans Neo'";

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Kind {
    Basic,
    NetworkIp,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ItemKind {
    Registered,
    Custom,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Directions {
    pub(super) registered: Rc<RefCell<Option<EndpointKind>>>,
    pub(super) custom: Rc<RefCell<Option<EndpointKind>>>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, PartialEq, Eq)]
pub struct Model {
    pub(super) search_result: Option<Vec<usize>>,
    pub(super) search_text: String,
    pub(super) input_text: String,
    pub(super) input_wrong_msg: Option<&'static str>,
    pub(super) directions: Directions,
    pub(super) direction_items: HashMap<String, Rc<RefCell<Option<SelectionExtraInfo>>>>,

    pub(super) view_list: bool,
    pub(super) view_input: bool,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Message {
    Click,
    Close,
    InputSearch(String),
    InputInput(String),
    ToggleList,
    ToggleInput,
    ClickItem(String, ItemKind),
    ClickAll,
    ClickAllBelow(ItemKind),
    ClickAddInput,
    DeleteInputItem(String),
    Render,
    SetDirection,
    SetDirectionItem(ItemKind),
    InputError,
}

#[derive(Clone, PartialEq, Eq, Properties)]
pub struct Props {
    pub txt: Texts,
    pub language: Language,
    #[prop_or(Kind::Basic)]
    pub kind: Kind,
    pub id: AttrValue,
    pub title: AttrValue,
    pub empty_msg: AttrValue,
    pub top_width: u32,
    #[prop_or(DEFAULT_POP_WIDTH)]
    pub pop_width: u32,
    #[prop_or(DEFAULT_FONT.into())]
    pub font: AttrValue,
    pub list: Rc<RefCell<Vec<NetworkItem>>>,
    pub selected: Rc<ComplexSelection>,
    #[prop_or(true)]
    pub allow_input: bool,
    #[prop_or(false)]
    pub allow_empty: bool,
    #[prop_or(true)]
    pub default_all: bool,
    #[prop_or(None)]
    pub theme: Option<Theme>,
}

impl Component for Model {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut s = Self {
            search_result: None,
            search_text: String::new(),
            input_text: String::new(),
            input_wrong_msg: None,
            view_list: false,
            view_input: false,
            directions: Directions {
                registered: Rc::new(RefCell::new(None)),
                custom: Rc::new(RefCell::new(None)),
            },
            direction_items: HashMap::new(),
        };
        s.buffer_direction_items(ctx);

        s
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if let (Ok(mut sel), Ok(list)) = (
            ctx.props().selected.predefined.try_borrow_mut(),
            ctx.props().list.try_borrow(),
        ) && let Some(predefined) = sel.as_mut()
        {
            let list_tmp = list
                .iter()
                .map(NetworkItem::id)
                .collect::<HashSet<&String>>();
            predefined.retain(|k, _| list_tmp.contains(k));
            if !cfg!(feature = "pumpkin") && self.check_status(ctx, false) == CheckStatus::Checked {
                *sel = None;
            }
        }
        self.buffer_direction_items(ctx);

        true
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Click | Message::Close => {
                let _ = toggle_visibility_complex(&ctx.props().id);
            }
            Message::ToggleList => {
                self.view_list = !self.view_list;
                if self.view_list {
                    self.view_input = false;
                }
            }
            Message::ToggleInput => {
                self.view_input = !self.view_input;
                if self.view_input {
                    self.view_list = false;
                }
            }
            Message::InputSearch(text) => {
                self.search_text.clone_from(&text);
                if text.is_empty() {
                    self.search_result = None;
                } else if let Ok(list) = ctx.props().list.try_borrow() {
                    let text = text.to_lowercase();
                    self.search_result = Some(
                        list.iter()
                            .enumerate()
                            .filter_map(|(i, item)| {
                                if let Some(networks) = item.networks() {
                                    if search_network_ip_item(
                                        &item.value.to_string(),
                                        networks,
                                        &text,
                                    ) {
                                        Some(i)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    );
                }
            }
            Message::InputInput(text) => {
                self.input_wrong_msg = None;
                match ctx.props().kind {
                    Kind::NetworkIp => self.input_text = text.trim_start().to_string(),
                    Kind::Basic => self.input_text = text,
                }
            }
            Message::ClickItem(key, network_kind) => match network_kind {
                ItemKind::Registered => {
                    if let (Ok(mut sel), Ok(list)) = (
                        ctx.props().selected.predefined.try_borrow_mut(),
                        ctx.props().list.try_borrow(),
                    ) {
                        if let Some(predefined) = sel.as_mut() {
                            match predefined.entry(key.clone()) {
                                Vacant(entry) => {
                                    // Item being selected: use direction_items or default (NOT cache)
                                    let extra = if ctx.props().kind == Kind::NetworkIp {
                                        if let Some(existing) =
                                            self.direction_items.get(entry.key())
                                        {
                                            existing.try_borrow().ok().and_then(|v| *v)
                                        } else {
                                            Some(SelectionExtraInfo::Network(EndpointKind::Both))
                                        }
                                    } else {
                                        None
                                    };
                                    entry.insert(Rc::new(RefCell::new(extra)));

                                    // Remove from cache on selection (cache is for deselected items only)
                                    if ctx.props().kind == Kind::NetworkIp
                                        && let Ok(mut cache) =
                                            ctx.props().selected.direction_cache.try_borrow_mut()
                                    {
                                        cache.remove(&key);
                                    }
                                }
                                Occupied(entry) => {
                                    // Item being deselected: save to cache
                                    if ctx.props().kind == Kind::NetworkIp
                                        && let Ok(direction) = entry.get().try_borrow()
                                        && let Some(dir) = *direction
                                        && let Ok(mut cache) =
                                            ctx.props().selected.direction_cache.try_borrow_mut()
                                    {
                                        cache.insert(key.clone(), dir);
                                    }

                                    match ctx.props().kind {
                                        Kind::NetworkIp => {
                                            if entry.get().try_borrow_mut().is_ok() {
                                                entry.remove_entry();
                                            }
                                        }
                                        Kind::Basic => {
                                            entry.remove_entry();
                                        }
                                    }
                                }
                            }
                        } else {
                            let mut s =
                                HashMap::<String, Rc<RefCell<Option<SelectionExtraInfo>>>>::new();
                            let extra = match ctx.props().kind {
                                Kind::NetworkIp => {
                                    Some(SelectionExtraInfo::Network(EndpointKind::Both))
                                }
                                Kind::Basic => None,
                            };
                            for list in list.iter() {
                                s.insert(
                                    list.id().clone(),
                                    self.direction_items
                                        .get(list.id())
                                        .cloned()
                                        .unwrap_or_else(|| Rc::new(RefCell::new(extra))),
                                );
                            }
                            s.remove(&key);
                            *sel = Some(s);

                            // When switching from "all selected" to partial, save the deselected item to cache
                            if ctx.props().kind == Kind::NetworkIp
                                && let Some(dir_rc) = self.direction_items.get(&key)
                                && let Ok(dir) = dir_rc.try_borrow()
                                && let Some(direction) = *dir
                                && let Ok(mut cache) =
                                    ctx.props().selected.direction_cache.try_borrow_mut()
                            {
                                cache.insert(key.clone(), direction);
                            }
                        }
                    }
                    if self.check_status(ctx, false) == CheckStatus::Checked
                        && let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut()
                    {
                        *predefined = None;
                    }
                    self.buffer_direction_items(ctx);
                }
                ItemKind::Custom => {
                    if ctx.props().kind == Kind::NetworkIp
                        && let Ok(custom) = ctx.props().selected.custom.try_borrow()
                        && let Some(value) = custom.get(&key)
                        && let Ok(mut value) = value.try_borrow_mut()
                    {
                        if let Some(SelectionExtraInfo::Network(_)) = *value {
                            // Item being deselected: save direction to cache
                            if let Some(direction) = *value
                                && let Ok(mut cache) =
                                    ctx.props().selected.direction_cache.try_borrow_mut()
                            {
                                cache.insert(key.clone(), direction);
                            }
                            *value = None;
                        } else {
                            // Item being selected: use direction_items or default (NOT cache)
                            *value = self
                                .direction_items
                                .get(&key)
                                .and_then(|existing| existing.try_borrow().ok())
                                .and_then(|opt| *opt)
                                .or(Some(SelectionExtraInfo::Network(EndpointKind::Both)));

                            // Remove from cache on selection (cache is for deselected items only)
                            if let Ok(mut cache) =
                                ctx.props().selected.direction_cache.try_borrow_mut()
                            {
                                cache.remove(&key);
                            }
                        }
                    }
                }
            },
            Message::ClickAll => {
                match self.check_status(ctx, false) {
                    CheckStatus::Checked => {
                        if let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut()
                        {
                            *predefined = Some(HashMap::<
                                String,
                                Rc<RefCell<Option<SelectionExtraInfo>>>,
                            >::new());
                        }
                    }
                    CheckStatus::Unchecked | CheckStatus::Indeterminate => {
                        if cfg!(feature = "pumpkin") {
                            if let (Ok(list), Ok(mut predefined)) = (
                                ctx.props().list.try_borrow(),
                                ctx.props().selected.predefined.try_borrow_mut(),
                            ) {
                                let mut s = HashMap::<
                                    String,
                                    Rc<RefCell<Option<SelectionExtraInfo>>>,
                                >::new();
                                let extra = match ctx.props().kind {
                                    Kind::NetworkIp => {
                                        Some(SelectionExtraInfo::Network(EndpointKind::Both))
                                    }
                                    Kind::Basic => None,
                                };
                                for list in list.iter() {
                                    let id = list.id().clone();
                                    if let Some(existing) = self.direction_items.get(&id) {
                                        s.insert(id, Rc::clone(existing));
                                    } else {
                                        s.insert(id, Rc::new(RefCell::new(extra)));
                                    }
                                }
                                *predefined = Some(s);
                            }
                        } else if let Ok(mut predefined) =
                            ctx.props().selected.predefined.try_borrow_mut()
                        {
                            *predefined = None;
                        }
                    }
                }
                self.buffer_direction_items(ctx);
            }
            Message::ClickAllBelow(network_kind) => match network_kind {
                ItemKind::Registered => {
                    if let Some(search) = self.search_result.as_ref() {
                        let check_status = self.check_status(ctx, true);
                        if let (Ok(mut sel), Ok(list)) = (
                            ctx.props().selected.predefined.try_borrow_mut(),
                            ctx.props().list.try_borrow(),
                        ) {
                            if let Some(predefined) = sel.as_mut() {
                                match check_status {
                                    CheckStatus::Checked => {
                                        // Deselecting items: save directions to cache
                                        for &index in search {
                                            if let Some(item) = list.get(index)
                                                && item.networks().is_some()
                                            {
                                                if ctx.props().kind == Kind::NetworkIp
                                                    && let Some(dir_rc) = predefined.get(item.id())
                                                    && let Ok(dir) = dir_rc.try_borrow()
                                                    && let Some(direction) = *dir
                                                    && let Ok(mut cache) = ctx
                                                        .props()
                                                        .selected
                                                        .direction_cache
                                                        .try_borrow_mut()
                                                {
                                                    cache.insert(item.id().clone(), direction);
                                                }
                                                predefined.remove(item.id());
                                            }
                                        }
                                    }
                                    CheckStatus::Unchecked | CheckStatus::Indeterminate => {
                                        // Selecting items: restore from cache and remove from cache
                                        for &index in search {
                                            if let Some(item) = list.get(index)
                                                && item.networks().is_some()
                                            {
                                                let id = item.id().clone();
                                                // Use direction_items or default (NOT cache)
                                                let extra = if ctx.props().kind == Kind::NetworkIp {
                                                    if let Some(existing) =
                                                        self.direction_items.get(&id)
                                                    {
                                                        existing.try_borrow().ok().and_then(|v| *v)
                                                    } else {
                                                        Some(SelectionExtraInfo::Network(
                                                            EndpointKind::Both,
                                                        ))
                                                    }
                                                } else {
                                                    None
                                                };
                                                predefined.insert(
                                                    id.clone(),
                                                    Rc::new(RefCell::new(extra)),
                                                );

                                                // Remove from cache on selection (cache is for deselected items only)
                                                if ctx.props().kind == Kind::NetworkIp
                                                    && let Ok(mut cache) = ctx
                                                        .props()
                                                        .selected
                                                        .direction_cache
                                                        .try_borrow_mut()
                                                {
                                                    cache.remove(&id);
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                let mut s = HashMap::<
                                    String,
                                    Rc<RefCell<Option<SelectionExtraInfo>>>,
                                >::new();
                                for item in list.iter() {
                                    if let Some(dir) = self.direction_items.get(item.id())
                                        && let Ok(dir) = dir.try_borrow()
                                    {
                                        s.insert(item.id().clone(), Rc::new(RefCell::new(*dir)));
                                    }
                                }
                                for &index in search {
                                    if let Some(item) = list.get(index) {
                                        let id = item.id();
                                        // Deselecting items when switching from "all": save to cache
                                        if ctx.props().kind == Kind::NetworkIp
                                            && let Some(dir_rc) = self.direction_items.get(id)
                                            && let Ok(dir) = dir_rc.try_borrow()
                                            && let Some(direction) = *dir
                                            && let Ok(mut cache) = ctx
                                                .props()
                                                .selected
                                                .direction_cache
                                                .try_borrow_mut()
                                        {
                                            cache.insert(id.clone(), direction);
                                        }
                                        s.remove(id);
                                    }
                                }
                                *sel = Some(s);
                            }
                        }
                        if !cfg!(feature = "pumpkin")
                            && check_status != CheckStatus::Checked
                            && self.check_status(ctx, false) == CheckStatus::Checked
                            && let Ok(mut sel) = ctx.props().selected.predefined.try_borrow_mut()
                        {
                            *sel = None;
                        }
                        self.buffer_direction_items(ctx);
                    } else {
                        ctx.link().send_message(Message::ClickAll);
                        return false;
                    }
                }
                ItemKind::Custom => {
                    if ctx.props().kind == Kind::NetworkIp {
                        let status = Self::check_custom_status(ctx);
                        if let Ok(mut custom) = ctx.props().selected.custom.try_borrow_mut() {
                            match status {
                                CheckStatus::Checked => {
                                    // Deselecting all custom items: save directions to cache
                                    for (k, v) in &mut *custom {
                                        if let Ok(mut vv) = v.try_borrow_mut()
                                            && vv.is_some()
                                        {
                                            if let Some(direction) = *vv
                                                && let Ok(mut cache) = ctx
                                                    .props()
                                                    .selected
                                                    .direction_cache
                                                    .try_borrow_mut()
                                            {
                                                cache.insert(k.clone(), direction);
                                            }
                                            *vv = None;
                                        }
                                    }
                                }
                                CheckStatus::Unchecked | CheckStatus::Indeterminate => {
                                    // Selecting custom items: use direction_items or default (NOT cache)
                                    for (k, v) in custom.iter_mut() {
                                        if let Ok(mut vv) = v.try_borrow_mut()
                                            && vv.is_none()
                                        {
                                            if let Some(existing) = self.direction_items.get(k) {
                                                if let Ok(existing) = existing.try_borrow() {
                                                    *vv = *existing;
                                                }
                                            } else {
                                                *vv = Some(SelectionExtraInfo::Network(
                                                    EndpointKind::Both,
                                                ));
                                            }

                                            // Remove from cache on selection (cache is for deselected items only)
                                            if let Ok(mut cache) = ctx
                                                .props()
                                                .selected
                                                .direction_cache
                                                .try_borrow_mut()
                                            {
                                                cache.remove(k);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Message::ClickAddInput => {
                if cfg!(feature = "pumpkin") {
                    if !self.view_input {
                        self.view_input = true;
                    }
                    if self.view_list {
                        self.view_list = false;
                    }
                }
                if self.validate_user_input(ctx) {
                    if let Ok(mut custom) = ctx.props().selected.custom.try_borrow_mut() {
                        match custom.entry(self.input_text.clone()) {
                            Vacant(entry) => {
                                let extra = match ctx.props().kind {
                                    Kind::NetworkIp => {
                                        Some(SelectionExtraInfo::Network(EndpointKind::Both))
                                    }
                                    Kind::Basic => None,
                                };
                                entry.insert(Rc::new(RefCell::new(extra)));
                                self.input_text.clear();
                                self.input_wrong_msg = None;
                            }
                            Occupied(_) => {
                                self.input_wrong_msg = Some("The input already exists.");
                            }
                        }
                    }
                } else {
                    self.input_wrong_msg = Some(
                        "Invalid input (valid examples: 10.84.1.7, 10.1.1.1 - 10.1.1.20, 192.168.10.0/24)",
                    );
                }
            }
            Message::DeleteInputItem(key) => {
                if let Ok(mut custom) = ctx.props().selected.custom.try_borrow_mut() {
                    if let Occupied(entry) = custom.entry(key) {
                        entry.remove_entry();
                    }
                    if custom.is_empty() {
                        self.view_input = false;
                    }
                }
            }
            Message::SetDirection => {
                self.set_direction(ctx);
                self.load_direction_items(ctx);
            }
            Message::SetDirectionItem(network_kind) => match network_kind {
                ItemKind::Registered => {
                    self.load_direction_items(ctx);
                }
                ItemKind::Custom => {
                    self.set_direction_custom(ctx);
                }
            },
            Message::Render => {
                let check_status = self.check_status(ctx, false);
                if let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut() {
                    match check_status {
                        CheckStatus::Checked => {
                            if !cfg!(feature = "pumpkin") {
                                *predefined = None;
                            }
                        }
                        CheckStatus::Unchecked | CheckStatus::Indeterminate => (),
                    }
                }
            }
            Message::InputError => {
                // TODO: issue #5
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = format!("width: {}px;", ctx.props().top_width);
        let onclick = ctx.link().callback(|_| Message::Click);
        let mut class_input = "complex-select-input";
        let txt = ctx.props().txt.txt.clone();
        let check_status = self.check_status(ctx, false);
        let value = if cfg!(feature = "pumpkin") {
            if let Ok(list) = ctx.props().list.try_borrow() {
                if list.is_empty() {
                    class_input = "complex-select-input";
                    text!(txt, ctx.props().language, "All").to_string()
                } else {
                    let (list_selected, custom_selected) = ctx.props().selected.len();
                    if list_selected.unwrap_or_default() == 0 && custom_selected == 0 {
                        text!(txt, ctx.props().language, "All").to_string()
                    } else {
                        let selected_len = Self::selected_len(ctx);
                        if selected_len == 0 {
                            text!(txt, ctx.props().language, "All").to_string()
                        } else {
                            format!(
                                "({}) {}",
                                selected_len,
                                text!(txt, ctx.props().language, "Selected Filters"),
                            )
                        }
                    }
                }
            } else {
                "complex-select-input-empty".to_string()
            }
        } else if let Ok(list) = ctx.props().list.try_borrow() {
            if list.is_empty() {
                class_input = "complex-select-input-empty";
                text!(txt, ctx.props().language, "None").to_string()
            } else if check_status == CheckStatus::Checked {
                text!(txt, ctx.props().language, "All").to_string()
            } else if ctx.props().selected.is_empty() {
                if ctx.props().allow_empty {
                    class_input = "complex-select-input-empty";
                } else {
                    class_input = "complex-select-input-empty-alert";
                }
                text!(txt, ctx.props().language, &ctx.props().empty_msg).to_string()
            } else {
                format!(
                    "({}) {}",
                    Self::selected_len(ctx),
                    text!(txt, ctx.props().language, &ctx.props().title)
                )
            }
        } else {
            "complex-select-input-empty".to_string()
        };

        html! {
            <div class="complex-select">
                <div onclick={onclick} class="complex-select-top">
                    <input type="text" class={classes!("complex-select-top-input", class_input)} readonly={true} value={value} style={style} />
                </div>
                { self.view_pop(ctx) }
            </div>
        }
    }
}

impl Model {
    pub(super) fn check_status(&self, ctx: &Context<Self>, search: bool) -> CheckStatus {
        if let (Ok(predefined), Ok(list)) = (
            ctx.props().selected.predefined.try_borrow(),
            ctx.props().list.try_borrow(),
        ) {
            predefined.as_ref().map_or_else(
                || {
                    self.search_result
                        .as_ref()
                        .map_or(CheckStatus::Checked, |search_result| {
                            if search_result.is_empty() {
                                CheckStatus::Unchecked
                            } else {
                                CheckStatus::Checked
                            }
                        })
                },
                |selected| {
                    let mut indeterminate = false;
                    let (all_len, match_len) = if search {
                        self.search_result.as_ref().map_or((0, 0), |search| {
                            (
                                search.len(),
                                search
                                    .iter()
                                    .filter_map(|&index| {
                                        list.get(index).and_then(|item| {
                                            if item.networks().is_some() {
                                                match check_network(item.id(), selected) {
                                                    CheckStatus::Checked => Some(true),
                                                    CheckStatus::Indeterminate => {
                                                        indeterminate = true;
                                                        None
                                                    }
                                                    CheckStatus::Unchecked => None,
                                                }
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                    .count(),
                            )
                        })
                    } else {
                        (
                            list.len(),
                            list.iter()
                                .filter_map(|item| {
                                    if item.networks().is_some() {
                                        match check_network(item.id(), selected) {
                                            CheckStatus::Checked => Some(true),
                                            CheckStatus::Indeterminate => {
                                                indeterminate = true;
                                                None
                                            }
                                            CheckStatus::Unchecked => None,
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .count(),
                        )
                    };

                    if match_len == 0 && indeterminate {
                        CheckStatus::Indeterminate
                    } else if match_len == 0 {
                        CheckStatus::Unchecked
                    } else if match_len == all_len {
                        CheckStatus::Checked
                    } else {
                        CheckStatus::Indeterminate
                    }
                },
            )
        } else {
            CheckStatus::Unchecked
        }
    }

    fn set_direction(&mut self, ctx: &Context<Self>) {
        if ctx.props().kind != Kind::NetworkIp {
            return;
        }
        if let Ok(direction) = self.directions.registered.try_borrow()
            && let Some(direction) = direction.as_ref()
        {
            if let (Some(search), Ok(list)) =
                (self.search_result.as_ref(), ctx.props().list.try_borrow())
            {
                // Update direction for visible/searched items
                for &index in search {
                    if let Some(item) = list.get(index)
                        && item.networks().is_some()
                    {
                        let value = self.direction_items.get(&item.id);
                        if let Some(value) = value
                            && let Ok(mut value) = value.try_borrow_mut()
                            && let Some(SelectionExtraInfo::Network(_)) = value.as_ref()
                        {
                            *value = Some(SelectionExtraInfo::Network(*direction));

                            // Only persist to cache if item is deselected
                            if let Ok(predefined) = ctx.props().selected.predefined.try_borrow()
                                && predefined
                                    .as_ref()
                                    .is_some_and(|map| !map.contains_key(item.id()))
                                && let Ok(mut cache) =
                                    ctx.props().selected.direction_cache.try_borrow_mut()
                            {
                                cache.insert(
                                    item.id().clone(),
                                    SelectionExtraInfo::Network(*direction),
                                );
                            }
                        }
                    }
                }
            } else {
                // Update direction for all items
                for (key, value) in &self.direction_items {
                    if let Ok(mut value) = value.try_borrow_mut()
                        && let Some(SelectionExtraInfo::Network(_)) = value.as_ref()
                    {
                        *value = Some(SelectionExtraInfo::Network(*direction));

                        // Only persist to cache if item is deselected
                        if let Ok(predefined) = ctx.props().selected.predefined.try_borrow()
                            && predefined
                                .as_ref()
                                .is_some_and(|map| !map.contains_key(key))
                            && let Ok(mut cache) =
                                ctx.props().selected.direction_cache.try_borrow_mut()
                        {
                            cache.insert(key.clone(), SelectionExtraInfo::Network(*direction));
                        }
                    }
                }
            }
        }
    }

    #[inline]
    pub(super) fn check_custom_status(ctx: &Context<Self>) -> CheckStatus {
        if let Ok(custom) = ctx.props().selected.custom.try_borrow() {
            if custom.is_empty() {
                return CheckStatus::Unchecked;
            }
            let mut all_none = true;
            let mut all_some = true;
            for v in custom.values() {
                if let Ok(v) = v.try_borrow() {
                    if v.is_none() {
                        all_some = false;
                    } else {
                        all_none = false;
                    }
                } else {
                    all_none = false;
                    all_some = false;
                    break;
                }
            }
            if all_none {
                CheckStatus::Unchecked
            } else if all_some {
                CheckStatus::Checked
            } else {
                CheckStatus::Indeterminate
            }
        } else {
            CheckStatus::Unchecked
        }
    }

    fn set_direction_custom(&mut self, ctx: &Context<Self>) {
        if ctx.props().kind != Kind::NetworkIp {
            return;
        }
        if let Ok(direction) = self.directions.custom.try_borrow()
            && let Some(direction) = direction.as_ref()
            && let Ok(mut custom) = ctx.props().selected.custom.try_borrow_mut()
        {
            for (key, v) in custom.iter_mut() {
                if let Ok(mut vv) = v.try_borrow_mut() {
                    if vv.is_some() {
                        // Item is selected: update its direction directly (don't persist to cache)
                        *vv = Some(SelectionExtraInfo::Network(*direction));
                    } else {
                        // Item is deselected: update cache only
                        if let Ok(mut cache) = ctx.props().selected.direction_cache.try_borrow_mut()
                        {
                            cache.insert(key.clone(), SelectionExtraInfo::Network(*direction));
                        }
                    }
                }
            }
        }
    }

    fn buffer_direction_items(&mut self, ctx: &Context<Self>) {
        if ctx.props().kind != Kind::NetworkIp {
            self.direction_items.clear();
            return;
        }
        let (Ok(predefined), Ok(list)) = (
            ctx.props().selected.predefined.try_borrow(),
            ctx.props().list.try_borrow(),
        ) else {
            self.direction_items.clear();
            return;
        };

        // If all items are selected (predefined is None), clear the cache
        // since the cache is only for deselected items
        if predefined.is_none()
            && let Ok(mut cache) = ctx.props().selected.direction_cache.try_borrow_mut()
        {
            cache.clear();
        }

        let mut current_ids = HashSet::new();
        for item in list.iter() {
            let id = item.id();
            current_ids.insert(id);

            // Determine the direction to use for this item
            let direction = if let Some(predefined_map) = predefined.as_ref() {
                if let Some(selected_rc) = predefined_map.get(id) {
                    // Item is selected: use the direction from predefined selection
                    selected_rc.try_borrow().ok().and_then(|v| *v)
                } else {
                    // Item is deselected: restore from direction_cache, then fall back to existing direction_items, then default
                    if let Ok(cache) = ctx.props().selected.direction_cache.try_borrow()
                        && let Some(cached_direction) = cache.get(id)
                    {
                        Some(*cached_direction)
                    } else if let Some(existing) = self.direction_items.get(id) {
                        existing.try_borrow().ok().and_then(|v| *v)
                    } else {
                        Some(SelectionExtraInfo::Network(EndpointKind::Both))
                    }
                }
            } else {
                // All items selected (predefined is None): use existing direction_items or default
                // Do NOT read from cache since cache is only for deselected items
                if let Some(existing) = self.direction_items.get(id) {
                    existing.try_borrow().ok().and_then(|v| *v)
                } else {
                    Some(SelectionExtraInfo::Network(EndpointKind::Both))
                }
            };

            // Update or insert the direction in direction_items
            match self.direction_items.entry(id.clone()) {
                Occupied(mut entry) => {
                    if let Ok(mut value) = entry.get().try_borrow_mut() {
                        *value = direction;
                    } else {
                        *entry.get_mut() = Rc::new(RefCell::new(direction));
                    }
                }
                Vacant(entry) => {
                    entry.insert(Rc::new(RefCell::new(direction)));
                }
            }
        }

        // Clean up stale entries: retain only items that exist in current list
        self.direction_items
            .retain(|id, _| current_ids.contains(id));

        // Clean up stale cache entries to avoid memory bloat
        if let Ok(mut cache) = ctx.props().selected.direction_cache.try_borrow_mut() {
            cache.retain(|id, _| current_ids.contains(id));
        }
    }

    fn load_direction_items(&mut self, ctx: &Context<Self>) {
        if let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut()
            && let Some(current) = predefined.as_ref()
        {
            // Update selected items with new directions from direction_items
            // Don't persist to cache since these are selected items
            let s = current
                .iter()
                .filter_map(|(k, v)| {
                    self.direction_items
                        .get(k)
                        .and_then(|rc| rc.try_borrow().ok())
                        .map(|b| (k.clone(), Rc::new(RefCell::new(*b))))
                        .or_else(|| Some((k.clone(), Rc::clone(v))))
                })
                .collect::<HashMap<String, Rc<RefCell<Option<SelectionExtraInfo>>>>>();
            *predefined = Some(s);
        }
        if !cfg!(feature = "pumpkin")
            && self.check_status(ctx, false) == CheckStatus::Checked
            && let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut()
        {
            *predefined = None;
        }
    }

    fn validate_user_input(&mut self, ctx: &Context<Self>) -> bool {
        match ctx.props().kind {
            Kind::NetworkIp => {
                let (valid, range) = validate_host_network(&self.input_text);
                if valid {
                    if let Some(range) = range {
                        self.input_text = range;
                    }
                    return true;
                }
            }
            Kind::Basic => (),
        }
        false
    }

    pub(super) fn selected_len(ctx: &Context<Self>) -> usize {
        let len = ctx.props().selected.len();
        let list_len = if let Ok(list) = ctx.props().list.try_borrow() {
            list.len()
        } else {
            0
        };
        len.0.unwrap_or(list_len) + len.1
    }
}

#[inline]
pub(super) fn check_network(
    id: &String,
    selected: &HashMap<String, Rc<RefCell<Option<SelectionExtraInfo>>>>,
) -> CheckStatus {
    selected
        .get(id)
        .map_or(CheckStatus::Unchecked, |direction| {
            if let Ok(direction) = direction.try_borrow() {
                match *direction {
                    Some(SelectionExtraInfo::Network(_)) => CheckStatus::Checked,
                    _ => CheckStatus::Unchecked,
                }
            } else {
                CheckStatus::Unchecked
            }
        })
}

#[inline]
fn search_network_ip_item(name: &str, networks: &NetworkGroup, text: &str) -> bool {
    if name.to_lowercase().contains(text) {
        return true;
    }
    for host in &networks.hosts {
        if host.contains(text) {
            return true;
        }
    }
    for n in &networks.networks {
        if n.contains(text) {
            return true;
        }
    }
    for r in &networks.ranges {
        if r.start.contains(text) {
            return true;
        }
        if r.end.contains(text) {
            return true;
        }
    }
    false
}
