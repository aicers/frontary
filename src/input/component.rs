use super::{
    user_input::MAX_PER_LAYER, InputHostNetworkGroup, InputItem, InputTag, InputTagGroup,
    InputType, Value as ComparisonValue,
};
use crate::{
    language::Language,
    list::{Column, ListItem},
    sort_hosts, sort_networks, text, InputNic, MessageType, Rerender, Texts, ViewString,
};
use data_encoding::BASE64;
use gloo_file::{
    callbacks::{read_as_bytes, FileReader},
    File,
};
use json_gettext::get_text;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};
use yew::{html, virtual_dom::AttrValue, Component, Context, Html, Properties};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum InvalidMessage {
    InvalidInput,
    PasswordHasSpace,
    PasswordHasControlCharacter,
    PasswordNotMatch,
    PasswordTooShort,
    PasswordNoLowercaseLetter,
    PasswordNoUppercaseLetter,
    PasswordNoNumber,
    PasswordNoSymbol,
    PasswordHasConsecutiveLetters,
    PasswordHasAdjacentLetters,
    InterfaceNameRequired,
    InterfaceRequired,
    GatewayRequired,
    WrongInterface,
    WrongGateway,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Verification {
    Valid,
    Invalid(InvalidMessage),
}

#[derive(Clone, PartialEq, Eq)]
pub enum InputSecondId {
    Add,
    Edit(String),
}

type CompValueBuf = HashMap<
    usize,
    (
        Rc<RefCell<Option<ComparisonValue>>>,
        Rc<RefCell<Option<ComparisonValue>>>,
    ),
>;

type VecSelectBuf = HashMap<usize, Vec<Rc<RefCell<Option<HashSet<String>>>>>>;

pub struct Model<T> {
    pub(super) radio_buffer: HashMap<usize, Rc<RefCell<String>>>,
    pub(super) host_network_buffer: HashMap<usize, Rc<RefCell<InputHostNetworkGroup>>>,
    pub(super) select_searchable_buffer: HashMap<usize, Rc<RefCell<Option<HashSet<String>>>>>,
    pub(super) vec_select_buffer: VecSelectBuf,
    pub(super) tag_buffer: HashMap<usize, Rc<RefCell<InputTagGroup>>>,
    pub(super) comparison_value_kind_buffer: HashMap<usize, Rc<RefCell<Option<HashSet<String>>>>>,
    pub(super) comparison_value_cmp_buffer: HashMap<usize, Rc<RefCell<Option<HashSet<String>>>>>,
    pub(super) comparison_value_buffer: CompValueBuf,

    pub(super) confirm_password: HashMap<usize, String>,
    pub(super) unique_msg: HashSet<usize>,
    pub(super) required_msg: HashSet<usize>,

    pub(super) verification: HashMap<usize, Verification>,
    pub(super) verification_nic: HashMap<(usize, usize), Verification>, // 2nd usize: 0 -> name, 1 -> interface ip, 2 -> gateway ip
    pub(super) verification_host_network: HashMap<usize, Option<bool>>, // None means no checking done yet, Some(true) valid Some(false) invalid
    pub(super) verify_host_network_group: bool,

    file_data_id: Option<usize>,
    file_input_data: Option<Rc<RefCell<InputItem>>>,
    file_name: Option<String>,
    file_content: Option<String>,
    file_reader: Option<FileReader>,

    pub(super) rerender_serial_host_network: u64,

    phantom: PhantomData<T>,
}

impl<T> PartialEq for Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.radio_buffer == other.radio_buffer
            && self.host_network_buffer == other.host_network_buffer
            && self.select_searchable_buffer == other.select_searchable_buffer
            && self.vec_select_buffer == other.vec_select_buffer
            && self.tag_buffer == other.tag_buffer
            && self.comparison_value_kind_buffer == other.comparison_value_kind_buffer
            && self.comparison_value_cmp_buffer == other.comparison_value_cmp_buffer
            && self.comparison_value_buffer == other.comparison_value_buffer
            && self.confirm_password == other.confirm_password
            && self.unique_msg == other.unique_msg
            && self.required_msg == other.required_msg
            && self.verification == other.verification
            && self.verification_nic == other.verification_nic
            && self.verification_host_network == other.verification_host_network
            && self.verify_host_network_group == other.verify_host_network_group
            && self.file_data_id == other.file_data_id
            && self.file_input_data == other.file_input_data
            && self.file_name == other.file_name
            && self.file_content == other.file_content
            && self.rerender_serial_host_network == other.rerender_serial_host_network
    }
}

impl<T> Clone for Model<T> {
    fn clone(&self) -> Self {
        Self {
            radio_buffer: self.radio_buffer.clone(),
            host_network_buffer: self.host_network_buffer.clone(),
            select_searchable_buffer: self.select_searchable_buffer.clone(),
            vec_select_buffer: self.vec_select_buffer.clone(),
            tag_buffer: self.tag_buffer.clone(),
            comparison_value_kind_buffer: self.comparison_value_kind_buffer.clone(),
            comparison_value_cmp_buffer: self.comparison_value_cmp_buffer.clone(),
            comparison_value_buffer: self.comparison_value_buffer.clone(),
            confirm_password: self.confirm_password.clone(),
            unique_msg: self.unique_msg.clone(),
            required_msg: self.required_msg.clone(),
            verification: self.verification.clone(),
            verification_nic: self.verification_nic.clone(),
            verification_host_network: self.verification_host_network.clone(),
            verify_host_network_group: self.verify_host_network_group,
            file_data_id: self.file_data_id,
            file_input_data: self.file_input_data.clone(),
            file_name: self.file_name.clone(),
            file_content: self.file_content.clone(),
            file_reader: None,
            rerender_serial_host_network: self.rerender_serial_host_network,
            phantom: PhantomData,
        }
    }
}

