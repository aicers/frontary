use std::{cell::RefCell, collections::HashSet, net::Ipv4Addr, rc::Rc, str::FromStr};

use ipnet::Ipv4Net;
use passwords::analyzer;
use yew::{Component, Context};

use super::{
    super::CheckStatus,
    component::{InvalidMessage, Model, Verification},
    user_input::MAX_PER_LAYER,
    GroupItem, InputConfig, InputItem,
};

const PASSWORD_MIN_LEN: usize = if cfg!(feature = "cc-password") { 9 } else { 8 };
const PASSWORD_MIN_FORBID_ADJACENT_LEN: usize = 4; // adjacent keyboard characters

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
                            self.host_network_buffer
                                .insert(this_index, Rc::new(RefCell::new(data.into_inner())));
                        }
                        (InputItem::SelectMultiple(data), InputConfig::SelectMultiple(config)) => {
                            if config.all {
                                self.select_searchable_buffer
                                    .insert(this_index, Rc::new(RefCell::new(None)));
                            } else {
                                self.select_searchable_buffer.insert(
                                    this_index,
                                    Rc::new(RefCell::new(Some(data.into_inner()))),
                                );
                            }
                        }
                        (InputItem::SelectSingle(data), InputConfig::SelectSingle(_)) => {
                            let mut buf = HashSet::new();
                            if let Some(data) = data.as_ref() {
                                buf.insert(data.clone());
                            }
                            self.select_searchable_buffer
                                .insert(this_index, Rc::new(RefCell::new(Some(buf))));
                        }
                        (InputItem::Tag(data), InputConfig::Tag(_)) => {
                            self.tag_buffer
                                .insert(this_index, Rc::new(RefCell::new(data.into_inner())));
                        }
                        (
                            InputItem::Radio(data_option, data_children_group),
                            InputConfig::Radio(config),
                        ) => {
                            self.radio_buffer
                                .insert(this_index, Rc::new(RefCell::new(data_option.clone())));

                            if let Some(index) = config
                                .options
                                .iter()
                                .position(|vs| data_option == &vs.to_string())
                            {
                                if let (Some(data_children), Some(Some(config_children))) = (
                                    data_children_group.get(index),
                                    config.children_group.get(index),
                                ) {
                                    self.prepare_buffer_recursive(
                                        data_children,
                                        config_children,
                                        this_index * MAX_PER_LAYER,
                                    );
                                }
                            }
                        }
                        (InputItem::CheckBox(_, data_children), InputConfig::CheckBox(config)) => {
                            if let Some((_, config_children)) = config.children.as_ref() {
                                if !data_children.is_empty() {
                                    self.prepare_buffer_recursive(
                                        data_children,
                                        config_children,
                                        this_index * MAX_PER_LAYER,
                                    );
                                }
                            }
                        }
                        (InputItem::Group(data), InputConfig::Group(config)) => {
                            for (row, d) in data.iter().enumerate() {
                                for ((col, d), t) in d.iter().enumerate().zip(config.items.iter()) {
                                    if let Ok(d) = d.try_borrow() {
                                        let sub_base_index = this_index * MAX_PER_LAYER;

                                        match (&*d, &**t) {
                                            (
                                                InputItem::SelectSingle(data),
                                                InputConfig::SelectSingle(..),
                                            ) => {
                                                let mut buf = HashSet::new();
                                                if let Some(data) = data.as_ref() {
                                                    buf.insert(data.clone());
                                                }
                                                self.select_searchable_buffer.insert(
                                                    col + (row + sub_base_index) * MAX_PER_LAYER,
                                                    Rc::new(RefCell::new(Some(buf))),
                                                );
                                            }
                                            (
                                                InputItem::Comparison(data),
                                                InputConfig::Comparison(..),
                                            ) => {
                                                let (mut buf, mut kind) =
                                                    (HashSet::new(), HashSet::new());
                                                let (first, second) = if let Some(data) =
                                                    data.as_ref()
                                                {
                                                    buf.insert(data.value_kind().to_string());
                                                    kind.insert(data.comparison_kind().to_string());
                                                    (Some(data.first()), data.second())
                                                } else {
                                                    (None, None)
                                                };
                                                self.comparison_value_kind_buffer.insert(
                                                    col + (row + sub_base_index) * MAX_PER_LAYER,
                                                    Rc::new(RefCell::new(Some(buf))),
                                                );
                                                self.comparison_value_cmp_buffer.insert(
                                                    col + (row + sub_base_index) * MAX_PER_LAYER,
                                                    Rc::new(RefCell::new(Some(kind))),
                                                );
                                                self.comparison_value_buffer.insert(
                                                    col + (row + sub_base_index) * MAX_PER_LAYER,
                                                    (
                                                        Rc::new(RefCell::new(first)),
                                                        Rc::new(RefCell::new(second)),
                                                    ),
                                                );
                                            }
                                            (
                                                InputItem::VecSelect(data),
                                                InputConfig::VecSelect(..),
                                            ) => {
                                                self.vec_select_buffer.insert(
                                                    col + (row + sub_base_index) * MAX_PER_LAYER,
                                                    data.iter()
                                                        .map(|d| {
                                                            Rc::new(RefCell::new(Some(d.clone())))
                                                        })
                                                        .collect::<Vec<_>>(),
                                                );
                                            }
                                            _ => {}
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
        self.prepare_default_recursive(&ctx.props().input_data, &ctx.props().input_conf, true, 1);
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_default_recursive(
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
                        (InputItem::Text(_), InputConfig::Text(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                }
                            }
                        }
                        (InputItem::Password(_), InputConfig::Password(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                }
                            }
                        }
                        (InputItem::HostNetworkGroup(_), InputConfig::HostNetworkGroup(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                }
                            }
                        }
                        (InputItem::Tag(_), InputConfig::Tag(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                }
                            }
                        }
                        (InputItem::Unsigned32(_), InputConfig::Unsigned32(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                }
                            }
                        }
                        (InputItem::Float64(_), InputConfig::Float64(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                }
                            }
                        }
                        (InputItem::Percentage(_), InputConfig::Percentage(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                }
                            }
                        }
                        (InputItem::Nic(_), InputConfig::Nic(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                }
                            }
                        }
                        (InputItem::File(_), InputConfig::File(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                }
                            }
                        }
                        (
                            InputItem::Radio(option, data_children_group),
                            InputConfig::Radio(config),
                        ) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    if let InputItem::Radio(default_option, _) = default {
                                        option.clone_from(default_option);
                                        let checked_index = config
                                            .options
                                            .iter()
                                            .position(|o| &o.to_string() == default_option);
                                        if let Some(checked_index) = checked_index {
                                            if let (
                                                Some(data_children),
                                                Some(Some(config_children)),
                                            ) = (
                                                data_children_group.get(checked_index),
                                                config.children_group.get(checked_index),
                                            ) {
                                                self.prepare_default_recursive(
                                                    data_children,
                                                    config_children,
                                                    parent_checked,
                                                    (base_index + index) * MAX_PER_LAYER,
                                                );
                                            }
                                        }
                                    }
                                    let id = base_index + index;
                                    self.default_to_buffer_radio(id, default);
                                }
                            }
                        }
                        (
                            InputItem::CheckBox(checked, data_children),
                            InputConfig::CheckBox(config),
                        ) => {
                            if let Some(InputItem::CheckBox(c, _)) = &config.ess.default {
                                if parent_checked {
                                    *checked = *c;
                                }
                            }
                            if let Some((_, config_children)) = config.children.as_ref() {
                                if !data_children.is_empty() && *checked != CheckStatus::Unchecked {
                                    self.prepare_default_recursive(
                                        data_children,
                                        config_children,
                                        *checked == CheckStatus::Checked
                                            || *checked == CheckStatus::Indeterminate,
                                        (base_index + index) * MAX_PER_LAYER,
                                    );
                                }
                            }
                        }
                        (InputItem::SelectSingle(_), InputConfig::SelectSingle(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                    let id = base_index + index;
                                    self.default_to_buffer_select_single(id, default);
                                }
                            }
                        }
                        (InputItem::SelectMultiple(_), InputConfig::SelectMultiple(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                    let id = base_index + index;
                                    self.default_to_buffer_select_multiple(id, default);
                                }
                            }
                        }
                        (InputItem::VecSelect(_), InputConfig::VecSelect(config)) => {
                            if let Some(default) = &config.ess.default {
                                if parent_checked {
                                    *item = default.clone();
                                    let id = base_index + index;
                                    self.default_to_buffer_vec_select(id, default);
                                }
                            }
                        }
                        (InputItem::Group(_), InputConfig::Group(config)) => {
                            if let Some(InputItem::Group(default)) = &config.ess.default {
                                if let Some(default) = default.first() {
                                    if let Some(copy_default) = Self::copy_default(default) {
                                        if parent_checked {
                                            *item = InputItem::Group(GroupItem::new(vec![
                                                copy_default,
                                            ]));
                                        }
                                    }
                                }
                            }
                        }
                        // TODO: InputItem::Comparison isn't implemented yet
                        _ => (),
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
            .map(|d| Rc::new(RefCell::new(d)))
            .collect::<Vec<Rc<RefCell<InputItem>>>>();
        if default.len() == copy_default.len() {
            Some(copy_default)
        } else {
            None
        }
    }

    pub(super) fn default_to_buffer_radio(&mut self, id: usize, default: &InputItem) {
        if let (InputItem::Radio(default, _), Some(buffer)) = (default, self.radio_buffer.get(&id))
        {
            if let Ok(mut buffer) = buffer.try_borrow_mut() {
                buffer.clone_from(default);
            }
        }
    }

    pub(super) fn default_to_buffer_select_single(&mut self, id: usize, default: &InputItem) {
        if let (InputItem::SelectSingle(default), Some(buffer)) =
            (default, self.select_searchable_buffer.get(&id))
        {
            if let Some(default) = default.selected() {
                if let Ok(mut buffer) = buffer.try_borrow_mut() {
                    if !default.is_empty() {
                        let mut value: HashSet<String> = HashSet::new();
                        value.insert(default.to_string());
                        *buffer = Some(value);
                    }
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
                    *buffer = Some(default.into_inner());
                }
            }
        }
    }

    pub(super) fn default_to_buffer_vec_select(&mut self, id: usize, default: &InputItem) {
        if let (InputItem::VecSelect(default), Some(buffer)) =
            (default, self.vec_select_buffer.get(&id))
        {
            for (b, d) in buffer.iter().zip(default.iter()) {
                if let Ok(mut b) = b.try_borrow_mut() {
                    *b = Some(d.clone());
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
                            InputItem::Text(data) => data.is_empty(),
                            InputItem::Radio(option, _) => option.is_empty(),
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
                            InputItem::File(file) => file.content().is_empty(),
                            InputItem::Comparison(cmp) => cmp.is_none(),
                            InputItem::Tag(_) | InputItem::Group(_) => false,
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
                            .filter_map(|t| match &(**t) {
                                InputConfig::Text(config) => Some(config.ess.required),
                                InputConfig::SelectSingle(config) => Some(config.ess.required),
                                // TODO: SelectMultiple isn't needed?
                                InputConfig::VecSelect(config) => Some(config.ess.required),
                                InputConfig::Unsigned32(config) => Some(config.ess.required),
                                InputConfig::Float64(config) => Some(config.ess.required),
                                InputConfig::Comparison(config) => Some(config.ess.required),
                                // TODO: If not supported, how about a panic?
                                _ => None,
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
                                                InputItem::Text(v) => Some(Some(v.is_empty())),
                                                InputItem::Unsigned32(v) => Some(Some(v.is_none())),
                                                InputItem::Float64(v) => Some(Some(v.is_none())),
                                                InputItem::SelectSingle(v) => {
                                                    Some(Some(v.is_none()))
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
                                                _ => None,
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
                    } else if let (
                        InputItem::CheckBox(checked, data_children),
                        InputConfig::CheckBox(config),
                    ) = (&(*item), &**input_conf)
                    {
                        if let Some((_, config_children)) = config.children.as_ref() {
                            if !data_children.is_empty()
                                && self.decide_required_all_recursive(
                                    ctx,
                                    data_children,
                                    config_children,
                                    (base_index + index) * MAX_PER_LAYER,
                                    *checked == CheckStatus::Checked,
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
                        (
                            InputItem::CheckBox(checked, data_children),
                            InputConfig::CheckBox(config),
                        ) => {
                            if let Some((_, config_children)) = config.children.as_ref() {
                                if *checked != CheckStatus::Unchecked
                                    && !data_children.is_empty()
                                    && !self.verify_recursive(
                                        data_children,
                                        config_children,
                                        (base_index + index) * MAX_PER_LAYER,
                                        *checked == CheckStatus::Checked,
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
                    if let (
                        InputItem::CheckBox(checked, data_children),
                        InputConfig::CheckBox(config),
                    ) = (&*input_data, &**input_conf)
                    {
                        if let Some((_, config_children)) = config.children.as_ref() {
                            if *checked != CheckStatus::Unchecked && !data_children.is_empty() {
                                Self::trim_nic_recursive(data_children, config_children);
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
                    if let (InputItem::CheckBox(_, _), InputConfig::CheckBox(_))
                    | (InputItem::Radio(_, _), InputConfig::Radio(_)) = (&(*item), &**input_conf)
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
        checked: Option<CheckStatus>,
        layer_index: usize,
        base_index: usize,
    ) -> Option<CheckStatus> {
        let this_checked = if Rc::ptr_eq(click, pos) {
            if let Ok(mut click) = click.try_borrow_mut() {
                if let (InputItem::CheckBox(status, _), InputConfig::CheckBox(config)) =
                    (&mut *click, &**input_conf)
                {
                    match *status {
                        CheckStatus::Checked => {
                            if config.always != Some(CheckStatus::Checked)
                                && config.always != Some(CheckStatus::Indeterminate)
                            {
                                *status = checked.unwrap_or(CheckStatus::Unchecked);
                            }
                            Some(*status)
                        }
                        CheckStatus::Indeterminate | CheckStatus::Unchecked => {
                            if config.always != Some(CheckStatus::Unchecked) {
                                *status = checked.unwrap_or(CheckStatus::Checked);
                            }
                            Some(*status)
                        }
                    }
                } else if let (InputItem::Radio(option, _), InputConfig::Radio(_)) =
                    (&*click, &**input_conf)
                {
                    if option.is_empty() {
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
                if let (InputItem::CheckBox(status, _), InputConfig::CheckBox(_)) =
                    (&mut *pos, &**input_conf)
                {
                    *status = checked;
                    Some(checked)
                } else if let (InputItem::Radio(option, _), InputConfig::Radio(_)) =
                    (&*pos, &**input_conf)
                {
                    if option.is_empty() {
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

        let mut propa_children: Vec<(usize, Rc<RefCell<InputItem>>, Rc<InputConfig>)> = Vec::new();
        if let Ok(pos) = pos.try_borrow_mut() {
            let children =
                if let (InputItem::CheckBox(_, children), InputConfig::CheckBox(config)) =
                    (&*pos, &**input_conf)
                {
                    if let Some((_, config_children)) = config.children.as_ref() {
                        Some((children, config_children))
                    } else {
                        None
                    }
                } else if let (
                    InputItem::Radio(option, children_group),
                    InputConfig::Radio(config),
                ) = (&*pos, &**input_conf)
                {
                    let checked_index =
                        config.options.iter().position(|o| option == &o.to_string());
                    if let Some(checked_index) = checked_index {
                        if let (Some(children), Some(Some(config_children))) = (
                            children_group.get(checked_index),
                            config.children_group.get(checked_index),
                        ) {
                            Some((children, config_children))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

            if let Some((children, config_children)) = children {
                for (index, child) in children.iter().enumerate() {
                    if let (Ok(mut c), Some(t)) =
                        (child.try_borrow_mut(), config_children.get(index))
                    {
                        match (&(*c), &**t) {
                            (InputItem::CheckBox(_, _), InputConfig::CheckBox(_)) => {
                                propa_children.push((index, Rc::clone(child), Rc::clone(t)));
                            }
                            (InputItem::Text(user), InputConfig::Text(config)) => {
                                if user.is_empty() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(value) = &config.ess.default {
                                        *c = value.clone();
                                    }
                                }
                            }
                            (InputItem::Radio(option, _), InputConfig::Radio(config)) => {
                                if option.is_empty() || this_checked == Some(CheckStatus::Unchecked)
                                {
                                    if let (
                                        Some(InputItem::Radio(default_option, _)),
                                        InputItem::Radio(option, _),
                                    ) = (&config.ess.default, &mut *c)
                                    {
                                        option.clone_from(default_option);
                                    }
                                }
                                propa_children.push((index, Rc::clone(child), Rc::clone(t)));
                            }
                            (
                                InputItem::HostNetworkGroup(user),
                                InputConfig::HostNetworkGroup(config),
                            ) => {
                                if user.is_empty() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(value) = &config.ess.default {
                                        *c = value.clone();
                                    }
                                }
                            }
                            (InputItem::SelectSingle(user), InputConfig::SelectSingle(config)) => {
                                if user.is_none() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(value) = &config.ess.default {
                                        *c = value.clone();
                                    }
                                }
                            }
                            (
                                InputItem::SelectMultiple(user),
                                InputConfig::SelectMultiple(config),
                            ) => {
                                if user.is_empty() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(value) = &config.ess.default {
                                        *c = value.clone();
                                    }
                                }
                            }
                            (InputItem::Tag(user), InputConfig::Tag(config)) => {
                                if (user.old.is_empty()
                                    && user.new.is_none()
                                    && user.edit.is_none()
                                    && user.delete.is_none())
                                    || this_checked == Some(CheckStatus::Unchecked)
                                {
                                    if let Some(value) = &config.ess.default {
                                        *c = value.clone();
                                    }
                                }
                            }
                            (InputItem::Unsigned32(user), InputConfig::Unsigned32(config)) => {
                                if user.is_none() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(value) = &config.ess.default {
                                        *c = value.clone();
                                    }
                                }
                            }
                            (InputItem::Percentage(user), InputConfig::Percentage(config)) => {
                                if user.is_none() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(value) = &config.ess.default {
                                        *c = value.clone();
                                    }
                                }
                            }
                            (InputItem::Nic(user), InputConfig::Nic(config)) => {
                                if user.is_empty() || this_checked == Some(CheckStatus::Unchecked) {
                                    if let Some(value) = &config.ess.default {
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
        let mut rtn_checked: Vec<Option<CheckStatus>> = Vec::new();

        for (index, child, config_child) in &propa_children {
            rtn_checked.push(self.propagate_checkbox_recursive(
                click,
                child,
                config_child,
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

                    if let (
                        InputItem::CheckBox(checked, data_children),
                        InputConfig::CheckBox(config),
                    ) = (&*input_data, &**input_conf)
                    {
                        if let Some((_, config_children)) = config.children.as_ref() {
                            if *checked != CheckStatus::Unchecked && !data_children.is_empty() {
                                self.reset_veri_host_network_recursive(
                                    data_children,
                                    config_children,
                                    (base_index + index) * MAX_PER_LAYER,
                                    *checked == CheckStatus::Checked,
                                );
                            }
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
