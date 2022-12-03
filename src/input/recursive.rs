use super::{
    super::CheckStatus,
    component::{InvalidMessage, Model, Verification},
    user_input::MAX_PER_LAYER,
    InputItem, InputType,
};
use ipnet::Ipv4Net;
use passwords::analyzer;
use std::cell::RefCell;
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::rc::Rc;
use std::str::FromStr;
use yew::{Component, Context};

const PASSWORD_MIN_LEN: usize = if cfg!(feature = "cc-password") { 9 } else { 8 };
const PASSWORD_MIN_FORBID_ADJACENT_LEN: usize = 4; // adjacent keyboard characters

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub(super) fn prepare_buffer(&mut self, ctx: &Context<Self>) {
        self.prepare_buffer_recursive(&ctx.props().input_data, &ctx.props().input_type, 1);
    }

    pub(super) fn prepare_buffer_recursive(
        &mut self,
        input_data: &[Rc<RefCell<InputItem>>],
        input_type: &[Rc<InputType>],
        base_index: usize,
    ) {
        input_data
            .iter()
            .enumerate()
            .zip(input_type.iter())
            .for_each(|((index, input_data), input_type)| {
                let this_index = base_index + index;
                if let Ok(data) = input_data.try_borrow() {
                    match (&*data, &**input_type) {
                        (InputItem::Text(data), InputType::Radio(_, _)) => {
                            self.radio_buffer
                                .insert(this_index, Rc::new(RefCell::new(data.clone())));
                        }
                        (
                            InputItem::HostNetworkGroup(data),
                            InputType::HostNetworkGroup(_, _, _, _),
                        ) => {
                            self.host_network_buffer
                                .insert(this_index, Rc::new(RefCell::new(data.clone())));
                        }
                        (
                            InputItem::SelectMultiple(data),
                            InputType::SelectMultiple(_, _, _, _, all),
                        ) => {
                            if *all {
                                self.select_searchable_buffer
                                    .insert(this_index, Rc::new(RefCell::new(None)));
                            } else {
                                self.select_searchable_buffer
                                    .insert(this_index, Rc::new(RefCell::new(Some(data.clone()))));
                            }
                        }
                        (InputItem::SelectSingle(data), InputType::SelectSingle(_, _, _)) => {
                            let mut buf = HashSet::new();
                            if let Some(data) = data {
                                buf.insert(data.clone());
                            }
                            self.select_searchable_buffer
                                .insert(this_index, Rc::new(RefCell::new(Some(buf))));
                        }
                        (InputItem::Tag(data), InputType::Tag(_, _)) => {
                            self.tag_buffer
                                .insert(this_index, Rc::new(RefCell::new(data.clone())));
                        }
                        (
                            InputItem::CheckBox(_, data_children),
                            InputType::CheckBox(_, _, Some(children)),
                        ) => {
                            if let Some(data_children) = data_children {
                                self.prepare_buffer_recursive(
                                    data_children,
                                    &children.1,
                                    this_index * MAX_PER_LAYER,
                                );
                            }
                        }
                        (InputItem::Group(data), InputType::Group(_, _, _, group)) => {
                            for (row, d) in data.iter().enumerate() {
                                for ((col, d), t) in d.iter().enumerate().zip(group.iter()) {
                                    if let Ok(d) = d.try_borrow() {
                                        let sub_base_index = this_index * MAX_PER_LAYER;
                                        if let (
                                            InputItem::SelectSingle(data),
                                            InputType::SelectSingle(..),
                                        ) = (&*d, &**t)
                                        {
                                            let mut buf = HashSet::new();
                                            if let Some(data) = data {
                                                buf.insert(data.clone());
                                            }
                                            self.select_searchable_buffer.insert(
                                                col + (row + sub_base_index) * MAX_PER_LAYER,
                                                Rc::new(RefCell::new(Some(buf))),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        // TODO: implement if necessary
                        (_, _) => (),
                    }
                }
            });
    }

    pub(super) fn prepare_default(&mut self, ctx: &Context<Self>) {
        self.prepare_default_recursive(&ctx.props().input_data, &ctx.props().input_type, true, 1);
    }

    pub(super) fn prepare_default_recursive(
        &mut self,
        input_data: &[Rc<RefCell<InputItem>>],
        input_type: &[Rc<InputType>],
        parent_checked: bool,
        base_index: usize,
    ) {
        input_data
            .iter()
            .enumerate()
            .zip(input_type.iter())
            .for_each(|((index, input_data), input_type)| {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    match (&mut *item, &**input_type) {
                        (
                            InputItem::Text(_) | InputItem::Password(_),
                            InputType::Text(ess, _, _) | InputType::Radio(ess, _),
                        )
                        | (
                            InputItem::HostNetworkGroup(_),
                            InputType::HostNetworkGroup(ess, _, _, _),
                        )
                        | (InputItem::Tag(_), InputType::Tag(ess, _))
                        | (InputItem::Unsigned32(_), InputType::Unsigned32(ess, _, _, _))
                        | (InputItem::Percentage(_), InputType::Percentage(ess, _, _, _, _))
                        | (InputItem::Nic(_), InputType::Nic(ess))
                        | (InputItem::File(_, _), InputType::File(ess)) => {
                            if let Some(default) = &ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                    let id = base_index + index;
                                    self.default_to_buffer_radio(id, default);
                                }
                            }
                        }
                        (
                            InputItem::CheckBox(checked, data_children),
                            InputType::CheckBox(ess, _, children),
                        ) => {
                            if let Some(InputItem::CheckBox(c, _)) = &ess.default {
                                if parent_checked {
                                    *checked = *c;
                                }
                            }
                            if let (Some(data_children), Some(children)) = (data_children, children)
                            {
                                if *checked != CheckStatus::Unchecked {
                                    self.prepare_default_recursive(
                                        data_children,
                                        &children.1,
                                        *checked == CheckStatus::Checked
                                            || *checked == CheckStatus::Indeterminate,
                                        (base_index + index) * MAX_PER_LAYER,
                                    );
                                }
                            }
                        }
                        (InputItem::SelectSingle(_), InputType::SelectSingle(ess, _, _)) => {
                            if let Some(default) = &ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                    let id = base_index + index;
                                    self.default_to_buffer_select_single(id, default);
                                }
                            }
                        }
                        (
                            InputItem::SelectMultiple(_),
                            InputType::SelectMultiple(ess, _, _, _, _),
                        ) => {
                            if let Some(default) = &ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                    let id = base_index + index;
                                    self.default_to_buffer_select_multiple(id, default);
                                }
                            }
                        }
                        (InputItem::Group(_), InputType::Group(ess, _, _, _)) => {
                            if let Some(InputItem::Group(default)) = &ess.default {
                                if let Some(default) = default.first() {
                                    if let Some(copy_default) = Self::copy_default(default) {
                                        if parent_checked {
                                            *item = InputItem::Group(vec![copy_default]);
                                        }
                                    }
                                }
                            }
                        }
                        (_, _) => (),
                    }
                }
            });
    }

    pub(super) fn copy_default(
        default: &[Rc<RefCell<InputItem>>],
    ) -> Option<Vec<Rc<RefCell<InputItem>>>> {
        let copy_default = default
            .iter()
            .filter_map(|d| {
                if let Ok(d) = d.try_borrow() {
                    Some(d.clone())
                } else {
                    None
                }
            })
            .into_iter()
            .map(|d| Rc::new(RefCell::new(d)))
            .collect::<Vec<Rc<RefCell<InputItem>>>>();
        if default.len() == copy_default.len() {
            Some(copy_default)
        } else {
            None
        }
    }

    pub(super) fn default_to_buffer_radio(&mut self, id: usize, default: &InputItem) {
        if let (InputItem::Text(default), Some(buffer)) = (default, self.radio_buffer.get(&id)) {
            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                if !default.is_empty() {
                    *buffer = default.clone();
                }
            }
        }
    }

    pub(super) fn default_to_buffer_select_single(&mut self, id: usize, default: &InputItem) {
        if let (InputItem::SelectSingle(Some(default)), Some(buffer)) =
            (default, self.select_searchable_buffer.get(&id))
        {
            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                if !default.is_empty() {
                    let mut value: HashSet<String> = HashSet::new();
                    value.insert(default.clone());
                    *buffer = Some(value);
                }
            }
        }
    }

    pub(super) fn default_to_buffer_select_multiple(&mut self, id: usize, default: &InputItem) {
        if let (InputItem::SelectMultiple(default), Some(buffer)) =
            (default, self.select_searchable_buffer.get(&id))
        {
            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                if !default.is_empty() {
                    *buffer = Some(default.clone());
                }
            }
        }
    }

    pub(super) fn decide_required_all(&mut self, ctx: &Context<Self>) -> bool {
        self.decide_required_all_recursive(
            ctx,
            &ctx.props().input_data,
            &ctx.props().input_type,
            1,
            true,
        )
    }

    pub(super) fn decide_required_all_recursive(
        &mut self,
        ctx: &Context<Self>,
        input_data: &[Rc<RefCell<InputItem>>],
        input_type: &[Rc<InputType>],
        base_index: usize,
        parent_checked: bool,
    ) -> bool {
        let mut required = Vec::<bool>::new();

        input_data
            .iter()
            .enumerate()
            .zip(input_type.iter())
            .for_each(|((index, input_data), input_type)| {
                if let Ok(item) = input_data.try_borrow() {
                    if parent_checked && (*input_type).required() {
                        let empty = match &(*item) {
                            InputItem::Text(txt) => txt.is_empty(),
                            InputItem::Password(pw) => {
                                // HIGHLIGHT: In case of Edit, empty means no change of passwords
                                ctx.props().input_id.is_none() && pw.is_empty()
                            }
                            InputItem::HostNetworkGroup(n) => {
                                // HIGHTLIGHT: if empty, HostNetworkHtml may return Message::RightHostNetworkGroup
                                n.is_empty()
                                    && self
                                        .verification_host_network
                                        .get(&(base_index + index))
                                        .map_or(false, |v| v.map_or(true, |v| v))
                            }
                            InputItem::SelectSingle(s) => s.is_none(),
                            InputItem::SelectMultiple(s) => s.is_empty(),
                            InputItem::Unsigned32(v) => v.is_none(),
                            InputItem::Float64(v) => v.is_none(),
                            InputItem::Percentage(v) => v.is_none(),
                            InputItem::CheckBox(s, _) => *s == CheckStatus::Unchecked,
                            InputItem::Nic(n) => n
                                .iter()
                                .find_map(|n| {
                                    if !n.name.is_empty()
                                        || !n.interface.is_empty()
                                        || !n.gateway.is_empty()
                                    {
                                        Some(true)
                                    } else {
                                        None
                                    }
                                })
                                .is_none(),
                            InputItem::File(_, content) => content.is_empty(),
                            InputItem::Tag(_) => false,
                            InputItem::Group(group) => group.is_empty(),
                        };
                        if empty {
                            self.required_msg.insert(base_index + index);
                            required.push(true);
                        }
                    }

                    if let (
                        InputItem::CheckBox(checked, Some(data_children)),
                        InputType::CheckBox(_, _, Some(type_children)),
                    ) = (&(*item), &**input_type)
                    {
                        if self.decide_required_all_recursive(
                            ctx,
                            data_children,
                            &type_children.1,
                            (base_index + index) * MAX_PER_LAYER,
                            *checked == CheckStatus::Checked,
                        ) {
                            required.push(true);
                        }
                    }
                }
            });

        !required.is_empty() || !self.required_msg.is_empty()
    }

    pub(super) fn verify(&mut self, ctx: &Context<Self>) -> bool {
        self.verify_recursive(&ctx.props().input_data, &ctx.props().input_type, 1, true)
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn verify_recursive(
        &mut self,
        input_data: &[Rc<RefCell<InputItem>>],
        input_type: &[Rc<InputType>],
        base_index: usize,
        parent_checked: bool,
    ) -> bool {
        let mut rtn = true;

        input_data
            .iter()
            .enumerate()
            .zip(input_type.iter())
            .for_each(|((index, input_data), input_type)| {
                if let Ok(input_data) = input_data.try_borrow() {
                    // HIGHTLIGHT: All kinds are not necessarily to be verified.
                    // HIGHTLIGHT: Since HostNetworkGroup items were verified yet, they don't need to be verified here.
                    match (&*input_data, &**input_type) {
                        (
                            InputItem::Unsigned32(Some(value)),
                            InputType::Unsigned32(_, min, max, _),
                        ) => {
                            if parent_checked {
                                if *value >= *min && *value <= *max {
                                    self.verification
                                        .insert(base_index + index, Verification::Valid);
                                } else {
                                    self.verification.insert(
                                        base_index + index,
                                        Verification::Invalid(InvalidMessage::InvalidInput),
                                    );
                                    rtn = false;
                                }
                            }
                        }
                        (
                            InputItem::Percentage(Some(value)),
                            InputType::Percentage(_, min, max, _, _),
                        ) => {
                            if parent_checked {
                                if *value >= (*min).unwrap_or(0.0)
                                    && *value <= (*max).unwrap_or(1.0)
                                {
                                    self.verification
                                        .insert(base_index + index, Verification::Valid);
                                } else {
                                    self.verification.insert(
                                        base_index + index,
                                        Verification::Invalid(InvalidMessage::InvalidInput),
                                    );
                                    rtn = false;
                                }
                            }
                        }
                        (InputItem::Password(pwd), InputType::Password(_, _)) => {
                            if parent_checked {
                                if let Some(cnf_pwd) =
                                    self.confirm_password.get(&(base_index + index))
                                {
                                    if pwd == cnf_pwd {
                                        if let Some(v) = invalid_password(pwd) {
                                            self.verification.insert(
                                                base_index + index,
                                                Verification::Invalid(v),
                                            );
                                            rtn = false;
                                        } else {
                                            self.verification
                                                .insert(base_index + index, Verification::Valid);
                                        }
                                    } else {
                                        self.verification.insert(
                                            base_index + index,
                                            Verification::Invalid(InvalidMessage::PasswordNotMatch),
                                        );
                                        rtn = false;
                                    }
                                }
                            }
                        }
                        (InputItem::Nic(nics), InputType::Nic(_)) => {
                            if parent_checked {
                                for (i, nic) in nics.iter().enumerate() {
                                    if !nic.interface.is_empty() || !nic.gateway.is_empty() {
                                        if nic.name.is_empty() {
                                            self.verification_nic.insert(
                                                ((base_index + index) * MAX_PER_LAYER + i, 0),
                                                Verification::Invalid(
                                                    InvalidMessage::InterfaceNameRequired,
                                                ),
                                            );
                                            rtn = false;
                                        }
                                        if nic.interface.is_empty() {
                                            self.verification_nic.insert(
                                                ((base_index + index) * MAX_PER_LAYER + i, 1),
                                                Verification::Invalid(
                                                    InvalidMessage::InterfaceRequired,
                                                ),
                                            );
                                            rtn = false;
                                        } else if Ipv4Net::from_str(&nic.interface).is_err() {
                                            self.verification_nic.insert(
                                                ((base_index + index) * MAX_PER_LAYER + i, 1),
                                                Verification::Invalid(
                                                    InvalidMessage::WrongInterface,
                                                ),
                                            );
                                            rtn = false;
                                        }
                                        if nic.gateway.is_empty() {
                                            self.verification_nic.insert(
                                                ((base_index + index) * MAX_PER_LAYER + i, 2),
                                                Verification::Invalid(
                                                    InvalidMessage::GatewayRequired,
                                                ),
                                            );
                                            rtn = false;
                                        } else if Ipv4Addr::from_str(&nic.gateway).is_err() {
                                            self.verification_nic.insert(
                                                ((base_index + index) * MAX_PER_LAYER + i, 2),
                                                Verification::Invalid(InvalidMessage::WrongGateway),
                                            );
                                            rtn = false;
                                        }
                                    } else if !nic.name.is_empty() {
                                        self.verification_nic.insert(
                                            ((base_index + index) * MAX_PER_LAYER + i, 1),
                                            Verification::Invalid(
                                                InvalidMessage::InterfaceRequired,
                                            ),
                                        );
                                        self.verification_nic.insert(
                                            ((base_index + index) * MAX_PER_LAYER + i, 2),
                                            Verification::Invalid(InvalidMessage::GatewayRequired),
                                        );
                                        rtn = false;
                                    }
                                }
                            }
                        }
                        (
                            InputItem::CheckBox(checked, Some(data_children)),
                            InputType::CheckBox(_, _, Some(type_children)),
                        ) => {
                            if *checked != CheckStatus::Unchecked
                                && !self.verify_recursive(
                                    data_children,
                                    &type_children.1,
                                    (base_index + index) * MAX_PER_LAYER,
                                    *checked == CheckStatus::Checked,
                                )
                            {
                                rtn = false;
                            }
                        }
                        (_, _) => (),
                    }
                }
            });

        rtn
    }

    pub(super) fn trim_nic(ctx: &Context<Self>) {
        Self::trim_nic_recursive(&ctx.props().input_data, &ctx.props().input_type);
    }

    pub(super) fn trim_nic_recursive(
        input_data: &[Rc<RefCell<InputItem>>],
        input_type: &[Rc<InputType>],
    ) {
        input_data
            .iter()
            .zip(input_type.iter())
            .for_each(|(input_data, input_type)| {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(nics) = &mut *input_data {
                        nics.retain(|n| {
                            !n.name.is_empty() && !n.interface.is_empty() && !n.gateway.is_empty()
                        });
                    }
                }
                if let Ok(input_data) = input_data.try_borrow() {
                    if let (
                        InputItem::CheckBox(checked, Some(data_children)),
                        InputType::CheckBox(_, _, Some(type_children)),
                    ) = (&*input_data, &**input_type)
                    {
                        if *checked != CheckStatus::Unchecked {
                            Self::trim_nic_recursive(data_children, &type_children.1);
                        }
                    }
                }
            });
    }

    pub(super) fn propagate_checkbox(
        &mut self,
        ctx: &Context<Self>,
        click: &Rc<RefCell<InputItem>>,
    ) -> bool {
        let mut propagate: Vec<(usize, Rc<RefCell<InputItem>>, Rc<InputType>)> = Vec::new();

        ctx.props()
            .input_data
            .iter()
            .enumerate()
            .zip(ctx.props().input_type.iter())
            .for_each(|((index, input_data), input_type)| {
                if let Ok(item) = input_data.try_borrow() {
                    if let (InputItem::CheckBox(_, _), InputType::CheckBox(_, _, _)) =
                        (&(*item), &**input_type)
                    {
                        propagate.push((index, Rc::clone(input_data), Rc::clone(input_type)));
                    }
                }
            });

        for (index, p, t) in &propagate {
            self.propagate_checkbox_recursive(click, p, t, None, *index, 1);
        }

        true
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines)]
    pub(super) fn propagate_checkbox_recursive(
        &mut self,
        click: &Rc<RefCell<InputItem>>,
        pos: &Rc<RefCell<InputItem>>,
        input_type: &Rc<InputType>,
        checked: Option<CheckStatus>,
        layer_index: usize,
        base_index: usize,
    ) -> Option<CheckStatus> {
        let this_checked = if Rc::ptr_eq(click, pos) {
            if let Ok(mut click) = click.try_borrow_mut() {
                if let (InputItem::CheckBox(status, _), InputType::CheckBox(_, always, _)) =
                    (&mut *click, &**input_type)
                {
                    match *status {
                        CheckStatus::Checked => {
                            if *always != Some(CheckStatus::Checked)
                                && *always != Some(CheckStatus::Indeterminate)
                            {
                                *status = checked.unwrap_or(CheckStatus::Unchecked);
                            }
                            Some(*status)
                        }
                        CheckStatus::Indeterminate | CheckStatus::Unchecked => {
                            if *always != Some(CheckStatus::Unchecked) {
                                *status = checked.unwrap_or(CheckStatus::Checked);
                            }
                            Some(*status)
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else if let Some(checked) = checked {
            if let Ok(mut pos) = pos.try_borrow_mut() {
                if let (InputItem::CheckBox(status, _), InputType::CheckBox(_, _, _)) =
                    (&mut *pos, &**input_type)
                {
                    *status = checked;
                    Some(checked)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let mut propa_children: Vec<(usize, Rc<RefCell<InputItem>>, Rc<InputType>)> = Vec::new();
        if let Ok(pos) = pos.try_borrow_mut() {
            if let (InputItem::CheckBox(_, children), InputType::CheckBox(_, _, type_children)) =
                (&*pos, &**input_type)
            {
                if let (Some(children), Some(type_children)) = (children, type_children) {
                    for (index, child) in children.iter().enumerate() {
                        if let (Ok(mut c), Some(t)) =
                            (child.try_borrow_mut(), type_children.1.get(index))
                        {
                            match (&(*c), &**t) {
                                (InputItem::CheckBox(_, _), InputType::CheckBox(_, _, _)) => {
                                    propa_children.push((index, Rc::clone(child), Rc::clone(t)));
                                }
                                (
                                    InputItem::Text(user),
                                    InputType::Text(ess, _, _) | InputType::Radio(ess, _),
                                ) => {
                                    if user.is_empty()
                                        || this_checked == Some(CheckStatus::Unchecked)
                                    {
                                        if let Some(value) = &ess.default {
                                            *c = value.clone();
                                        }
                                    }
                                }
                                (
                                    InputItem::HostNetworkGroup(user),
                                    InputType::HostNetworkGroup(ess, _, _, _),
                                ) => {
                                    if user.is_empty()
                                        || this_checked == Some(CheckStatus::Unchecked)
                                    {
                                        if let Some(value) = &ess.default {
                                            *c = value.clone();
                                        }
                                    }
                                }
                                (
                                    InputItem::SelectSingle(user),
                                    InputType::SelectSingle(ess, _, _),
                                ) => {
                                    if user.is_none()
                                        || this_checked == Some(CheckStatus::Unchecked)
                                    {
                                        if let Some(value) = &ess.default {
                                            *c = value.clone();
                                        }
                                    }
                                }
                                (
                                    InputItem::SelectMultiple(user),
                                    InputType::SelectMultiple(ess, _, _, _, _),
                                ) => {
                                    if user.is_empty()
                                        || this_checked == Some(CheckStatus::Unchecked)
                                    {
                                        if let Some(value) = &ess.default {
                                            *c = value.clone();
                                        }
                                    }
                                }
                                (InputItem::Tag(user), InputType::Tag(ess, _)) => {
                                    if (user.old.is_empty()
                                        && user.new.is_none()
                                        && user.edit.is_none()
                                        && user.delete.is_none())
                                        || this_checked == Some(CheckStatus::Unchecked)
                                    {
                                        if let Some(value) = &ess.default {
                                            *c = value.clone();
                                        }
                                    }
                                }
                                (
                                    InputItem::Unsigned32(user),
                                    InputType::Unsigned32(ess, _, _, _),
                                ) => {
                                    if user.is_none()
                                        || this_checked == Some(CheckStatus::Unchecked)
                                    {
                                        if let Some(value) = &ess.default {
                                            *c = value.clone();
                                        }
                                    }
                                }
                                (
                                    InputItem::Percentage(user),
                                    InputType::Percentage(ess, _, _, _, _),
                                ) => {
                                    if user.is_none()
                                        || this_checked == Some(CheckStatus::Unchecked)
                                    {
                                        if let Some(value) = &ess.default {
                                            *c = value.clone();
                                        }
                                    }
                                }
                                (InputItem::Nic(user), InputType::Nic(ess)) => {
                                    if user.is_empty()
                                        || this_checked == Some(CheckStatus::Unchecked)
                                    {
                                        if let Some(value) = &ess.default {
                                            *c = value.clone();
                                        }
                                    }
                                }
                                (_, _) => (),
                            }
                        }
                    }
                }
            }
        }
        let mut rtn_checked: Vec<Option<CheckStatus>> = Vec::new();

        for (index, child, type_child) in &propa_children {
            rtn_checked.push(self.propagate_checkbox_recursive(
                click,
                child,
                type_child,
                this_checked,
                *index,
                (base_index + layer_index) * MAX_PER_LAYER,
            ));
        }

        let updated_checked = if rtn_checked.is_empty() {
            None
        } else if rtn_checked.len()
            == rtn_checked
                .iter()
                .filter_map(|r| {
                    r.map_or(None, |r| {
                        if r == CheckStatus::Checked {
                            Some(true)
                        } else {
                            None
                        }
                    })
                })
                .count()
        {
            Some(CheckStatus::Checked)
        } else if rtn_checked.len()
            == rtn_checked
                .iter()
                .filter_map(|r| {
                    r.map_or(None, |r| {
                        if r == CheckStatus::Unchecked {
                            Some(true)
                        } else {
                            None
                        }
                    })
                })
                .count()
        {
            Some(CheckStatus::Unchecked)
        } else {
            Some(CheckStatus::Indeterminate)
        };

        let final_checked = if let Ok(mut pos) = pos.try_borrow_mut() {
            if let InputItem::CheckBox(status, _) = &mut (*pos) {
                if let Some(updated_checked) = updated_checked {
                    *status = updated_checked;
                }
                Some(*status)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(CheckStatus::Unchecked) = final_checked {
            self.required_msg
                .retain(|x| *x / MAX_PER_LAYER != (base_index + layer_index));
        }

        final_checked
    }

    pub(super) fn reset_veri_host_network(&mut self, ctx: &Context<Self>) {
        self.verification_host_network.clear();

        self.reset_veri_host_network_recursive(
            &ctx.props().input_data,
            &ctx.props().input_type,
            1,
            true,
        );
    }

    pub(super) fn reset_veri_host_network_recursive(
        &mut self,
        input_data: &[Rc<RefCell<InputItem>>],
        input_type: &[Rc<InputType>],
        base_index: usize,
        parent_checked: bool,
    ) {
        input_data
            .iter()
            .enumerate()
            .zip(input_type.iter())
            .for_each(|((index, input_data), input_type)| {
                if let Ok(input_data) = input_data.try_borrow() {
                    if parent_checked {
                        if let (
                            InputItem::HostNetworkGroup(_),
                            InputType::HostNetworkGroup(_, _, _, _),
                        ) = (&*input_data, &**input_type)
                        {
                            self.verification_host_network
                                .insert(base_index + index, None);
                        }
                    }

                    if let (
                        InputItem::CheckBox(checked, Some(data_children)),
                        InputType::CheckBox(_, _, Some(type_children)),
                    ) = (&*input_data, &**input_type)
                    {
                        if *checked != CheckStatus::Unchecked {
                            self.reset_veri_host_network_recursive(
                                data_children,
                                &type_children.1,
                                (base_index + index) * MAX_PER_LAYER,
                                *checked == CheckStatus::Checked,
                            );
                        }
                    }
                }
            });
    }
}

const PASSWD_CMP: [&str; 7] = [
    "1234567890",
    "qwertyuiop",
    "QWERTYUIOP",
    "asdfghjkl",
    "ASDFGHJKL",
    "zxcvbnm",
    "ZXCVBNM",
];

fn invalid_password(password: &str) -> Option<InvalidMessage> {
    let analyzed = analyzer::analyze(password);
    let filtered = analyzed.password();

    #[allow(clippy::collapsible_else_if)]
    if cfg!(feature = "cc-password") {
        if password != filtered {
            Some(InvalidMessage::PasswordHasControlCharacter)
        } else if analyzed.spaces_count() > 0 {
            Some(InvalidMessage::PasswordHasSpace)
        } else if analyzed.length() < PASSWORD_MIN_LEN {
            Some(InvalidMessage::PasswordTooShort)
        } else if analyzed.lowercase_letters_count() == 0 {
            Some(InvalidMessage::PasswordNoLowercaseLetter)
        } else if analyzed.uppercase_letters_count() == 0 {
            Some(InvalidMessage::PasswordNoUppercaseLetter)
        } else if analyzed.numbers_count() == 0 {
            Some(InvalidMessage::PasswordNoNumber)
        } else if analyzed.symbols_count() == 0 {
            Some(InvalidMessage::PasswordNoSymbol)
        } else if analyzed.consecutive_count() > 0 {
            Some(InvalidMessage::PasswordHasConsecutiveLetters)
        } else if cmp_consecutive(password) {
            Some(InvalidMessage::PasswordHasAdjacentLetters)
        } else {
            None
        }
    } else {
        if password != filtered {
            Some(InvalidMessage::PasswordHasControlCharacter)
        } else if analyzed.spaces_count() > 0 {
            Some(InvalidMessage::PasswordHasSpace)
        } else if analyzed.length() < PASSWORD_MIN_LEN {
            Some(InvalidMessage::PasswordTooShort)
        } else if analyzed.lowercase_letters_count() == 0 {
            Some(InvalidMessage::PasswordNoLowercaseLetter)
        } else if analyzed.uppercase_letters_count() == 0 {
            Some(InvalidMessage::PasswordNoUppercaseLetter)
        } else if analyzed.numbers_count() == 0 {
            Some(InvalidMessage::PasswordNoNumber)
        } else if analyzed.symbols_count() == 0 {
            Some(InvalidMessage::PasswordNoSymbol)
        } else {
            None
        }
    }
}

fn cmp_consecutive(password: &str) -> bool {
    for c in PASSWD_CMP {
        for i in 0..=c.len() - PASSWORD_MIN_FORBID_ADJACENT_LEN {
            if let Some(slice) = c.get(i..i + PASSWORD_MIN_FORBID_ADJACENT_LEN) {
                if password.contains(slice) {
                    return true;
                }
            }
        }
        let c_rev: String = c.chars().rev().collect();
        for i in 0..=c_rev.len() - PASSWORD_MIN_FORBID_ADJACENT_LEN {
            if let Some(slice) = c_rev.get(i..i + PASSWORD_MIN_FORBID_ADJACENT_LEN) {
                if password.contains(slice) {
                    return true;
                }
            }
        }
    }
    false
}
