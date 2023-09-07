use super::DEFAULT_POP_WIDTH;
use crate::{
    language::Language, text, toggle_visibility_complex, validate_host_network, CheckStatus,
    ComplexSelection, EndpointKind, NetworkGroup, NetworkItem, SelectionExtraInfo, Texts,
};
use json_gettext::get_text;
use std::cell::RefCell;
use std::collections::{
    hash_map::Entry::{Occupied, Vacant},
    HashMap, HashSet,
};
use std::rc::Rc;
use yew::virtual_dom::AttrValue;
use yew::{classes, html, Component, Context, Html, Properties};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Kind {
    Basic,
    NetworkIp,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, PartialEq, Eq)]
pub struct Model {
    pub(super) search_result: Option<Vec<usize>>,
    pub(super) search_text: String,
    pub(super) input_text: String,
    pub(super) input_wrong_msg: Option<&'static str>,
    pub(super) direction: Rc<RefCell<Option<EndpointKind>>>, // for Network/IP
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
    ClickItem(String),
    ClickAll,
    ClickAllBelow,
    ClickAddInput,
    DeleteInputItem(String),
    Render,
    SetDirection,
    SetDirectionItem,
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
    pub font: AttrValue,
    pub list: Rc<RefCell<Vec<NetworkItem>>>,
    pub selected: Rc<ComplexSelection>,
    #[prop_or(true)]
    pub allow_input: bool,
    #[prop_or(false)]
    pub allow_empty: bool,
    #[prop_or(true)]
    pub default_all: bool,
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
            direction: Rc::new(RefCell::new(None)),
            direction_items: HashMap::new(),
        };
        s.buffer_direction_items(ctx);

        s
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if let (Ok(mut sel), Ok(list)) = (
            ctx.props().selected.predefined.try_borrow_mut(),
            ctx.props().list.try_borrow(),
        ) {
            if let Some(predefined) = sel.as_mut() {
                let list_tmp = list
                    .iter()
                    .map(NetworkItem::id)
                    .collect::<HashSet<&String>>();
                predefined.retain(|k, _| list_tmp.get(k).is_some());
                if self.check_status(ctx, false) == CheckStatus::Checked {
                    *sel = None;
                }
            }
        }
        self.buffer_direction_items(ctx);

        true
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Click | Message::Close => {
                toggle_visibility_complex(&ctx.props().id);
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
                self.search_text = text.clone();
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
            Message::ClickItem(key) => {
                if let (Ok(mut sel), Ok(list)) = (
                    ctx.props().selected.predefined.try_borrow_mut(),
                    ctx.props().list.try_borrow(),
                ) {
                    if let Some(predefined) = sel.as_mut() {
                        match predefined.entry(key) {
                            Vacant(entry) => {
                                let extra = match ctx.props().kind {
                                    Kind::NetworkIp => {
                                        Some(SelectionExtraInfo::Network(EndpointKind::Both))
                                    }
                                    Kind::Basic => None,
                                };
                                entry.insert(Rc::new(RefCell::new(extra)));
                            }
                            Occupied(entry) => match ctx.props().kind {
                                Kind::NetworkIp => {
                                    let remove = if let Ok(mut extra) = entry.get().try_borrow_mut()
                                    {
                                        if *extra
                                            == Some(SelectionExtraInfo::Network(EndpointKind::Both))
                                        {
                                            true
                                        } else {
                                            *extra = Some(SelectionExtraInfo::Network(
                                                EndpointKind::Both,
                                            ));
                                            false
                                        }
                                    } else {
                                        false
                                    };
                                    if remove {
                                        entry.remove_entry();
                                    }
                                }
                                Kind::Basic => {
                                    entry.remove_entry();
                                }
                            },
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
                            s.insert(list.id().clone(), Rc::new(RefCell::new(extra)));
                        }
                        s.remove(&key);
                        *sel = Some(s);
                    }
                }
                if self.check_status(ctx, false) == CheckStatus::Checked {
                    if let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut() {
                        *predefined = None;
                    }
                }
                self.buffer_direction_items(ctx);
            }
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
                        if let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut()
                        {
                            *predefined = None;
                        }
                    }
                }
                self.buffer_direction_items(ctx);
            }
            Message::ClickAllBelow => {
                if let Some(search) = self.search_result.as_ref() {
                    let check_status = self.check_status(ctx, true);
                    if let (Ok(mut sel), Ok(list)) = (
                        ctx.props().selected.predefined.try_borrow_mut(),
                        ctx.props().list.try_borrow(),
                    ) {
                        if let Some(predefined) = sel.as_mut() {
                            match check_status {
                                CheckStatus::Checked => {
                                    for &index in search {
                                        if let Some(item) = list.get(index) {
                                            if item.networks().is_some() {
                                                predefined.remove(item.id());
                                            }
                                        }
                                    }
                                }
                                CheckStatus::Unchecked | CheckStatus::Indeterminate => {
                                    for &index in search {
                                        if let Some(item) = list.get(index) {
                                            if item.networks().is_some() {
                                                check_item_as_both(item.id(), predefined);
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            let mut s =
                                HashMap::<String, Rc<RefCell<Option<SelectionExtraInfo>>>>::new();
                            for item in list.iter() {
                                if let Some(dir) = self.direction_items.get(item.id()) {
                                    if let Ok(dir) = dir.try_borrow() {
                                        s.insert(item.id().clone(), Rc::new(RefCell::new(*dir)));
                                    }
                                }
                            }
                            for &index in search {
                                if let Some(item) = list.get(index) {
                                    s.remove(item.id());
                                }
                            }
                            *sel = Some(s);
                        }
                    }
                    if check_status != CheckStatus::Checked
                        && self.check_status(ctx, false) == CheckStatus::Checked
                    {
                        if let Ok(mut sel) = ctx.props().selected.predefined.try_borrow_mut() {
                            *sel = None;
                        }
                    };
                    self.buffer_direction_items(ctx);
                } else {
                    ctx.link().send_message(Message::ClickAll);
                    return false;
                };
            }
            Message::ClickAddInput => {
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
                            }
                            Occupied(_) => {
                                self.input_wrong_msg = Some("The input already exists.");
                            }
                        }
                    }
                } else {
                    self.input_wrong_msg = Some("Invalid input (valid examples: 10.84.1.7, 10.1.1.1 ~ 10.1.1.20, 192.168.10.0/24)");
                };
            }
            Message::DeleteInputItem(key) => {
                if let Ok(mut custom) = ctx.props().selected.custom.try_borrow_mut() {
                    if let Occupied(entry) = custom.entry(key) {
                        entry.remove_entry();
                    }
                };
            }
            Message::SetDirection => {
                self.set_direction(ctx);
                self.load_direction_items(ctx);
            }
            Message::SetDirectionItem => {
                self.load_direction_items(ctx);
            }
            Message::Render => {
                let check_status = self.check_status(ctx, false);
                if let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut() {
                    match check_status {
                        CheckStatus::Checked => *predefined = None,
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
        let value = if let Ok(list) = ctx.props().list.try_borrow() {
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
        if let Ok(direction) = self.direction.try_borrow() {
            if let Some(direction) = direction.as_ref() {
                if let (Some(search), Ok(list)) =
                    (self.search_result.as_ref(), ctx.props().list.try_borrow())
                {
                    for &index in search {
                        if let Some(item) = list.get(index) {
                            if item.networks().is_some() {
                                let value = self.direction_items.get(&item.id);
                                if let Some(value) = value {
                                    if let Ok(mut value) = value.try_borrow_mut() {
                                        if let Some(SelectionExtraInfo::Network(_)) = value.as_ref()
                                        {
                                            *value = Some(SelectionExtraInfo::Network(*direction));
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    for value in self.direction_items.values() {
                        if let Ok(mut value) = value.try_borrow_mut() {
                            if let Some(SelectionExtraInfo::Network(_)) = value.as_ref() {
                                *value = Some(SelectionExtraInfo::Network(*direction));
                            }
                        }
                    }
                }
            }
        }
    }

    fn buffer_direction_items(&mut self, ctx: &Context<Self>) {
        self.direction_items = if let (Ok(predefined), Ok(list)) = (
            ctx.props().selected.predefined.try_borrow(),
            ctx.props().list.try_borrow(),
        ) {
            if ctx.props().kind == Kind::NetworkIp {
                list.iter()
                    .map(|item| {
                        (item.id().clone(), {
                            predefined.as_ref().map_or_else(
                                || {
                                    Rc::new(RefCell::new(Some(SelectionExtraInfo::Network(
                                        EndpointKind::Both,
                                    ))))
                                },
                                |predefined| {
                                    predefined.get(item.id()).map_or(
                                        Rc::new(RefCell::new(None)),
                                        |d| {
                                            if let Ok(d) = d.try_borrow() {
                                                Rc::new(RefCell::new(*d))
                                            } else {
                                                Rc::new(RefCell::new(None))
                                            }
                                        },
                                    )
                                },
                            )
                        })
                    })
                    .collect::<HashMap<String, Rc<RefCell<Option<SelectionExtraInfo>>>>>()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };
    }

    fn load_direction_items(&mut self, ctx: &Context<Self>) {
        if let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut() {
            let s = self
                .direction_items
                .iter()
                .filter_map(|(k, v)| {
                    if let Ok(v) = v.try_borrow() {
                        Some((k.clone(), Rc::new(RefCell::new(*v))))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<String, Rc<RefCell<Option<SelectionExtraInfo>>>>>();
            *predefined = Some(s);
        }
        if self.check_status(ctx, false) == CheckStatus::Checked {
            if let Ok(mut predefined) = ctx.props().selected.predefined.try_borrow_mut() {
                *predefined = None;
            }
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
fn check_item_as_both(
    id: &String,
    selected: &mut HashMap<String, Rc<RefCell<Option<SelectionExtraInfo>>>>,
) {
    if let Some(value) = selected.get(id) {
        if let Ok(mut value) = value.try_borrow_mut() {
            *value = Some(SelectionExtraInfo::Network(EndpointKind::Both));
        }
    } else {
        selected.insert(
            id.clone(),
            Rc::new(RefCell::new(Some(SelectionExtraInfo::Network(
                EndpointKind::Both,
            )))),
        );
    }
}

#[inline]
fn check_network(
    id: &String,
    selected: &HashMap<String, Rc<RefCell<Option<SelectionExtraInfo>>>>,
) -> CheckStatus {
    selected
        .get(id)
        .map_or(CheckStatus::Unchecked, |direction| {
            if let Ok(direction) = direction.try_borrow() {
                direction.map_or(CheckStatus::Unchecked, |direction| {
                    if direction == SelectionExtraInfo::Network(EndpointKind::Both) {
                        CheckStatus::Checked
                    } else {
                        CheckStatus::Indeterminate
                    }
                })
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
