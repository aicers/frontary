use std::{cell::RefCell, collections::HashSet, net::Ipv4Addr, rc::Rc, str::FromStr};

use ipnet::Ipv4Net;
use passwords::analyzer;
use yew::{Component, Context};

use super::{
    super::CheckStatus,
    component::{InvalidMessage, Model, Verification},
    user_input::MAX_PER_LAYER,
    ComparisonItem, HostNetworkGroupItem, InputConfig, InputItem, RadioConfig, RadioItem,
    SelectMultipleConfig, SelectMultipleItem, SelectSingleItem, TagItem, VecSelectItem,
};

const PASSWORD_MIN_LEN: usize = if cfg!(feature = "cc-password") { 9 } else { 8 };
const PASSWORD_MIN_FORBID_ADJACENT_LEN: usize = 4; // adjacent keyboard characters
type PropaChildren = Vec<(usize, usize, Rc<RefCell<InputItem>>, Rc<InputConfig>)>;

impl<T> Model<T>
where
    T: Clone + Component + PartialEq,
    <T as Component>::Message: Clone + PartialEq,
{
    pub(super) fn prepare_buffer(&mut self, ctx: &Context<Self>) {
        self.prepare_buffer_recursive(&ctx.props().input_data, &ctx.props().input_conf, 1);
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_buffer_recursive(
        &mut self,
        input_data: &[Rc<RefCell<InputItem>>],
        input_conf: &[Rc<InputConfig>],
        base_index: usize,
    ) {
        input_data
            .iter()
            .enumerate()
            .zip(input_conf.iter())
            .for_each(|((index, input_data), input_conf)| {
                let this_index = base_index + index;
                if let Ok(data) = input_data.try_borrow() {
                    match (&*data, &**input_conf) {
                        (InputItem::HostNetworkGroup(data), InputConfig::HostNetworkGroup(_)) => {
                            self.host_network_to_buffer(this_index, data);
                        }
                        (InputItem::SelectSingle(data), InputConfig::SelectSingle(_)) => {
                            self.select_searchable_to_buffer(this_index, data);
                        }
                        (InputItem::SelectMultiple(data), InputConfig::SelectMultiple(config)) => {
                            self.select_multiple_to_buffer(this_index, data, config);
                        }
                        (InputItem::Tag(data), InputConfig::Tag(_)) => {
                            self.tag_to_buffer(this_index, data);
                        }
                        (InputItem::Comparison(data), InputConfig::Comparison(_)) => {
                            self.comparison_to_buffer(this_index, data);
                        }
                        (InputItem::VecSelect(data), InputConfig::VecSelect(_)) => {
                            self.vec_select_to_buffer(this_index, data);
                        }
                        (InputItem::Group(data), InputConfig::Group(config)) => {
                            for (row, d) in data.iter().enumerate() {
                                for ((col, d), t) in d.iter().enumerate().zip(config.items.iter()) {
                                    if let Ok(d) = d.try_borrow() {
                                        let sub_base_index = this_index * MAX_PER_LAYER;
                                        let item_index =
                                            col + (row + sub_base_index) * MAX_PER_LAYER;

                                        match (&*d, &**t) {
                                            (
                                                InputItem::HostNetworkGroup(data),
                                                InputConfig::HostNetworkGroup(..),
                                            ) => {
                                                self.host_network_to_buffer(item_index, data);
                                            }
                                            (
                                                InputItem::SelectSingle(data),
                                                InputConfig::SelectSingle(..),
                                            ) => {
                                                self.select_searchable_to_buffer(item_index, data);
                                            }
                                            (
                                                InputItem::SelectMultiple(data),
                                                InputConfig::SelectMultiple(config),
                                            ) => {
                                                self.select_multiple_to_buffer(
                                                    item_index, data, config,
                                                );
                                            }
                                            (
                                                InputItem::Comparison(data),
                                                InputConfig::Comparison(..),
                                            ) => {
                                                self.comparison_to_buffer(item_index, data);
                                            }
                                            (
                                                InputItem::VecSelect(data),
                                                InputConfig::VecSelect(..),
                                            ) => {
                                                self.vec_select_to_buffer(item_index, data);
                                            }
                                            (InputItem::Text(_), InputConfig::Text(_))
                                            | (InputItem::Password(_), InputConfig::Password(_))
                                            | (InputItem::Tag(_), InputConfig::Tag(_))
                                            | (
                                                InputItem::Unsigned32(_),
                                                InputConfig::Unsigned32(_),
                                            )
                                            | (InputItem::Float64(_), InputConfig::Float64(_))
                                            | (
                                                InputItem::Percentage(_),
                                                InputConfig::Percentage(_),
                                            )
                                            | (InputItem::Nic(_), InputConfig::Nic(_))
                                            | (InputItem::File(_), InputConfig::File(_))
                                            | (InputItem::Group(_), InputConfig::Group(_))
                                            | (InputItem::Checkbox(_), InputConfig::Checkbox(_))
                                            | (InputItem::Radio(_), InputConfig::Radio(_)) => (), // These don't have buffers.
                                            _ => {
                                                panic!("InputItem and InputConfig is not matched");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        (InputItem::Checkbox(data), InputConfig::Checkbox(config)) => {
                            if let Some(config_children) = config.children.as_ref() {
                                if !data.children().is_empty() {
                                    self.prepare_buffer_recursive(
                                        data.children(),
                                        &config_children.children,
                                        this_index * MAX_PER_LAYER,
                                    );
                                }
                            }
                        }
                        (InputItem::Radio(data), InputConfig::Radio(config)) => {
                            self.radio_to_buffer(this_index, data, config);
                        }
                        (InputItem::Text(_), InputConfig::Text(_))
                        | (InputItem::Password(_), InputConfig::Password(_))
                        | (InputItem::Unsigned32(_), InputConfig::Unsigned32(_))
                        | (InputItem::Float64(_), InputConfig::Float64(_))
                        | (InputItem::Percentage(_), InputConfig::Percentage(_))
                        | (InputItem::Nic(_), InputConfig::Nic(_))
                        | (InputItem::File(_), InputConfig::File(_)) => (), // These don't have buffers.
                        _ => {
                            panic!("InputItem and InputConfig is not matched")
                        }
                    }
                }
            });
    }

    pub(super) fn prepare_preset(&mut self, ctx: &Context<Self>) {
        self.prepare_preset_recursive(&ctx.props().input_data, &ctx.props().input_conf, true, 1);
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_preset_recursive(
        &mut self,
        input_data: &[Rc<RefCell<InputItem>>],
        input_conf: &[Rc<InputConfig>],
        parent_checked: bool,
        base_index: usize,
    ) {
        input_data
            .iter()
            .enumerate()
            .zip(input_conf.iter())
            .for_each(|((index, input_data), input_conf)| {
                if let Ok(mut item) = input_data.try_borrow_mut() {
                    match (&mut *item, &**input_conf) {
                        (InputItem::Text(item), InputConfig::Text(config)) => {
                            if let Some(preset) = &config.preset {
                                if parent_checked {
                                    item.set(preset);
                                }
                            }
                        }
                        (InputItem::SelectSingle(item), InputConfig::SelectSingle(config)) => {
                            if let Some(preset) = &config.preset {
                                if parent_checked {
                                    item.set(preset);
                                    let id = base_index + index;
                                    self.preset_to_buffer_select_single(id, preset);
                                }
                            }
                        }
                        (InputItem::SelectMultiple(item), InputConfig::SelectMultiple(config)) => {
                            if let Some(preset) = &config.preset {
                                if parent_checked {
                                    item.set(preset);
                                    let id = base_index + index;
                                    self.preset_to_buffer_select_multiple(id, preset);
                                }
                            }
                        }
                        (InputItem::Unsigned32(item), InputConfig::Unsigned32(config)) => {
                            if let Some(preset) = &config.preset {
                                if parent_checked {
                                    item.set(*preset);
                                }
                            }
                        }
                        (InputItem::Float64(item), InputConfig::Float64(config)) => {
                            if let Some(preset) = &config.preset {
                                if parent_checked {
                                    item.set(*preset);
                                }
                            }
                        }
                        (InputItem::Percentage(item), InputConfig::Percentage(config)) => {
                            if let Some(preset) = &config.preset {
                                if parent_checked {
                                    item.set(*preset);
                                }
                            }
                        }
                        (InputItem::VecSelect(item), InputConfig::VecSelect(config)) => {
                            if let Some(preset) = &config.preset {
                                if parent_checked {
                                    item.set(preset);
                                    let id = base_index + index;
                                    self.preset_to_buffer_vec_select(id, preset);
                                }
                            }
                        }
                        (InputItem::Checkbox(data), InputConfig::Checkbox(config)) => {
                            if let Some(preset) = config.preset {
                                if parent_checked {
                                    data.set_status(preset);
                                }
                            }
                            if let Some(config_children) = config.children.as_ref() {
                                if !data.children().is_empty()
                                    && data.status() != CheckStatus::Unchecked
                                {
                                    self.prepare_preset_recursive(
                                        data.children(),
                                        &config_children.children,
                                        data.status() == CheckStatus::Checked
                                            || data.status() == CheckStatus::Indeterminate,
                                        (base_index + index) * MAX_PER_LAYER,
                                    );
                                }
                            }
                        }
                        (InputItem::Radio(data), InputConfig::Radio(config)) => {
                            if let Some(preset) = &config.preset {
                                if parent_checked {
                                    data.set_selected(preset.clone());
                                    for (sub_index, (data, config)) in data
                                        .children_group()
                                        .iter()
                                        .zip(config.children_group.iter())
                                        .enumerate()
                                    {
                                        if let Some(config) = config {
                                            self.prepare_preset_recursive(
                                                data,
                                                config,
                                                parent_checked,
                                                ((base_index + index) * MAX_PER_LAYER + sub_index)
                                                    * MAX_PER_LAYER,
                                            );
                                        }
                                    }
                                    self.preset_to_buffer_radio(base_index + index, preset);
                                }
                            }
                        }
                        (InputItem::Password(_), InputConfig::Password(_))
                        | (InputItem::HostNetworkGroup(_), InputConfig::HostNetworkGroup(_))
                        | (InputItem::Tag(_), InputConfig::Tag(_))
                        | (InputItem::Nic(_), InputConfig::Nic(_))
                        | (InputItem::File(_), InputConfig::File(_))
                        | (InputItem::Comparison(_), InputConfig::Comparison(_))
                        | (InputItem::Group(_), InputConfig::Group(_)) => (), // These don't have presets
                        _ => {
                            panic!("InputItem and InputConfig is not matched");
                        }
                    }
                }
            });
    }

    pub(super) fn preset_to_buffer_radio(&mut self, id: usize, preset: &String) {
        if let Some(buffer) = self.radio_buffer.get(&id) {
            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                buffer.clone_from(preset);
            }
        }
    }

    pub(super) fn preset_to_buffer_select_single(&mut self, id: usize, preset: &str) {
        if let Some(buffer) = self.select_searchable_buffer.get(&id) {
            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                if !preset.is_empty() {
                    let mut value: HashSet<String> = HashSet::new();
                    value.insert(preset.to_string());
                    *buffer = Some(value);
                }
            }
        }
    }

    pub(super) fn preset_to_buffer_select_multiple(&mut self, id: usize, preset: &[String]) {
        if let Some(buffer) = self.select_searchable_buffer.get(&id) {
            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                if !preset.is_empty() {
                    *buffer = Some(preset.iter().cloned().collect::<HashSet<String>>());
                }
            }
        }
    }

    pub(super) fn preset_to_buffer_vec_select(&mut self, id: usize, preset: &[HashSet<String>]) {
        if let Some(buffer) = self.vec_select_buffer.get(&id) {
            for (b, p) in buffer.iter().zip(preset.iter()) {
                if let Ok(mut b) = b.try_borrow_mut() {
                    *b = Some(p.clone());
                }
            }
        }
    }

    pub(super) fn decide_required_all(&mut self, ctx: &Context<Self>) -> bool {
        self.decide_required_all_recursive(
            ctx,
            &ctx.props().input_data,
            &ctx.props().input_conf,
            1,
            true,
        )
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn decide_required_all_recursive(
        &mut self,
        ctx: &Context<Self>,
        input_data: &[Rc<RefCell<InputItem>>],
        input_conf: &[Rc<InputConfig>],
        base_index: usize,
        parent_checked: bool,
    ) -> bool {
        let mut required = Vec::<bool>::new();

        input_data
            .iter()
            .enumerate()
            .zip(input_conf.iter())
            .for_each(|((index, input_data), input_conf)| {
                if let Ok(item) = input_data.try_borrow() {
                    if parent_checked && (*input_conf).required() {
                        let empty = match &(*item) {
                            InputItem::Text(_)
                            | InputItem::SelectSingle(_)
                            | InputItem::SelectMultiple(_)
                            | InputItem::Unsigned32(_)
                            | InputItem::Float64(_)
                            | InputItem::Percentage(_)
                            | InputItem::File(_)
                            | InputItem::Comparison(_)
                            | InputItem::Tag(_)
                            | InputItem::Group(_)
                            | InputItem::Checkbox(_)
                            | InputItem::Radio(_) => item.is_empty(),
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
                            InputItem::VecSelect(s) => {
                                s.is_empty()
                                    || if let InputConfig::VecSelect(config) = &**input_conf {
                                        s.iter().zip(config.items_ess_list.iter()).any(
                                            |(selected, ess)| selected.is_empty() && ess.required,
                                        )
                                    } else {
                                        true
                                    }
                            }
                        };
                        if empty {
                            self.required_msg.insert(base_index + index);
                            required.push(true);
                        }
                    }
                    if let (InputItem::Group(group), InputConfig::Group(config)) =
                        (&(*item), &**input_conf)
                    {
                        let required = config
                            .items
                            .iter()
                            .map(|t| match &(**t) {
                                InputConfig::Text(_)
                                | InputConfig::HostNetworkGroup(_)
                                | InputConfig::SelectSingle(_)
                                | InputConfig::SelectMultiple(_)
                                | InputConfig::Unsigned32(_)
                                | InputConfig::Float64(_)
                                | InputConfig::Percentage(_)
                                | InputConfig::Comparison(_)
                                | InputConfig::VecSelect(_) => t.required(),
                                InputConfig::Password(_)
                                | InputConfig::Tag(_)
                                | InputConfig::Nic(_)
                                | InputConfig::File(_)
                                | InputConfig::Group(_)
                                | InputConfig::Checkbox(_)
                                | InputConfig::Radio(_) => {
                                    panic!("Input Group does not support some items such as Password, Tag, Nic, File, Group, Checkbox, and Radio.")
                                }
                            })
                            .collect::<Vec<bool>>();
                        // in the case of VecSelect or Comparison
                        // Some(true) means all of sub elements empty
                        // Some(false) means all of sub elements not empty
                        // None means not all of any of sub elements empty but at least one empty
                        let empty = group
                            .iter()
                            .enumerate()
                            .map(|(row_index, row)| {
                                row.iter()
                                    .enumerate()
                                    .filter_map(|(col_index, col)| {
                                        if let Ok(col) = col.try_borrow() {
                                            match &*col {
                                                InputItem::Text(_)
                                                | InputItem::HostNetworkGroup(_)
                                                | InputItem::SelectSingle(_)
                                                | InputItem::SelectMultiple(_)
                                                | InputItem::Unsigned32(_)
                                                | InputItem::Float64(_)
                                                | InputItem::Percentage(_) => Some(Some(col.is_empty())),
                                                InputItem::Comparison(v) => {
                                                    if v.is_some() {
                                                        Some(Some(false))
                                                    } else if self
                                                        .comparison_kind(
                                                            col_index
                                                                + (row_index
                                                                    + (base_index + index)
                                                                        * MAX_PER_LAYER)
                                                                    * MAX_PER_LAYER,
                                                        )
                                                        .is_some()
                                                    {
                                                        Some(None)
                                                    } else {
                                                        Some(Some(true))
                                                    }
                                                }
                                                InputItem::VecSelect(v) => {
                                                    if v.iter().all(HashSet::is_empty) {
                                                        Some(Some(true))
                                                    } else if v.iter().all(|d| !d.is_empty()) {
                                                        Some(Some(false))
                                                    } else {
                                                        Some(None)
                                                    }
                                                }
                                                InputItem::Password(_)
                                                | InputItem::Tag(_)
                                                | InputItem::Nic(_)
                                                | InputItem::File(_)
                                                | InputItem::Group(_)
                                                | InputItem::Checkbox(_)
                                                | InputItem::Radio(_) => {
                                                    panic!("Input Group does not support some items such as Password, Tag, Nic, File, Group, Checkbox, and Radio.")
                                                },
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<Option<bool>>>()
                            })
                            .collect::<Vec<Vec<Option<bool>>>>();
                        let sub_base_index = (base_index + index) * MAX_PER_LAYER;
                        for (row_index, row) in empty.iter().enumerate() {
                            let all_empty = row.iter().all(|x| x.map_or(false, |x| x));
                            if !all_empty || config.ess.required && row_index == 0 {
                                let base_index = (row_index + sub_base_index) * MAX_PER_LAYER;
                                for ((col_index, data_empty), data_required) in
                                    row.iter().enumerate().zip(required.iter())
                                {
                                    if data_empty.map_or(true, |x| x) && *data_required {
                                        self.required_msg.insert(base_index + col_index);
                                    }
                                }
                            }
                        }
                    } else if let (InputItem::Checkbox(data), InputConfig::Checkbox(config)) =
                        (&(*item), &**input_conf)
                    {
                        if let Some(config_children) = config.children.as_ref() {
                            if !data.children().is_empty()
                                && self.decide_required_all_recursive(
                                    ctx,
                                    data.children(),
                                    &config_children.children,
                                    (base_index + index) * MAX_PER_LAYER,
                                    data.status() == CheckStatus::Checked,
                                )
                            {
                                required.push(true);
                            }
                        }
                    }
                }
            });

        !required.is_empty() || !self.required_msg.is_empty()
    }

    pub(super) fn verify(&mut self, ctx: &Context<Self>) -> bool {
        self.verify_recursive(&ctx.props().input_data, &ctx.props().input_conf, 1, true)
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn verify_recursive(
        &mut self,
        input_data: &[Rc<RefCell<InputItem>>],
        input_conf: &[Rc<InputConfig>],
        base_index: usize,
        parent_checked: bool,
    ) -> bool {
        let mut rtn = true;

        input_data
            .iter()
            .enumerate()
            .zip(input_conf.iter())
            .for_each(|((index, input_data), input_conf)| {
                if let Ok(input_data) = input_data.try_borrow() {
                    // HIGHTLIGHT: All kinds are not necessarily to be verified.
                    // HIGHTLIGHT: Since HostNetworkGroup items were verified yet, they don't need to be verified here.
                    match (&*input_data, &**input_conf) {
                        (InputItem::Unsigned32(value), InputConfig::Unsigned32(config)) => {
                            if let Some(value) = value.as_ref() {
                                if parent_checked {
                                    if *value >= config.min && *value <= config.max {
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
                        }
                        (InputItem::Percentage(value), InputConfig::Percentage(config)) => {
                            if let Some(value) = value.as_ref() {
                                if parent_checked {
                                    if *value >= config.min.unwrap_or(0.0)
                                        && *value <= config.max.unwrap_or(1.0)
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
                        }
                        (InputItem::Password(pwd), InputConfig::Password(_)) => {
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
                        (InputItem::Nic(nics), InputConfig::Nic(_)) => {
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
                        (InputItem::Checkbox(data), InputConfig::Checkbox(config)) => {
                            if let Some(config_children) = config.children.as_ref() {
                                if data.status() != CheckStatus::Unchecked
                                    && !data.children().is_empty()
                                    && !self.verify_recursive(
                                        data.children(),
                                        &config_children.children,
                                        (base_index + index) * MAX_PER_LAYER,
                                        data.status() == CheckStatus::Checked,
                                    )
                                {
                                    rtn = false;
                                }
                            }
                        }
                        (_, _) => (),
                    }
                }
            });

        rtn
    }

    pub(super) fn trim_nic(ctx: &Context<Self>) {
        Self::trim_nic_recursive(&ctx.props().input_data, &ctx.props().input_conf);
    }

    pub(super) fn trim_nic_recursive(
        input_data: &[Rc<RefCell<InputItem>>],
        input_conf: &[Rc<InputConfig>],
    ) {
        input_data
            .iter()
            .zip(input_conf.iter())
            .for_each(|(input_data, input_conf)| {
                if let Ok(mut input_data) = input_data.try_borrow_mut() {
                    if let InputItem::Nic(nics) = &mut *input_data {
                        nics.retain(|n| {
                            !n.name.is_empty() && !n.interface.is_empty() && !n.gateway.is_empty()
                        });
                    }
                }
                if let Ok(input_data) = input_data.try_borrow() {
                    if let (InputItem::Checkbox(data), InputConfig::Checkbox(config)) =
                        (&*input_data, &**input_conf)
                    {
                        if let Some(config_children) = config.children.as_ref() {
                            if data.status() != CheckStatus::Unchecked
                                && !data.children().is_empty()
                            {
                                Self::trim_nic_recursive(
                                    data.children(),
                                    &config_children.children,
                                );
                            }
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
        let mut propagate: Vec<(usize, Rc<RefCell<InputItem>>, Rc<InputConfig>)> = Vec::new();

        ctx.props()
            .input_data
            .iter()
            .enumerate()
            .zip(ctx.props().input_conf.iter())
            .for_each(|((index, input_data), input_conf)| {
                if let Ok(item) = input_data.try_borrow() {
                    if let (InputItem::Checkbox(_), InputConfig::Checkbox(_))
                    | (InputItem::Radio(_), InputConfig::Radio(_)) = (&(*item), &**input_conf)
                    {
                        propagate.push((index, Rc::clone(input_data), Rc::clone(input_conf)));
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
        input_conf: &Rc<InputConfig>,
        checked: Option<CheckStatus>, // parent of `this_checked`, that is, `pos`
        layer_index: usize,
        base_index: usize,
    ) -> Option<CheckStatus> {
        let this_checked = if Rc::ptr_eq(click, pos) {
            if let Ok(mut click) = click.try_borrow_mut() {
                if let (InputItem::Checkbox(data), InputConfig::Checkbox(config)) =
                    (&mut *click, &**input_conf)
                {
                    match data.status() {
                        CheckStatus::Checked | CheckStatus::Indeterminate => {
                            if config.always != Some(CheckStatus::Checked)
                                && config.always != Some(CheckStatus::Indeterminate)
                            {
                                data.set_status(checked.unwrap_or(CheckStatus::Unchecked));
                            }
                            Some(data.status())
                        }
                        CheckStatus::Unchecked => {
                            if config.always != Some(CheckStatus::Unchecked) {
                                if let Some(always) = config.always {
                                    data.set_status(always);
                                } else {
                                    data.set_status(checked.unwrap_or(CheckStatus::Checked));
                                }
                            }
                            Some(data.status())
                        }
                    }
                } else if let (InputItem::Radio(data), InputConfig::Radio(_)) =
                    (&mut *click, &**input_conf)
                {
                    if data.is_empty() {
                        Some(CheckStatus::Unchecked)
                    } else {
                        Some(CheckStatus::Checked)
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else if let Some(checked) = checked {
            if let Ok(mut pos) = pos.try_borrow_mut() {
                if let (InputItem::Checkbox(data), InputConfig::Checkbox(conf)) =
                    (&mut *pos, &**input_conf)
                {
                    if let Some(always) = conf.always {
                        data.set_status(always);
                    } else if let Some(preset) = conf.preset {
                        data.set_status(preset);
                    } else {
                        data.set_status(checked);
                    }
                    Some(data.status())
                } else if let (InputItem::Radio(data), InputConfig::Radio(_)) =
                    (&mut *pos, &**input_conf)
                {
                    if checked == CheckStatus::Unchecked {
                        data.set_selected(String::new());
                    }
                    if data.is_empty() {
                        Some(CheckStatus::Unchecked)
                    } else {
                        Some(CheckStatus::Checked)
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let mut propa_children: PropaChildren = Vec::new();
        if let Ok(mut pos) = pos.try_borrow_mut() {
            let children = if let (InputItem::Checkbox(data), InputConfig::Checkbox(config)) =
                (&mut *pos, &**input_conf)
            {
                if data.status() == CheckStatus::Checked
                    || data.status() == CheckStatus::Indeterminate
                {
                    config.children.as_ref().map(|config_children| {
                        (None, data.children_mut(), &config_children.children)
                    })
                } else {
                    None
                }
            } else if let (InputItem::Radio(data), InputConfig::Radio(config)) =
                (&mut *pos, &**input_conf)
            {
                let checked_index = config
                    .options
                    .iter()
                    .position(|o| data.selected() == o.to_string());
                if let Some(checked_index) = checked_index {
                    if let (Some(children), Some(Some(config_children))) = (
                        data.children_get_mut(checked_index),
                        config.children_group.get(checked_index),
                    ) {
                        Some((Some(checked_index), children, config_children))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some((radio, children, config_children)) = children {
                if children.is_empty() {
                    *children = InputItem::default_items_from_config(config_children);
                }
                for (index, child) in children.iter().enumerate() {
                    if let (Ok(mut c), Some(t)) =
                        (child.try_borrow_mut(), config_children.get(index))
                    {
                        let propa_index = if let Some(radio) = radio {
                            (layer_index + base_index) * MAX_PER_LAYER + radio
                        } else {
                            layer_index + base_index
                        };
                        match (&mut (*c), &**t) {
                            // HIGHLIGHT: `preset` should be set even when `this_checked` is
                            // `Unchecked`, because `this_checked`` may be changed later depending
                            // on its children.
                            (InputItem::Text(user), InputConfig::Text(config)) => {
                                if user.is_empty() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(preset) = &config.preset {
                                        user.set(preset);
                                    }
                                }
                            }
                            (
                                InputItem::HostNetworkGroup(user),
                                InputConfig::HostNetworkGroup(_),
                            ) => {
                                if user.is_empty() || this_checked == Some(CheckStatus::Unchecked) {
                                    *user = HostNetworkGroupItem::default();
                                    self.host_network_to_buffer(
                                        (propa_index) * MAX_PER_LAYER + index,
                                        user,
                                    );
                                }
                            }
                            (InputItem::SelectSingle(user), InputConfig::SelectSingle(config)) => {
                                if user.is_none() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(preset) = &config.preset {
                                        user.set(preset);
                                        self.select_searchable_to_buffer(
                                            (propa_index) * MAX_PER_LAYER + index,
                                            user,
                                        );
                                    }
                                }
                            }
                            (
                                InputItem::SelectMultiple(user),
                                InputConfig::SelectMultiple(config),
                            ) => {
                                if user.is_empty() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(preset) = &config.preset {
                                        user.set(preset);
                                        self.select_multiple_to_buffer(
                                            (propa_index) * MAX_PER_LAYER + index,
                                            user,
                                            config,
                                        );
                                    }
                                }
                            }
                            (InputItem::Unsigned32(user), InputConfig::Unsigned32(config)) => {
                                if user.is_none() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(preset) = &config.preset {
                                        user.set(*preset);
                                    }
                                }
                            }
                            (InputItem::Float64(user), InputConfig::Float64(config)) => {
                                if user.is_none() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(preset) = &config.preset {
                                        user.set(*preset);
                                    }
                                }
                            }
                            (InputItem::Percentage(user), InputConfig::Percentage(config)) => {
                                if user.is_none() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(preset) = &config.preset {
                                        user.set(*preset);
                                    }
                                }
                            }
                            (InputItem::VecSelect(user), InputConfig::VecSelect(config)) => {
                                if user.is_empty() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(preset) = &config.preset {
                                        user.set(preset);
                                    }
                                    self.vec_select_to_buffer(
                                        (propa_index) * MAX_PER_LAYER + index,
                                        user,
                                    );
                                }
                            }
                            (InputItem::Checkbox(_), InputConfig::Checkbox(_)) => {
                                // HIGHLIGHT: This should be called regardless of the user.status(),
                                // because it might be changed according to the status of children.
                                propa_children.push((
                                    propa_index,
                                    index,
                                    Rc::clone(child),
                                    Rc::clone(t),
                                ));
                            }
                            (InputItem::Radio(user), InputConfig::Radio(config)) => {
                                // HIGHLIGHT: Should this be handled after propagation? I don't
                                // think so. But, recheck this later.
                                if user.is_empty()
                                    && (this_checked == Some(CheckStatus::Checked)
                                        || this_checked == Some(CheckStatus::Indeterminate))
                                {
                                    if let Some(preset) = &config.preset {
                                        if user.selected() != preset {
                                            user.set_selected(preset.clone());
                                            self.radio_to_buffer(
                                                propa_index * MAX_PER_LAYER + index,
                                                user,
                                                config,
                                            );
                                        }
                                    }
                                }
                                // HIGHLIGHT: This CAN be called only if user.is_empty() is true,
                                // because the radio status is not affected by the status of children.
                                if !user.is_empty() {
                                    propa_children.push((
                                        propa_index,
                                        index,
                                        Rc::clone(child),
                                        Rc::clone(t),
                                    ));
                                }
                            }
                            (InputItem::Password(_), InputConfig::Password(_))
                            | (InputItem::Tag(_), InputConfig::Tag(_))
                            | (InputItem::Nic(_), InputConfig::Nic(_))
                            | (InputItem::File(_), InputConfig::File(_))
                            | (InputItem::Comparison(_), InputConfig::Comparison(_))
                            | (InputItem::Group(_), InputConfig::Group(_)) => {
                                panic!("Children items do not include Password, Tag, Nic, File, Comparison, and Group.")
                            }
                            (_, _) => {
                                panic!("InputItem and InputConfig is not matched");
                            }
                        }
                    }
                }
            }
        }
        let mut rtn_checked: Vec<Option<CheckStatus>> = Vec::new();

        for (index, sub_index, child, config_child) in &propa_children {
            rtn_checked.push(self.propagate_checkbox_recursive(
                click,
                child,
                config_child,
                this_checked,
                *sub_index,
                *index * MAX_PER_LAYER,
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
            match &mut (*pos) {
                InputItem::Checkbox(data) => {
                    if let Some(updated_checked) = updated_checked {
                        data.set_status(updated_checked);
                    }
                    Some(data.status())
                }
                InputItem::Radio(data) => {
                    if data.is_empty() {
                        Some(CheckStatus::Unchecked)
                    } else {
                        Some(CheckStatus::Checked)
                    }
                }
                _ => None,
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
            &ctx.props().input_conf,
            1,
            true,
        );
    }

    pub(super) fn reset_veri_host_network_recursive(
        &mut self,
        input_data: &[Rc<RefCell<InputItem>>],
        input_conf: &[Rc<InputConfig>],
        base_index: usize,
        parent_checked: bool,
    ) {
        input_data
            .iter()
            .enumerate()
            .zip(input_conf.iter())
            .for_each(|((index, input_data), input_conf)| {
                if let Ok(input_data) = input_data.try_borrow() {
                    if parent_checked {
                        if let (InputItem::HostNetworkGroup(_), InputConfig::HostNetworkGroup(_)) =
                            (&*input_data, &**input_conf)
                        {
                            self.verification_host_network
                                .insert(base_index + index, None);
                        }
                    }

                    if let (InputItem::Checkbox(data), InputConfig::Checkbox(config)) =
                        (&*input_data, &**input_conf)
                    {
                        if let Some(config_children) = config.children.as_ref() {
                            if data.status() != CheckStatus::Unchecked
                                && !data.children().is_empty()
                            {
                                self.reset_veri_host_network_recursive(
                                    data.children(),
                                    &config_children.children,
                                    (base_index + index) * MAX_PER_LAYER,
                                    data.status() == CheckStatus::Checked,
                                );
                            }
                        }
                    }
                }
            });
    }

    fn host_network_to_buffer(&mut self, index: usize, data: &HostNetworkGroupItem) {
        self.host_network_buffer
            .insert(index, Rc::new(RefCell::new(data.into_inner())));
    }

    fn select_searchable_to_buffer(&mut self, index: usize, data: &SelectSingleItem) {
        let mut buf = HashSet::new();
        if let Some(data) = data.as_ref() {
            buf.insert(data.clone());
        }
        self.select_searchable_buffer
            .insert(index, Rc::new(RefCell::new(Some(buf))));
    }

    fn select_multiple_to_buffer(
        &mut self,
        index: usize,
        data: &SelectMultipleItem,
        config: &SelectMultipleConfig,
    ) {
        if config.all {
            self.select_searchable_buffer
                .insert(index, Rc::new(RefCell::new(None)));
        } else {
            self.select_searchable_buffer
                .insert(index, Rc::new(RefCell::new(Some(data.into_inner()))));
        }
    }

    fn tag_to_buffer(&mut self, index: usize, data: &TagItem) {
        self.tag_buffer
            .insert(index, Rc::new(RefCell::new(data.into_inner())));
    }

    fn comparison_to_buffer(&mut self, index: usize, data: &ComparisonItem) {
        let (mut buf, mut kind) = (HashSet::new(), HashSet::new());
        let (first, second) = if let Some(data) = data.as_ref() {
            buf.insert(data.value_kind().to_string());
            kind.insert(data.comparison_kind().to_string());
            (Some(data.first()), data.second())
        } else {
            (None, None)
        };
        self.comparison_value_kind_buffer
            .insert(index, Rc::new(RefCell::new(Some(buf))));
        self.comparison_value_cmp_buffer
            .insert(index, Rc::new(RefCell::new(Some(kind))));
        self.comparison_value_buffer.insert(
            index,
            (Rc::new(RefCell::new(first)), Rc::new(RefCell::new(second))),
        );
    }

    fn vec_select_to_buffer(&mut self, index: usize, data: &VecSelectItem) {
        self.vec_select_buffer.insert(
            index,
            data.iter()
                .map(|d| Rc::new(RefCell::new(Some(d.clone()))))
                .collect::<Vec<_>>(),
        );
    }

    fn radio_to_buffer(&mut self, index: usize, data: &RadioItem, _: &RadioConfig) {
        self.radio_buffer
            .insert(index, Rc::new(RefCell::new(data.selected().to_string())));
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