impl<T> Rerender for Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    fn rerender_serial(&mut self) -> &mut u64 {
        &mut self.rerender_serial_host_network
    }
}

pub enum Message {
    Escape,
    Save,
    TrySave,
    InputText(usize, String, Rc<RefCell<InputItem>>),
    InputPassword(usize, String, Rc<RefCell<InputItem>>),
    InputConfirmPassword(usize, String),
    InputUnsigned32(usize, u32, Rc<RefCell<InputItem>>),
    InputFloat64(usize, f64, Rc<RefCell<InputItem>>),
    InvalidInputUnsigned32,
    InvalidInputFloat64,
    InputPercentage(usize, f32, Rc<RefCell<InputItem>>),
    InvalidInputPercentage,
    InputRadio(usize, Rc<RefCell<InputItem>>),
    InputHostNetworkGroup(usize, Rc<RefCell<InputItem>>),
    InputMultipleSelect(usize, Rc<RefCell<InputItem>>, Rc<Vec<(String, ViewString)>>),
    InputSingleSelect(usize, Rc<RefCell<InputItem>>, Rc<Vec<(String, ViewString)>>),
    InputVecSelect(
        usize, // data_id
        usize, // col_id
        Rc<RefCell<InputItem>>,
    ),
    InputTagGroup(usize, Rc<RefCell<InputItem>>),
    UserInputHostNetworkGroup(usize),
    WrongHostNetworkGroup(usize),
    RightHostNetworkGroup(usize, Rc<RefCell<InputItem>>),
    ClickCheckBox(Rc<RefCell<InputItem>>),
    InputNicName(usize, usize, String, Rc<RefCell<InputItem>>),
    InputNicInterface(usize, usize, String, Rc<RefCell<InputItem>>),
    InputNicGateway(usize, usize, String, Rc<RefCell<InputItem>>),
    InputNicAdd(usize, usize, Rc<RefCell<InputItem>>),
    InputNicDelete(usize, usize, Rc<RefCell<InputItem>>),
    InputGroupAdd(usize, Rc<RefCell<InputItem>>, Option<InputItem>),
    InputGroupDelete(usize, usize, Rc<RefCell<InputItem>>, Option<InputItem>),
    InputComparisonValueKind(usize, Rc<RefCell<InputItem>>),
    InputComparisonComparisionKind(usize, Rc<RefCell<InputItem>>),
    InputComparisonValue(usize, usize, ComparisonValue, Rc<RefCell<InputItem>>),
    InvalidInputComparisonValue,
    // TODO: extend multiple files
    ChooseFile(usize, Vec<File>, Rc<RefCell<InputItem>>),
    FileLoaded(String, Vec<u8>),
    FailLoadFile,
    InputError,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        match self {
            Self::Escape => Self::Escape,
            Self::Save => Self::Save,
            Self::TrySave => Self::TrySave,
            Self::InputText(a, b, c) => Self::InputText(*a, b.clone(), c.clone()),
            Self::InputPassword(a, b, c) => Self::InputPassword(*a, b.clone(), c.clone()),
            Self::InputConfirmPassword(a, b) => Self::InputConfirmPassword(*a, b.clone()),
            Self::InputUnsigned32(a, b, c) => Self::InputUnsigned32(*a, *b, c.clone()),
            Self::InputFloat64(a, b, c) => Self::InputFloat64(*a, *b, c.clone()),
            Self::InvalidInputUnsigned32 => Self::InvalidInputUnsigned32,
            Self::InvalidInputFloat64 => Self::InvalidInputFloat64,
            Self::InputPercentage(a, b, c) => Self::InputPercentage(*a, *b, c.clone()),
            Self::InvalidInputPercentage => Self::InvalidInputPercentage,
            Self::InputRadio(a, b) => Self::InputRadio(*a, b.clone()),
            Self::InputHostNetworkGroup(a, b) => Self::InputHostNetworkGroup(*a, b.clone()),
            Self::InputMultipleSelect(a, b, c) => {
                Self::InputMultipleSelect(*a, b.clone(), c.clone())
            }
            Self::InputSingleSelect(a, b, c) => Self::InputSingleSelect(*a, b.clone(), c.clone()),
            Self::InputVecSelect(a, b, c) => Self::InputVecSelect(*a, *b, c.clone()),
            Self::InputTagGroup(a, b) => Self::InputTagGroup(*a, b.clone()),
            Self::UserInputHostNetworkGroup(a) => Self::UserInputHostNetworkGroup(*a),
            Self::RightHostNetworkGroup(a, b) => Self::RightHostNetworkGroup(*a, b.clone()),
            Self::WrongHostNetworkGroup(a) => Self::WrongHostNetworkGroup(*a),
            Self::ClickCheckBox(a) => Self::ClickCheckBox(a.clone()),
            Self::InputNicName(a, b, c, d) => Self::InputNicName(*a, *b, c.clone(), d.clone()),
            Self::InputNicInterface(a, b, c, d) => {
                Self::InputNicInterface(*a, *b, c.clone(), d.clone())
            }
            Self::InputNicGateway(a, b, c, d) => {
                Self::InputNicGateway(*a, *b, c.clone(), d.clone())
            }
            Self::InputNicAdd(a, b, c) => Self::InputNicAdd(*a, *b, c.clone()),
            Self::InputNicDelete(a, b, c) => Self::InputNicDelete(*a, *b, c.clone()),
            Self::InputGroupAdd(a, b, c) => Self::InputGroupAdd(*a, b.clone(), c.clone()),
            Self::InputGroupDelete(a, b, c, d) => {
                Self::InputGroupDelete(*a, *b, c.clone(), d.clone())
            }
            Self::InputComparisonValueKind(a, b) => Self::InputComparisonValueKind(*a, b.clone()),
            Self::InputComparisonComparisionKind(a, b) => {
                Self::InputComparisonComparisionKind(*a, b.clone())
            }
            Self::InputComparisonValue(a, b, c, d) => {
                Self::InputComparisonValue(*a, *b, c.clone(), d.clone())
            }
            Self::InvalidInputComparisonValue => Self::InvalidInputComparisonValue,
            Self::ChooseFile(a, _, c) => Self::ChooseFile(*a, Vec::new(), c.clone()),
            Self::FileLoaded(a, b) => Self::FileLoaded(a.clone(), b.clone()),
            Self::FailLoadFile => Self::FailLoadFile,
            Self::InputError => Self::InputError,
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Escape, Self::Escape)
            | (Self::Save, Self::Save)
            | (Self::TrySave, Self::TrySave)
            | (Self::InvalidInputUnsigned32, Self::InvalidInputUnsigned32)
            | (Self::InvalidInputFloat64, Self::InvalidInputFloat64)
            | (Self::InvalidInputComparisonValue, Self::InvalidInputComparisonValue)
            | (Self::FailLoadFile, Self::FailLoadFile)
            | (Self::InputError, Self::InputError) => true,
            (Self::UserInputHostNetworkGroup(s1), Self::UserInputHostNetworkGroup(o1))
            | (Self::WrongHostNetworkGroup(s1), Self::WrongHostNetworkGroup(o1)) => s1 == o1,
            (Self::InputText(s1, s2, s3), Self::InputText(o1, o2, o3))
            | (Self::InputPassword(s1, s2, s3), Self::InputPassword(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::InputConfirmPassword(s1, s2), Self::InputConfirmPassword(o1, o2)) => {
                s1 == o1 && s2 == o2
            }
            (Self::InputUnsigned32(s1, s2, s3), Self::InputUnsigned32(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::InputFloat64(s1, s2, s3), Self::InputFloat64(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::InputPercentage(s1, s2, s3), Self::InputPercentage(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::InputRadio(s1, s2), Self::InputRadio(o1, o2))
            | (Self::InputTagGroup(s1, s2), Self::InputTagGroup(o1, o2))
            | (Self::InputComparisonValueKind(s1, s2), Self::InputComparisonValueKind(o1, o2))
            | (
                Self::InputComparisonComparisionKind(s1, s2),
                Self::InputComparisonComparisionKind(o1, o2),
            ) => s1 == o1 && s2 == o2,
            (
                Self::InputComparisonValue(s1, s2, s3, s4),
                Self::InputComparisonValue(o1, o2, o3, o4),
            ) => s1 == o1 && s2 == o2 && s3 == o3 && s4 == o4,
            (Self::InputHostNetworkGroup(s1, s2), Self::InputHostNetworkGroup(o1, o2))
            | (Self::RightHostNetworkGroup(s1, s2), Self::RightHostNetworkGroup(o1, o2)) => {
                s1 == o1 && s2 == o2
            }
            (Self::InputSingleSelect(s1, s2, s3), Self::InputSingleSelect(o1, o2, o3))
            | (Self::InputMultipleSelect(s1, s2, s3), Self::InputMultipleSelect(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::InputVecSelect(s1, s2, s3), Self::InputVecSelect(o1, o2, o3))
            | (Self::InputNicAdd(s1, s2, s3), Self::InputNicAdd(o1, o2, o3))
            | (Self::InputNicDelete(s1, s2, s3), Self::InputNicDelete(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::ClickCheckBox(s), Self::ClickCheckBox(o)) => s == o,
            (Self::InputNicName(s1, s2, s3, s4), Self::InputNicName(o1, o2, o3, o4))
            | (Self::InputNicGateway(s1, s2, s3, s4), Self::InputNicGateway(o1, o2, o3, o4))
            | (Self::InputNicInterface(s1, s2, s3, s4), Self::InputNicInterface(o1, o2, o3, o4)) => {
                s1 == o1 && s2 == o2 && s3 == o3 && s4 == o4
            }
            (Self::FileLoaded(s1, s2), Self::FileLoaded(o1, o2)) => s1 == o1 && s2 == o2,
            (Self::InputGroupAdd(s1, s2, s3), Self::InputGroupAdd(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::InputGroupDelete(s1, s2, s3, s4), Self::InputGroupDelete(o1, o2, o3, o4)) => {
                s1 == o1 && s2 == o2 && s3 == o3 && s4 == o4
            }
            (Self::ChooseFile(_, _, _), Self::ChooseFile(_, _, _)) | (_, _) => false,
        }
    }
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub txt: Texts,
    pub language: Language,

    pub data: Rc<HashMap<String, ListItem>>,
    #[prop_or(None)]
    pub input_id: Option<AttrValue>, // Some: Edit, None: Add
    #[prop_or(None)]
    pub input_second_id: Option<InputSecondId>,

    pub title: &'static str,
    pub width: u32,
    pub height: u32,
    pub input_type: Vec<Rc<InputType>>,
    pub input_data: Vec<Rc<RefCell<InputItem>>>,
    #[prop_or(None)]
    pub input_data_tag: Option<Rc<RefCell<InputTag>>>,

    pub action_message: T::Message,
    pub escape_message: T::Message,
    #[prop_or(None)]
    pub extra_messages: Option<HashMap<MessageType, T::Message>>,

    #[prop_or(true)]
    pub enable_pop_up: bool,

    #[prop_or(None)]
    pub example_message: Option<String>,
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
            radio_buffer: HashMap::new(),
            host_network_buffer: HashMap::new(),
            select_searchable_buffer: HashMap::new(),
            vec_select_buffer: HashMap::new(),
            tag_buffer: HashMap::new(),
            comparison_value_kind_buffer: HashMap::new(),
            comparison_value_cmp_buffer: HashMap::new(),
            comparison_value_buffer: HashMap::new(),

            confirm_password: HashMap::new(),
            unique_msg: HashSet::new(),
            required_msg: HashSet::new(),

            verification: HashMap::new(),
            verification_nic: HashMap::new(),
            verification_host_network: HashMap::new(),
            verify_host_network_group: false,

            file_data_id: None,
            file_input_data: None,
            file_name: None,
            file_content: None,
            file_reader: None,

            rerender_serial_host_network: 0,

            phantom: PhantomData,
        };
        Self::prepare_nic(ctx);
        if ctx.props().input_id.is_none() {
            s.prepare_default(ctx);
        }
        s.prepare_buffer(ctx);
        s
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        Self::prepare_nic(ctx);
        self.prepare_default(ctx);

        ctx.props()
            .input_data
            .iter()
            .enumerate()
            .zip(ctx.props().input_type.iter())
            .for_each(|((index, input_data), input_type)| {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    if let InputItem::Tag(_) = *item {
                        if let (InputType::Tag(_, updated), Some(buffer)) =
                            (&**input_type, self.tag_buffer.get(&(index + 1)))
                        // HIGHLIGHT: Since Tag is always on the first layer, don't need to check recursively
                        {
                            let reverse = updated
                                .iter()
                                .map(|(k, v)| (v.clone(), k.clone()))
                                .collect::<HashMap<String, String>>();

                            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                                let deleted = buffer
                                    .delete
                                    .as_ref()
                                    .map_or_else(String::new, Clone::clone);
                                buffer.old.remove(&deleted);
                                buffer.delete = None;
                                *item = InputItem::Tag((*buffer).clone());
                            }
                            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                                if let Some(new) = buffer.new.as_ref() {
                                    if let Some(key) = reverse.get(new) {
                                        buffer.old.insert(key.clone());
                                        buffer.new = None;
                                    }
                                }
                                // no need to verify the value has been actually edited right.
                                buffer.edit = None;
                                *item = InputItem::Tag((*buffer).clone());
                            }
                        }
                    }
                }
            });

        true
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Escape => {
                if let Some(parent) = ctx.link().get_parent() {
                    parent
                        .clone()
                        .downcast::<T>()
                        .send_message(ctx.props().escape_message.clone());
                }
            }
            Message::TrySave => {
                self.increase_rerender_serial();
                self.reset_veri_host_network(ctx);
                self.verify_host_network_group = !self.verification_host_network.is_empty();

                if !self.verify_host_network_group {
                    ctx.link().send_message(Message::Save);
                    return false;
                }
            }
            Message::WrongHostNetworkGroup(id) => {
                self.verification_host_network.insert(id, Some(false));
                self.clear_required_msg(id, false);
                self.decide_required_all(ctx);
                self.decide_unique_all(ctx);
            }
            Message::RightHostNetworkGroup(id, input_data) => {
                self.verification_host_network.insert(id, Some(true));
                self.input_host_network_group(id, &input_data);
                if !self
                    .verification_host_network
                    .values()
                    .any(|v| (*v).map_or(true, |v| !v))
                {
                    // HIGHLIGHT: Only if all of the HostNetworkGroup items are valid, proceed to save the user input
                    self.verify_host_network_group = false;
                    ctx.link().send_message(Message::Save);
                }
            }
            Message::Save => {
                let required = self.decide_required_all(ctx);
                let unique = self.decide_unique_all(ctx);
                let verify = self.verify(ctx);

                if !required && !unique && verify {
                    Self::trim_nic(ctx);
                    if let Some(parent) = ctx.link().get_parent() {
                        parent
                            .clone()
                            .downcast::<T>()
                            .send_message(ctx.props().action_message.clone());
                    }
                }
            }
            Message::InputText(id, txt, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Text(txt.clone());
                }
                self.clear_required_msg(id, txt.is_empty());
                self.unique_msg.remove(&id);
            }
            Message::InputPassword(id, txt, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Password(txt.clone());
                }
                self.clear_required_msg(id, txt.is_empty());
            }
            Message::InputConfirmPassword(id, txt) => {
                self.confirm_password.insert(id, txt.clone());
                self.clear_required_msg(id, txt.is_empty());
            }
            Message::InputUnsigned32(id, value, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Unsigned32(Some(value));
                }
                self.clear_required_msg(id, false);
                self.unique_msg.remove(&id);
            }
            Message::InputFloat64(id, value, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Float64(Some(value));
                }
                self.clear_required_msg(id, false);
                self.unique_msg.remove(&id);
            }
            Message::InvalidInputUnsigned32
            | Message::InvalidInputFloat64
            | Message::InvalidInputPercentage
            | Message::InvalidInputComparisonValue => return false,
            Message::InputPercentage(id, value, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Percentage(Some(value));
                }
                self.clear_required_msg(id, false);
                self.unique_msg.remove(&id);
            }
            Message::InputRadio(id, input_data) => {
                if let Some(buffer) = self.radio_buffer.get(&id) {
                    let empty = if let Ok(buffer) = buffer.try_borrow_mut() {
                        if let Ok(mut item) = input_data.try_borrow_mut() {
                            *item = InputItem::Text(buffer.clone());
                        }
                        buffer.is_empty()
                    } else {
                        false
                    };
                    self.clear_required_msg(id, empty);
                }
            }
            Message::InputHostNetworkGroup(id, input_data) => {
                self.input_host_network_group(id, &input_data);
            }
            Message::UserInputHostNetworkGroup(id) => {
                self.clear_required_msg(id, false);
            }
            Message::InputMultipleSelect(id, input_data, list) => {
                if let Some(buffer) = self.select_searchable_buffer.get(&id) {
                    let empty = if let Ok(buffer) = buffer.try_borrow() {
                        if let Some(buffer) = buffer.as_ref() {
                            if let Ok(mut item) = input_data.try_borrow_mut() {
                                *item = InputItem::SelectMultiple(buffer.clone());
                            }
                            buffer.is_empty()
                        } else if let Ok(mut item) = input_data.try_borrow_mut() {
                            let list = list
                                .iter()
                                .map(|item| item.0.clone())
                                .collect::<HashSet<String>>();
                            *item = InputItem::SelectMultiple(list);
                            false
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    self.clear_required_msg(id, empty);
                }
            }
            Message::InputSingleSelect(id, input_data, _) => {
                if let Some(buffer) = self.select_searchable_buffer.get(&id) {
                    let empty = if let Ok(buffer) = buffer.try_borrow() {
                        if let Some(buffer) = buffer.as_ref() {
                            let selected = buffer.iter().map(Clone::clone).next();
                            if let Ok(mut item) = input_data.try_borrow_mut() {
                                *item = InputItem::SelectSingle(selected);
                            }
                            buffer.is_empty()
                        } else if let Ok(mut item) = input_data.try_borrow_mut() {
                            *item = InputItem::SelectSingle(None);
                            false
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    self.clear_required_msg(id, empty);
                }
            }
            Message::InputVecSelect(data_id, col_id, input_data) => {
                if let (Some(buffer), Ok(mut input_data)) = (
                    self.vec_select_buffer.get(&(data_id)),
                    input_data.try_borrow_mut(),
                ) {
                    if let (Some(buffer), InputItem::VecSelect(input_data)) =
                        (buffer.get(col_id), &mut *input_data)
                    {
                        if let (Ok(buffer), Some(input_data)) =
                            (buffer.try_borrow(), input_data.get_mut(col_id))
                        {
                            if let Some(buffer) = &*buffer {
                                *input_data = buffer.clone();
                                if !buffer.is_empty() {
                                    self.required_msg.remove(&data_id);
                                }
                            }
                        }
                    }
                }
            }
            Message::InputTagGroup(id, input_data) => {
                let (new, edit, delete) = if let Some(buffer) = self.tag_buffer.get(&id) {
                    let (empty, new, edit, delete) = if let Ok(buffer) = buffer.try_borrow_mut() {
                        if let Ok(mut item) = input_data.try_borrow_mut() {
                            *item = InputItem::Tag(buffer.clone());
                        }
                        (
                            buffer.old.is_empty(),
                            buffer.new.clone(),
                            buffer.edit.clone(),
                            buffer.delete.clone(),
                        )
                    } else {
                        (false, None, None, None)
                    };
                    self.clear_required_msg(id, empty);

                    (new, edit, delete)
                } else {
                    (None, None, None)
                };

                if let Some(new) = new {
                    if !new.is_empty() {
                        if let Some(data_tag) = ctx.props().input_data_tag.as_ref() {
                            if let Ok(mut data_tag) = data_tag.try_borrow_mut() {
                                data_tag.new = Some(new);
                            }
                        }
                        let msg = ctx
                            .props()
                            .extra_messages
                            .as_ref()
                            .and_then(|m| m.get(&MessageType::AddTag).cloned());
                        if let (Some(parent), Some(msg)) = (ctx.link().get_parent(), msg) {
                            parent.clone().downcast::<T>().send_message(msg);
                        }
                    }
                }
                if let Some(edit) = edit {
                    if !edit.0.is_empty() && !edit.1.is_empty() {
                        if let Some(data_tag) = ctx.props().input_data_tag.as_ref() {
                            if let Ok(mut data_tag) = data_tag.try_borrow_mut() {
                                data_tag.edit = Some(edit);
                            }
                        }
                        let msg = ctx
                            .props()
                            .extra_messages
                            .as_ref()
                            .and_then(|m| m.get(&MessageType::EditTag).cloned());
                        if let (Some(parent), Some(msg)) = (ctx.link().get_parent(), msg) {
                            parent.clone().downcast::<T>().send_message(msg);
                        }
                    }
                }
                if let Some(delete) = delete {
                    if !delete.is_empty() {
                        if let Some(data_tag) = ctx.props().input_data_tag.as_ref() {
                            if let Ok(mut data_tag) = data_tag.try_borrow_mut() {
                                data_tag.delete = Some(delete);
                            }
                        }
                        let msg = ctx
                            .props()
                            .extra_messages
                            .as_ref()
                            .and_then(|m| m.get(&MessageType::DeleteTag).cloned());
                        if let (Some(parent), Some(msg)) = (ctx.link().get_parent(), msg) {
                            parent.clone().downcast::<T>().send_message(msg);
                        }
                    }
                }

                return false; // HIGHLIGHT: DO NOT return true
            }
            Message::ClickCheckBox(item) => {
                self.propagate_checkbox(ctx, &item);
            }
            Message::InputNicName(data_id, nic_id, name, input_data) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(data) = &mut *input_data {
                        if let Some(nic) = data.get_mut(nic_id) {
                            nic.name = name.clone();
                        }
                    }
                }
                self.clear_required_msg(data_id, name.is_empty());
                self.remove_verification_nic(data_id * MAX_PER_LAYER + nic_id);
            }
            Message::InputNicInterface(data_id, nic_id, interface, input_data) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(data) = &mut *input_data {
                        if let Some(nic) = data.get_mut(nic_id) {
                            nic.interface = interface.clone();
                        }
                    }
                }
                self.clear_required_msg(data_id, interface.is_empty());
                self.remove_verification_nic(data_id * MAX_PER_LAYER + nic_id);
            }
            Message::InputNicGateway(data_id, nic_id, gateway, input_data) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(data) = &mut *input_data {
                        if let Some(nic) = data.get_mut(nic_id) {
                            nic.gateway = gateway.clone();
                        }
                    }
                }
                self.clear_required_msg(data_id, gateway.is_empty());
                self.remove_verification_nic(data_id * MAX_PER_LAYER + nic_id);
            }
            Message::InputNicAdd(data_id, nic_id, input_data) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(data) = &mut *input_data {
                        data.push(InputNic::default());
                    }
                }
                self.remove_verification_nic(data_id * MAX_PER_LAYER + nic_id);
            }
            Message::InputNicDelete(data_id, nic_id, input_data) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(data) = &mut *input_data {
                        data.remove(nic_id);
                        if data.is_empty() {
                            data.push(InputNic::default());
                        }
                    }
                }
                self.remove_verification_nic(data_id * MAX_PER_LAYER + nic_id);
            }
            Message::InputGroupAdd(sub_base_index, input_data, default) => {
                if let (Ok(mut input_data), Some(InputItem::Group(default))) =
                    (input_data.try_borrow_mut(), default)
                {
                    let Some(default) = default.first() else {
                        return false;
                    };
                    if let (InputItem::Group(data), Some(copy_default)) =
                        (&mut *input_data, Self::copy_default(default))
                    {
                        data.push(copy_default);
                        if let Some(d) = data.last() {
                            for (col, d) in d.iter().enumerate() {
                                if let Ok(d) = d.try_borrow() {
                                    if let InputItem::SelectSingle(copied_default) = &*d {
                                        let mut buf = HashSet::new();
                                        if let Some(copied_default) = copied_default {
                                            buf.insert(copied_default.clone());
                                        }
                                        self.select_searchable_buffer.insert(
                                            col + (data.len() - 1 + sub_base_index) * MAX_PER_LAYER,
                                            Rc::new(RefCell::new(Some(buf))),
                                        );
                                    } else if let InputItem::Comparison(_) = &*d {
                                        self.comparison_value_kind_buffer.insert(
                                            col + (data.len() - 1 + sub_base_index) * MAX_PER_LAYER,
                                            Rc::new(RefCell::new(Some(HashSet::new()))),
                                        );
                                        self.comparison_value_cmp_buffer.insert(
                                            col + (data.len() - 1 + sub_base_index) * MAX_PER_LAYER,
                                            Rc::new(RefCell::new(Some(HashSet::new()))),
                                        );
                                        self.comparison_value_buffer.insert(
                                            col + (data.len() - 1 + sub_base_index) * MAX_PER_LAYER,
                                            (
                                                Rc::new(RefCell::new(None)),
                                                Rc::new(RefCell::new(None)),
                                            ),
                                        );
                                    } else if let InputItem::VecSelect(copied_default) = &*d {
                                        self.vec_select_buffer.insert(
                                            col + (data.len() - 1 + sub_base_index) * MAX_PER_LAYER,
                                            copied_default
                                                .iter()
                                                .map(|d| Rc::new(RefCell::new(Some(d.clone()))))
                                                .collect::<Vec<_>>(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Message::InputGroupDelete(sub_base_index, row_index, input_data, default) => {
                let empty = if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Group(data) = &mut *input_data {
                        if let Some(d) = data.get(row_index) {
                            for (col, d) in d.iter().enumerate() {
                                if let Ok(d) = d.try_borrow() {
                                    if let InputItem::SelectSingle(_) = &*d {
                                        rearrange_buffer(
                                            &mut self.select_searchable_buffer,
                                            sub_base_index,
                                            col,
                                            row_index,
                                            data.len(),
                                        );
                                    } else if let InputItem::Comparison(_) = &*d {
                                        rearrange_buffer(
                                            &mut self.comparison_value_kind_buffer,
                                            sub_base_index,
                                            col,
                                            row_index,
                                            data.len(),
                                        );
                                        rearrange_buffer(
                                            &mut self.comparison_value_cmp_buffer,
                                            sub_base_index,
                                            col,
                                            row_index,
                                            data.len(),
                                        );
                                        rearrange_buffer(
                                            &mut self.comparison_value_buffer,
                                            sub_base_index,
                                            col,
                                            row_index,
                                            data.len(),
                                        );
                                    } else if let InputItem::VecSelect(_) = &*d {
                                        rearrange_buffer(
                                            &mut self.vec_select_buffer,
                                            sub_base_index,
                                            col,
                                            row_index,
                                            data.len(),
                                        );
                                    }
                                }
                                self.required_msg
                                    .remove(&(col + (row_index + sub_base_index) * MAX_PER_LAYER));
                                for r in row_index + 1..data.len() {
                                    if self
                                        .required_msg
                                        .remove(&(col + (r + sub_base_index) * MAX_PER_LAYER))
                                    {
                                        self.required_msg
                                            .insert(col + (r - 1 + sub_base_index) * MAX_PER_LAYER);
                                    }
                                }
                            }
                        }
                        data.remove(row_index);
                        data.is_empty()
                    } else {
                        false
                    }
                } else {
                    false
                };
                if empty {
                    ctx.link().send_message(Message::InputGroupAdd(
                        sub_base_index,
                        input_data,
                        default,
                    ));
                }
            }
            Message::InputComparisonValueKind(data_id, input_data) => {
                self.clear_comparison_value(data_id, &input_data);
            }
            Message::InputComparisonComparisionKind(data_id, input_data) => {
                self.input_comparison_comparison_kind(data_id, &input_data);
            }
            Message::InputComparisonValue(data_id, value_index, value, input_data) => {
                self.input_comparison_value(data_id, value_index, &value, &input_data);
            }
            Message::ChooseFile(data_id, files, input_data) => {
                for file in files {
                    let file_name = file.name();
                    let task = {
                        let file_name = file_name.clone();
                        let link = ctx.link().clone();

                        read_as_bytes(&file, move |res| {
                            if let Ok(res) = res {
                                link.send_message(Message::FileLoaded(file_name, res));
                            } else {
                                link.send_message(Message::FailLoadFile);
                            }
                        })
                    };

                    self.file_data_id = Some(data_id);
                    self.file_input_data = Some(Rc::clone(&input_data));
                    self.file_name = Some(file_name);
                    self.file_reader = Some(task);
                }
            }
            Message::FileLoaded(file_name, file) => {
                if let Some(input_data) = self.file_input_data.as_ref() {
                    if let Ok(mut item) = input_data.try_borrow_mut() {
                        let content = BASE64.encode(&file);
                        *item = InputItem::File(file_name, content);
                    }
                }
                self.file_reader = None;
            }
            Message::FailLoadFile => {
                // TODO: show a message for users
                return false;
            }
            Message::InputError => {} // TODO: issue #5 in case of InputError
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = format!(
            "width: {}px; max-height: {}px;",
            ctx.props().width,
            ctx.props().height
        );
        let style_input = format!("height: {}px;", ctx.props().height - 166);
        let txt = ctx.props().txt.txt.clone();
        let onclick_escape = ctx.link().callback(|_| Message::Escape);
        let onclick_save = ctx.link().callback(|_| Message::TrySave);
        if ctx.props().enable_pop_up {
            html! {
                <div class="input-outer">
                    <div class="input-inner" style={style}> // padding-top: 20, padding-bottom: 32
                        <table class="input-head"> // margin-bottom: 24
                            <tr>
                                <td class="input-title"> // height: 24
                                    { text!(txt, ctx.props().language, &ctx.props().title) }
                                </td>
                                <td class="input-close-x" onclick={onclick_escape.clone()}>
                                </td>
                            </tr>
                            <tr>
                                <td colspan="2" class="input-head-space"> // height: 20
                                </td>
                            </tr>
                        </table>
                        <div class="input-contents" style={style_input}>
                            { self.view_input(ctx) }
                        </div>
                        <div class="input-cancel-save"> // margin-top: 16, height: 30
                            <div class="input-cancel" onclick={onclick_escape}>
                                { text!(txt, ctx.props().language, "Cancel") }
                            </div>
                            <div class="input-save" onclick={onclick_save}>
                                { text!(txt, ctx.props().language, "Save") }
                            </div>
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {
                    { self.view_input(ctx) }
            }
        }
    }
}

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    fn view_input(&self, ctx: &Context<Self>) -> Html {
        html! {
            for ctx.props().input_data.iter().enumerate().zip(ctx.props().input_type.iter()).map(|((index , input_data), input_type)| {
                match &**input_type {
                    InputType::Text(ess, length, width) => self.view_text(ctx, ess, *length, *width, input_data, index, 1 , index == 0, false),
                    InputType::Password(ess,width) => self.view_password(ctx, ess, *width, input_data, index, 1, index == 0),
                    InputType::Radio(ess, options) => self.view_radio(ctx, ess, options, input_data, index, 1),
                    InputType::HostNetworkGroup(ess, kind, num, width) => self.view_host_network_group(ctx, ess, *kind, *num, *width, input_data, index, 1),
                    InputType::SelectMultiple(ess, list, nics, _, _) => self.view_select_nic_or(ctx, list, *nics, ess, input_data, index, 1, 0),
                    InputType::SelectSingle(ess, list, width) => self.view_select_searchable(ctx, false, ess, *width, list, input_data, index, 1, 0, false),
                    InputType::Tag(ess, list) => self.view_tag_group(ctx, ess, list, input_data, index, 1),
                    InputType::VecSelect(ess, ess_list, last_multi, list, width, width_list, max_width_list, max_height_list) => self.view_vec_select(ctx, ess, ess_list, *last_multi, *width, width_list, max_width_list, max_height_list, list, input_data, index, 1, false),
                    InputType::Unsigned32(ess, min, max, width) => self.view_unsigned_32(ctx, ess, *min, *max, *width, input_data, index, 1, index == 0, false),
                    InputType::Float64(ess, step, width) => self.view_float_64(ctx, ess, *step, *width, input_data, index, 1, index == 0, false),
                    InputType::Percentage(ess, min, max, decimals, width) => self.view_percentage(ctx, ess, *min, *max, *decimals, *width, input_data, index, 1, index == 0),
                    InputType::CheckBox(ess, always, children) => {
                        let both = ctx.props().input_type.get(index + 1).map_or(Some(false),|next| {
                            if let InputType::CheckBox(_, _, _) = &**next {
                                Some(false)
                            } else {
                                Some(true)
                            }
                        });
                        self.view_checkbox(ctx, ess, *always, children, input_data, index, 1, both, 1)
                    }
                    InputType::Nic(ess) => self.view_nic(ctx, ess, input_data, index, 1),
                    InputType::File(ess) => self.view_file(ctx, ess, input_data, index, 1),
                    InputType::Group(ess, one_row, widths, group_type) => self.view_group(ctx, ess, *one_row, widths, group_type, input_data, index, 1),
                    InputType::Comparison(ess) => self.view_comparison(ctx, ess, input_data, index, 1, false),
                }
            })
        }
    }

    fn prepare_nic(ctx: &Context<Self>) {
        for input_data in &ctx.props().input_data {
            if let Ok(mut input_data) = input_data.try_borrow_mut() {
                if let InputItem::Nic(input_data) = &mut *input_data {
                    if input_data.is_empty() {
                        input_data.push(InputNic::default());
                    }
                }
            }
        }
    }

    fn clear_required_msg(&mut self, id: usize, empty: bool) {
        if !empty {
            self.required_msg.remove(&id);
        }
    }

    fn input_host_network_group(&mut self, id: usize, input_data: &Rc<RefCell<InputItem>>) {
        if let Some(buffer) = self.host_network_buffer.get(&id) {
            let empty = if let Ok(buffer) = buffer.try_borrow_mut() {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    let mut n = buffer.clone();
                    sort_hosts(&mut n.hosts);
                    sort_networks(&mut n.networks);
                    n.ranges.sort_unstable();
                    *item = InputItem::HostNetworkGroup(n);
                }
                buffer.is_empty()
            } else {
                false
            };
            self.clear_required_msg(id, empty);
        }
    }

    fn remove_verification_nic(&mut self, id: usize) {
        self.verification_nic.remove(&(id, 0));
        self.verification_nic.remove(&(id, 1));
        self.verification_nic.remove(&(id, 2));
    }

    fn decide_unique_all(&mut self, ctx: &Context<Self>) -> bool {
        // no need to check CheckBox's children because CheckBox and its children don't need to be unique
        let mut unique = Vec::<bool>::new();
        let id = ctx.props().input_second_id.as_ref().map_or_else(
            || ctx.props().input_id.as_deref(),
            |id| match id {
                InputSecondId::Add => None,
                InputSecondId::Edit(i) => Some(i),
            },
        );

        for (index, t) in ctx.props().input_type.iter().enumerate() {
            if let InputType::Text(..) = &(**t) {
                if let Some(data) = ctx.props().input_data.get(index) {
                    if let Ok(input) = data.try_borrow() {
                        if t.unique() {
                            let mut different = true;
                            for (key, item) in ctx.props().data.iter() {
                                if id.as_ref().map_or(true, |id| id != key) {
                                    if let Some(other) = item.columns.get(index) {
                                        if let (
                                            Column::Text(ViewString::Raw(other_value)),
                                            InputItem::Text(value),
                                        ) = (other, &(*input))
                                        {
                                            if other_value == value {
                                                different = false;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            if !different {
                                self.unique_msg.insert(index + 1);
                                unique.push(true);
                            }
                        }
                    }
                }
            }
        }
        !unique.is_empty()
    }
}

fn rearrange_buffer<T>(
    buffer: &mut HashMap<usize, T>,
    base: usize,
    col: usize,
    row: usize,
    len: usize,
) where
    T: Clone,
{
    for index in row + 1..len {
        let Some(item) = buffer.remove(&(
            col + (index + base) * MAX_PER_LAYER
        )) else {
            continue;
        };
        buffer.insert(col + (index - 1 + base) * MAX_PER_LAYER, item.clone());
    }
}
