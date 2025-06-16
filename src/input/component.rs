use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::{cell::RefCell, marker::PhantomData};

use data_encoding::BASE64;
use gloo_file::{
    File,
    callbacks::{FileReader, read_as_bytes},
};
use json_gettext::get_text;
use num_bigint::BigUint;
use yew::{Component, Context, Html, Properties, html, virtual_dom::AttrValue};

use super::{
    FileItem, Float64Item, HostNetworkGroupItem, InputConfig, InputHostNetworkGroup, InputItem,
    InputTag, InputTagGroup, PasswordItem, PercentageItem, SelectMultipleItem, SelectSingleItem,
    TagItem, TextItem, Unsigned8Item, Unsigned32Item, Value as ComparisonValue, cal_index,
    group_item_list_preset,
};
use crate::{
    InputNic, InvalidPasswordKind, MessageType, Rerender, Texts, ViewString,
    language::Language,
    list::{Column, ListItem},
    sort_hosts, sort_networks, text,
};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum InvalidMessage {
    InvalidInput,
    InvalidPassword(InvalidPasswordKind),
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
    BigUint,
    (
        Rc<RefCell<Option<ComparisonValue>>>,
        Rc<RefCell<Option<ComparisonValue>>>,
    ),
>;

type VecSelectBuf = HashMap<BigUint, Vec<Rc<RefCell<Option<HashSet<String>>>>>>;

pub struct Model<T> {
    pub(super) radio_buffer: HashMap<BigUint, Rc<RefCell<String>>>,
    pub(super) host_network_buffer: HashMap<BigUint, Rc<RefCell<InputHostNetworkGroup>>>,
    pub(super) select_searchable_buffer: HashMap<BigUint, Rc<RefCell<Option<HashSet<String>>>>>,
    pub(super) vec_select_buffer: VecSelectBuf,
    pub(super) tag_buffer: HashMap<BigUint, Rc<RefCell<InputTagGroup>>>,
    pub(super) comparison_value_kind_buffer: HashMap<BigUint, Rc<RefCell<Option<HashSet<String>>>>>,
    pub(super) comparison_value_cmp_buffer: HashMap<BigUint, Rc<RefCell<Option<HashSet<String>>>>>,
    pub(super) comparison_value_buffer: CompValueBuf,

    pub(super) confirm_password: HashMap<BigUint, String>,
    pub(super) unique_msg: HashSet<BigUint>,
    pub(super) required_msg: HashSet<BigUint>,

    pub(super) verification: HashMap<BigUint, Verification>,
    pub(super) verification_nic: HashMap<(BigUint, usize), Verification>, // 2nd usize: 0 -> name, 1 -> interface ip, 2 -> gateway ip
    pub(super) verification_host_network: HashMap<BigUint, Option<bool>>, // None means no checking done yet, Some(true) valid Some(false) invalid
    pub(super) verify_host_network_group: bool,

    file_data_id: Option<BigUint>,
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
            file_data_id: self.file_data_id.clone(),
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
    InputText(BigUint, String, Rc<RefCell<InputItem>>),
    InputPassword(BigUint, String, Rc<RefCell<InputItem>>),
    InputConfirmPassword(BigUint, String),
    InputUnsigned32(BigUint, Option<u32>, Rc<RefCell<InputItem>>),
    InputUnsigned8(BigUint, Option<u8>, Rc<RefCell<InputItem>>),
    InputFloat64(BigUint, Option<f64>, Rc<RefCell<InputItem>>),
    InvalidInputUnsigned32,
    InvalidInputUnsigned8,
    InvalidInputFloat64,
    InputPercentage(BigUint, Option<f32>, Rc<RefCell<InputItem>>),
    InvalidInputPercentage,
    InputRadio(BigUint, Rc<RefCell<InputItem>>),
    InputHostNetworkGroup(BigUint, Rc<RefCell<InputItem>>),
    InputMultipleSelect(
        BigUint,
        Rc<RefCell<InputItem>>,
        Rc<Vec<(String, ViewString)>>,
    ),
    InputSingleSelect(
        BigUint,
        Rc<RefCell<InputItem>>,
        Rc<Vec<(String, ViewString)>>,
    ),
    InputVecSelect(
        BigUint, // data_id
        usize,   // col_id
        Rc<RefCell<InputItem>>,
    ),
    InputTagGroup(BigUint, Rc<RefCell<InputItem>>),
    UserInputHostNetworkGroup(BigUint),
    WrongHostNetworkGroup(BigUint),
    RightHostNetworkGroup(BigUint, Rc<RefCell<InputItem>>),
    ClickCheckbox(BigUint, Rc<RefCell<InputItem>>),
    InputNicName(BigUint, usize, String, Rc<RefCell<InputItem>>),
    InputNicInterface(BigUint, usize, String, Rc<RefCell<InputItem>>),
    InputNicGateway(BigUint, usize, String, Rc<RefCell<InputItem>>),
    InputNicAdd(BigUint, usize, Rc<RefCell<InputItem>>),
    InputNicDelete(BigUint, usize, Rc<RefCell<InputItem>>),
    InputGroupAdd(BigUint, Rc<RefCell<InputItem>>, Vec<Rc<InputConfig>>),
    InputGroupDelete(
        BigUint,
        usize,
        Rc<RefCell<InputItem>>,
        Vec<Rc<InputConfig>>,
        bool,
    ), // bool for required
    InputComparisonValueKind(BigUint, Rc<RefCell<InputItem>>),
    InputComparisonComparisionKind(BigUint, Rc<RefCell<InputItem>>),
    InputComparisonValue(BigUint, usize, ComparisonValue, Rc<RefCell<InputItem>>),
    InvalidInputComparisonValue,
    ChooseFile(BigUint, Vec<File>, Rc<RefCell<InputItem>>),
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
            Self::InputText(a, b, c) => Self::InputText(a.clone(), b.clone(), c.clone()),
            Self::InputPassword(a, b, c) => Self::InputPassword(a.clone(), b.clone(), c.clone()),
            Self::InputConfirmPassword(a, b) => Self::InputConfirmPassword(a.clone(), b.clone()),
            Self::InputUnsigned32(a, b, c) => Self::InputUnsigned32(a.clone(), *b, c.clone()),
            Self::InputUnsigned8(a, b, c) => Self::InputUnsigned8(a.clone(), *b, c.clone()),
            Self::InputFloat64(a, b, c) => Self::InputFloat64(a.clone(), *b, c.clone()),
            Self::InvalidInputUnsigned32 => Self::InvalidInputUnsigned32,
            Self::InvalidInputUnsigned8 => Self::InvalidInputUnsigned8,
            Self::InvalidInputFloat64 => Self::InvalidInputFloat64,
            Self::InputPercentage(a, b, c) => Self::InputPercentage(a.clone(), *b, c.clone()),
            Self::InvalidInputPercentage => Self::InvalidInputPercentage,
            Self::InputRadio(a, b) => Self::InputRadio(a.clone(), b.clone()),
            Self::InputHostNetworkGroup(a, b) => Self::InputHostNetworkGroup(a.clone(), b.clone()),
            Self::InputMultipleSelect(a, b, c) => {
                Self::InputMultipleSelect(a.clone(), b.clone(), c.clone())
            }
            Self::InputSingleSelect(a, b, c) => {
                Self::InputSingleSelect(a.clone(), b.clone(), c.clone())
            }
            Self::InputVecSelect(a, b, c) => Self::InputVecSelect(a.clone(), *b, c.clone()),
            Self::InputTagGroup(a, b) => Self::InputTagGroup(a.clone(), b.clone()),
            Self::UserInputHostNetworkGroup(a) => Self::UserInputHostNetworkGroup(a.clone()),
            Self::RightHostNetworkGroup(a, b) => Self::RightHostNetworkGroup(a.clone(), b.clone()),
            Self::WrongHostNetworkGroup(a) => Self::WrongHostNetworkGroup(a.clone()),
            Self::ClickCheckbox(a, b) => Self::ClickCheckbox(a.clone(), b.clone()),
            Self::InputNicName(a, b, c, d) => {
                Self::InputNicName(a.clone(), *b, c.clone(), d.clone())
            }
            Self::InputNicInterface(a, b, c, d) => {
                Self::InputNicInterface(a.clone(), *b, c.clone(), d.clone())
            }
            Self::InputNicGateway(a, b, c, d) => {
                Self::InputNicGateway(a.clone(), *b, c.clone(), d.clone())
            }
            Self::InputNicAdd(a, b, c) => Self::InputNicAdd(a.clone(), *b, c.clone()),
            Self::InputNicDelete(a, b, c) => Self::InputNicDelete(a.clone(), *b, c.clone()),
            Self::InputGroupAdd(a, b, c) => Self::InputGroupAdd(a.clone(), b.clone(), c.clone()),
            Self::InputGroupDelete(a, b, c, d, required) => {
                Self::InputGroupDelete(a.clone(), *b, c.clone(), d.clone(), *required)
            }
            Self::InputComparisonValueKind(a, b) => {
                Self::InputComparisonValueKind(a.clone(), b.clone())
            }
            Self::InputComparisonComparisionKind(a, b) => {
                Self::InputComparisonComparisionKind(a.clone(), b.clone())
            }
            Self::InputComparisonValue(a, b, c, d) => {
                Self::InputComparisonValue(a.clone(), *b, c.clone(), d.clone())
            }
            Self::InvalidInputComparisonValue => Self::InvalidInputComparisonValue,
            Self::ChooseFile(a, _, c) => Self::ChooseFile(a.clone(), Vec::new(), c.clone()),
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
            | (Self::InvalidInputUnsigned8, Self::InvalidInputUnsigned8)
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
            (Self::InputUnsigned8(s1, s2, s3), Self::InputUnsigned8(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::InputFloat64(s1, s2, s3), Self::InputFloat64(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::InputPercentage(s1, s2, s3), Self::InputPercentage(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (Self::ClickCheckbox(s1, s2), Self::ClickCheckbox(o1, o2))
            | (Self::InputRadio(s1, s2), Self::InputRadio(o1, o2))
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
            (Self::InputNicName(s1, s2, s3, s4), Self::InputNicName(o1, o2, o3, o4))
            | (Self::InputNicGateway(s1, s2, s3, s4), Self::InputNicGateway(o1, o2, o3, o4))
            | (Self::InputNicInterface(s1, s2, s3, s4), Self::InputNicInterface(o1, o2, o3, o4)) => {
                s1 == o1 && s2 == o2 && s3 == o3 && s4 == o4
            }
            (Self::FileLoaded(s1, s2), Self::FileLoaded(o1, o2)) => s1 == o1 && s2 == o2,
            (Self::InputGroupAdd(s1, s2, s3), Self::InputGroupAdd(o1, o2, o3)) => {
                s1 == o1 && s2 == o2 && s3 == o3
            }
            (
                Self::InputGroupDelete(s1, s2, s3, s4, s5),
                Self::InputGroupDelete(o1, o2, o3, o4, o5),
            ) => s1 == o1 && s2 == o2 && s3 == o3 && s4 == o4 && s5 == o5,
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
    pub input_conf: Vec<Rc<InputConfig>>,
    pub input_data: Vec<Rc<RefCell<InputItem>>>,
    #[prop_or(None)]
    pub input_data_tag: Option<Rc<RefCell<InputTag>>>,

    pub action_message: T::Message,
    pub escape_message: T::Message,
    #[prop_or(None)]
    pub extra_messages: Option<HashMap<MessageType, T::Message>>,

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
            s.prepare_preset(ctx);
        }
        s.prepare_buffer(ctx);
        s
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        Self::prepare_nic(ctx);
        self.prepare_preset(ctx);

        ctx.props()
            .input_data
            .iter()
            .enumerate()
            .zip(ctx.props().input_conf.iter())
            .for_each(|((index, input_data), input_conf)| {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    if let InputItem::Tag(_) = *item {
                        if let (InputConfig::Tag(config), Some(buffer)) =
                            (&**input_conf, self.tag_buffer.get(&(BigUint::from(index))))
                        // HIGHLIGHT: Since Tag is always on the first layer, don't need to check recursively
                        {
                            let reverse = config
                                .name_map // This is the updated one.
                                .iter()
                                .map(|(k, v)| (v.clone(), k.clone()))
                                .collect::<HashMap<String, String>>();

                            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                                let deleted = buffer.delete.clone();
                                if let Some(deleted) = deleted {
                                    buffer.old.remove(&deleted);
                                }
                                buffer.delete = None;
                                *item = InputItem::Tag(TagItem::new((*buffer).clone()));
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
                                *item = InputItem::Tag(TagItem::new((*buffer).clone()));
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
                self.verification_host_network
                    .insert(id.clone(), Some(false));
                self.remove_required_msg(&id, false);
                self.decide_required_all(ctx);
                self.decide_unique_all(ctx);
            }
            Message::RightHostNetworkGroup(id, input_data) => {
                self.verification_host_network
                    .insert(id.clone(), Some(true));
                self.input_host_network_group(&id, &input_data);
                if !self
                    .verification_host_network
                    .values()
                    .any(|v| (*v).is_none_or(|v| !v))
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
                    *item = InputItem::Text(TextItem::new(txt.clone()));
                }
                self.remove_required_msg(&id, txt.is_empty());
                self.unique_msg.remove(&id);
            }
            Message::InputPassword(id, txt, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Password(PasswordItem::new(txt.clone()));
                }
                self.remove_required_msg(&id, txt.is_empty());
            }
            Message::InputConfirmPassword(id, txt) => {
                self.confirm_password.insert(id.clone(), txt.clone());
                self.remove_required_msg(&id, txt.is_empty());
            }
            Message::InputUnsigned32(id, value, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Unsigned32(Unsigned32Item::new(value));
                }
                self.remove_required_msg(&id, false);
                self.remove_group_required(ctx);
                self.unique_msg.remove(&id);
            }
            Message::InputUnsigned8(id, value, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Unsigned8(Unsigned8Item::new(value));
                }
                self.remove_required_msg(&id, false);
                self.remove_group_required(ctx);
                self.unique_msg.remove(&id);
            }
            Message::InputFloat64(id, value, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Float64(Float64Item::new(value));
                }
                self.remove_required_msg(&id, false);
                self.unique_msg.remove(&id);
            }
            Message::InvalidInputUnsigned32
            | Message::InvalidInputUnsigned8
            | Message::InvalidInputFloat64
            | Message::InvalidInputPercentage
            | Message::InvalidInputComparisonValue => return false,
            Message::InputPercentage(id, value, input_data) => {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    *item = InputItem::Percentage(PercentageItem::new(value));
                }
                self.remove_required_msg(&id, false);
                self.unique_msg.remove(&id);
            }
            Message::InputRadio(id, input_data) => {
                if let Some(buffer_option) = self.radio_buffer.get(&id) {
                    let empty = if let Ok(buffer_option) = buffer_option.try_borrow() {
                        if let Ok(mut item) = input_data.try_borrow_mut() {
                            if let InputItem::Radio(data) = &mut *item {
                                data.set_selected(buffer_option.clone());
                            }
                        }
                        buffer_option.is_empty()
                    } else {
                        false
                    };
                    self.remove_required_msg(&id, empty);
                }
                self.propagate_checkbox(ctx, &input_data);
            }
            Message::InputHostNetworkGroup(id, input_data) => {
                self.input_host_network_group(&id, &input_data);
            }
            Message::UserInputHostNetworkGroup(id) => {
                self.remove_required_msg(&id, false);
            }
            Message::InputMultipleSelect(id, input_data, list) => {
                if let Some(buffer) = self.select_searchable_buffer.get(&id) {
                    let empty = if let Ok(buffer) = buffer.try_borrow() {
                        if let Some(buffer) = buffer.as_ref() {
                            if let Ok(mut item) = input_data.try_borrow_mut() {
                                *item = InputItem::SelectMultiple(SelectMultipleItem::new(
                                    buffer.clone(),
                                ));
                            }
                            buffer.is_empty()
                        } else if let Ok(mut item) = input_data.try_borrow_mut() {
                            let list = list
                                .iter()
                                .map(|item| item.0.clone())
                                .collect::<HashSet<String>>();
                            *item = InputItem::SelectMultiple(SelectMultipleItem::new(list));
                            false
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    self.remove_required_msg(&id, empty);
                }
            }
            Message::InputSingleSelect(id, input_data, _) => {
                if let Some(buffer) = self.select_searchable_buffer.get(&id) {
                    let empty = if let Ok(buffer) = buffer.try_borrow() {
                        if let Some(buffer) = buffer.as_ref() {
                            let selected = buffer.iter().next().cloned();
                            if let Ok(mut item) = input_data.try_borrow_mut() {
                                *item = InputItem::SelectSingle(SelectSingleItem::new(selected));
                            }
                            buffer.is_empty()
                        } else if let Ok(mut item) = input_data.try_borrow_mut() {
                            *item = InputItem::SelectSingle(SelectSingleItem::new(None));
                            false
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    self.remove_required_msg(&id, empty);
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
                                input_data.clone_from(buffer);
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
                            *item = InputItem::Tag(TagItem::new(buffer.clone()));
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
                    self.remove_required_msg(&id, empty);

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
            Message::ClickCheckbox(data_id, item) => {
                self.radio_buffer_after_checkbox(&data_id, &item);
                self.propagate_checkbox(ctx, &item);
            }
            Message::InputNicName(data_id, nic_id, name, input_data) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(data) = &mut *input_data {
                        if let Some(nic) = data.get_mut(nic_id) {
                            nic.name.clone_from(&name);
                        }
                    }
                }
                self.remove_required_msg(&data_id, name.is_empty());
                self.remove_verification_nic(cal_index(Some(&data_id), nic_id));
            }
            Message::InputNicInterface(data_id, nic_id, interface, input_data) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(data) = &mut *input_data {
                        if let Some(nic) = data.get_mut(nic_id) {
                            nic.interface.clone_from(&interface);
                        }
                    }
                }
                self.remove_required_msg(&data_id, interface.is_empty());
                self.remove_verification_nic(cal_index(Some(&data_id), nic_id));
            }
            Message::InputNicGateway(data_id, nic_id, gateway, input_data) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(data) = &mut *input_data {
                        if let Some(nic) = data.get_mut(nic_id) {
                            nic.gateway.clone_from(&gateway);
                        }
                    }
                }
                self.remove_required_msg(&data_id, gateway.is_empty());
                self.remove_verification_nic(cal_index(Some(&data_id), nic_id));
            }
            Message::InputNicAdd(data_id, nic_id, input_data) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(data) = &mut *input_data {
                        data.push(InputNic::default());
                    }
                }
                self.remove_verification_nic(cal_index(Some(&data_id), nic_id));
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
                self.remove_verification_nic(cal_index(Some(&data_id), nic_id));
            }
            Message::InputGroupAdd(base_index, input_data, items_conf) => {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Group(data) = &mut *input_data {
                        if data.len() == 2_usize.pow(super::POWER_OF_MAX_NUM_OF_LAYER) {
                            // TODO: issue #188
                            return false;
                        }
                        let new_row = group_item_list_preset(&items_conf);
                        data.push(new_row);

                        if let Some(d) = data.last() {
                            let row_rep_index = cal_index(Some(&base_index), data.len() - 1);
                            self.group_row_to_buffer(&row_rep_index, d, &items_conf);
                        }
                    }
                }
            }
            Message::InputGroupDelete(base_index, row_index, input_data, items_conf, required) => {
                let empty = if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Group(data) = &mut *input_data {
                        if let Some(d) = data.get(row_index) {
                            for (col, d) in d.iter().enumerate() {
                                if let Ok(d) = d.try_borrow() {
                                    match &*d {
                                        InputItem::SelectSingle(_)
                                        | InputItem::SelectMultiple(_) => {
                                            rearrange_buffer(
                                                &mut self.select_searchable_buffer,
                                                &base_index,
                                                row_index,
                                                col,
                                                data.len(),
                                            );
                                        }
                                        InputItem::Comparison(_) => {
                                            rearrange_buffer(
                                                &mut self.comparison_value_kind_buffer,
                                                &base_index,
                                                row_index,
                                                col,
                                                data.len(),
                                            );
                                            rearrange_buffer(
                                                &mut self.comparison_value_cmp_buffer,
                                                &base_index,
                                                row_index,
                                                col,
                                                data.len(),
                                            );
                                            rearrange_buffer(
                                                &mut self.comparison_value_buffer,
                                                &base_index,
                                                row_index,
                                                col,
                                                data.len(),
                                            );
                                        }
                                        InputItem::VecSelect(_) => {
                                            rearrange_buffer(
                                                &mut self.vec_select_buffer,
                                                &base_index,
                                                row_index,
                                                col,
                                                data.len(),
                                            );
                                        }
                                        _ => (), // The rest don't have buffer.
                                    }
                                }
                                self.required_msg.remove(
                                    &(cal_index(
                                        Some(&cal_index(Some(&base_index), row_index)),
                                        col,
                                    )),
                                );

                                for r in row_index + 1..data.len() {
                                    if self.required_msg.remove(
                                        &(cal_index(Some(&cal_index(Some(&base_index), r)), col)),
                                    ) {
                                        self.required_msg.insert(cal_index(
                                            Some(&cal_index(Some(&base_index), r - 1)),
                                            col,
                                        ));
                                    }
                                }
                            }
                        }
                        data.remove(row_index);
                        if required && data.is_inside_empty() {
                            self.required_msg.insert(base_index.clone());
                        }
                        data.is_empty()
                    } else {
                        false
                    }
                } else {
                    false
                };
                if empty {
                    ctx.link()
                        .send_message(Message::InputGroupAdd(base_index, input_data, items_conf));
                }
            }
            Message::InputComparisonValueKind(data_id, input_data) => {
                self.clear_comparison_value(&data_id, &input_data);
            }
            Message::InputComparisonComparisionKind(data_id, input_data) => {
                self.input_comparison_comparison_kind(&data_id, &input_data);
            }
            Message::InputComparisonValue(data_id, value_index, value, input_data) => {
                self.input_comparison_value(&data_id, value_index, &value, &input_data);
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

                    self.file_data_id = Some(data_id.clone());
                    self.file_input_data = Some(Rc::clone(&input_data));
                    self.file_name = Some(file_name);
                    self.file_reader = Some(task);
                }
            }
            Message::FileLoaded(file_name, file) => {
                if let Some(input_data) = self.file_input_data.as_ref() {
                    if let Ok(mut item) = input_data.try_borrow_mut() {
                        let content = BASE64.encode(&file);
                        *item = InputItem::File(FileItem::new(file_name, content));
                    }
                }
                self.file_reader = None;
            }
            // TODO: issue #5
            Message::FailLoadFile => {
                return false;
            }
            // TODO: issue #5
            Message::InputError => {}
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = if cfg!(feature = "pumpkin") {
            format!("width: {}px;", ctx.props().width)
        } else {
            format!(
                "width: {}px; max-height: {}px;",
                ctx.props().width,
                ctx.props().height
            )
        };
        let style_input = if cfg!(feature = "pumpkin") {
            String::new()
        } else {
            format!("height: {}px;", ctx.props().height - 166)
        };
        let txt = ctx.props().txt.txt.clone();
        let onclick_escape = ctx.link().callback(|_| Message::Escape);
        let onclick_save = ctx.link().callback(|_| Message::TrySave);
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
    }
}

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    fn view_input(&self, ctx: &Context<Self>) -> Html {
        html! {
            for ctx.props().input_data.iter().enumerate().zip(ctx.props().input_conf.iter()).map(|((index , input_data), input_conf)| {
                match &**input_conf {
                    InputConfig::Text(config) => {
                        self.view_text(ctx, &config.ess, config.length, config.width, input_data,
                            None, index, index == 0, false, config.immutable)
                    }
                    InputConfig::Password(config) => {
                        self.view_password(ctx, &config.ess, config.width, input_data, None, index,
                            index == 0)
                    }
                    InputConfig::HostNetworkGroup(config) => {
                        self.view_host_network_group(ctx, &config.ess, config.kind, config.num,
                            config.width, input_data, None, index)
                    }
                    InputConfig::SelectSingle(config) => {
                        self.view_select_searchable(ctx, false, &config.ess, config.width,
                            &config.options, input_data, None, index, 0, false)
                    }
                    InputConfig::SelectMultiple(config) => {
                        self.view_select_nic_or(ctx, config.options.as_ref(), config.nic_index, &config.ess,
                            input_data, None, index, 0)
                    }
                    InputConfig::Tag(config) => {
                        self.view_tag_group(ctx, &config.ess, &config.name_map, input_data, None, index)
                    }
                    InputConfig::VecSelect(config) => {
                        self.view_vec_select(ctx, &config.ess, &config.items_ess_list, config.last,
                            config.full_width, &config.widths, &config.max_widths,
                            &config.max_heights, &config.map_list, input_data, None, index, false)
                    }
                    InputConfig::Unsigned32(config) => {
                        self.view_unsigned_32(ctx, &config.ess, config.min, config.max,
                            config.width, input_data, None, index, index == 0, false)
                    }
                    InputConfig::Unsigned8(config) => {
                        self.view_unsigned_8(ctx, &config.ess, config.min, config.max,
                            config.width, input_data, None, index, index == 0, false)
                    }
                    InputConfig::Float64(config) => {
                        self.view_float_64(ctx, &config.ess, config.step, config.width, input_data,
                            None, index, index == 0, false)
                    }
                    InputConfig::Percentage(config) => {
                        self.view_percentage(ctx, &config.ess, config.min, config.max,
                            config.num_decimals, config.width, input_data, None, index, index == 0)
                    }
                    InputConfig::Nic(config) => {
                        self.view_nic(ctx, &config.ess, input_data, None, index)
                    }
                    InputConfig::File(config) => {
                        self.view_file(ctx, &config.ess, &config.allowed_extensions, input_data, None, index)
                    }
                    InputConfig::Comparison(config) => {
                        self.view_comparison(ctx, &config.ess, input_data, None, index, false)
                    }
                    InputConfig::Group(config) => {
                        self.view_group(ctx, &config.ess, config.all_in_one_row, &config.widths,
                            &config.items, input_data, None, index)
                    }
                    InputConfig::Checkbox(config) => {
                        let both = ctx.props().input_conf.get(index + 1).map_or(Some(false),|next| {
                            if let InputConfig::Checkbox(_) = &**next {
                                Some(false)
                            } else {
                                Some(true)
                            }
                        });
                        self.view_checkbox(ctx, &config.ess, config.language, config.always, config.children.as_ref(),
                            input_data, None, index, both, 1)
                    }
                    InputConfig::Radio(config) => {
                        self.view_radio(ctx, &config.ess, &config.options, &config.children_group,
                            input_data, None, index, 1)
                    }
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

    fn remove_required_msg(&mut self, id: &BigUint, empty: bool) {
        if !empty {
            self.required_msg.remove(id);
        }
    }

    fn input_host_network_group(&mut self, id: &BigUint, input_data: &Rc<RefCell<InputItem>>) {
        if let Some(buffer) = self.host_network_buffer.get(id) {
            let empty = if let Ok(buffer) = buffer.try_borrow_mut() {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    let mut n = buffer.clone();
                    sort_hosts(&mut n.hosts);
                    sort_networks(&mut n.networks);
                    n.ranges.sort_unstable();
                    *item = InputItem::HostNetworkGroup(HostNetworkGroupItem::new(n));
                }
                buffer.is_empty()
            } else {
                false
            };
            self.remove_required_msg(id, empty);
        }
    }

    fn remove_verification_nic(&mut self, id: BigUint) {
        self.verification_nic.remove(&(id.clone(), 0));
        self.verification_nic.remove(&(id.clone(), 1));
        self.verification_nic.remove(&(id, 2));
    }

    fn decide_unique_all(&mut self, ctx: &Context<Self>) -> bool {
        // no need to check Checkbox's children because Checkbox and its children don't need to be unique
        let mut unique = Vec::<bool>::new();
        let id = ctx.props().input_second_id.as_ref().map_or_else(
            || ctx.props().input_id.as_deref(),
            |id| match id {
                InputSecondId::Add => None,
                InputSecondId::Edit(i) => Some(i),
            },
        );

        for (index, t) in ctx.props().input_conf.iter().enumerate() {
            if let InputConfig::Text(conf) = &(**t) {
                if let Some(data) = ctx.props().input_data.get(index) {
                    if let Ok(input) = data.try_borrow() {
                        if conf.unique {
                            let mut different = true;
                            for (key, item) in &*ctx.props().data {
                                if id.as_ref().is_none_or(|id| id != key) {
                                    if let Some(other) = item.columns.get(index) {
                                        if let (Column::Text(other_value), InputItem::Text(value)) =
                                            (other, &(*input))
                                        {
                                            if let ViewString::Raw(other_value) = &other_value.text
                                            {
                                                if value == other_value {
                                                    different = false;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if !different {
                                self.unique_msg.insert(BigUint::from(index));
                                unique.push(true);
                            }
                        }
                    }
                }
            }
        }
        !unique.is_empty()
    }

    fn radio_buffer_after_checkbox(&mut self, data_id: &BigUint, item: &Rc<RefCell<InputItem>>) {
        if let Ok(item) = item.try_borrow() {
            if let InputItem::Checkbox(cb) = &*item {
                for (sub_index, child) in cb.children().iter().enumerate() {
                    if let Ok(child) = child.try_borrow() {
                        if let InputItem::Radio(data) = &*child {
                            let id = cal_index(Some(data_id), sub_index);
                            if let Some(buffer_option) = self.radio_buffer.get(&id) {
                                if let Ok(mut buffer_option) = buffer_option.try_borrow_mut() {
                                    (*buffer_option).clone_from(&data.selected().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn rearrange_buffer<T>(
    buffer: &mut HashMap<BigUint, T>,
    base: &BigUint,
    row: usize,
    col: usize,
    len: usize,
) where
    T: Clone,
{
    for row_index in row + 1..len {
        let index = cal_index(Some(&cal_index(Some(base), row_index)), col);
        let Some(item) = buffer.remove(&index) else {
            continue;
        };
        let index = cal_index(Some(&cal_index(Some(base), row_index - 1)), col);
        buffer.insert(index, item.clone());
    }
}
