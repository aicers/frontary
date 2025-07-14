use std::include_bytes;

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn static_files() -> Vec<(&'static str, &'static [u8])> {
    let mut rtn: Vec<(&'static str, &'static [u8])> = Vec::new();

    if cfg!(feature = "pumpkin") {
        let theme = include_bytes!("../static/frontary/pumpkin/theme.css");
        rtn.push(("theme.css", theme));

        let enabled_unchecked = include_bytes!("../static/frontary/pumpkin/enabled-unchecked.svg");
        rtn.push(("enabled-unchecked.svg", enabled_unchecked));

        let disabled_unchecked =
            include_bytes!("../static/frontary/pumpkin/disabled-unchecked.svg");
        rtn.push(("disabled-unchecked.svg", disabled_unchecked));

        let enabled_indeterminate =
            include_bytes!("../static/frontary/pumpkin/enabled-indeterminate.svg");
        rtn.push(("enabled-indeterminate.svg", enabled_indeterminate));

        let disabled_indeterminate =
            include_bytes!("../static/frontary/pumpkin/disabled-indeterminate.svg");
        rtn.push(("disabled-indeterminate.svg", disabled_indeterminate));

        let enabled_checked = include_bytes!("../static/frontary/pumpkin/enabled-checked.svg");
        rtn.push(("enabled-checked.svg", enabled_checked));

        let disabled_checked = include_bytes!("../static/frontary/pumpkin/disabled-checked.svg");
        rtn.push(("disabled-checked.svg", disabled_checked));

        let radio_unchecked = include_bytes!("../static/frontary/pumpkin/radio-unchecked.svg");
        rtn.push(("radio-unchecked.svg", radio_unchecked));

        let radio_checked = include_bytes!("../static/frontary/pumpkin/radio-checked.svg");
        rtn.push(("radio-checked.svg", radio_checked));

        let modal_divider = include_bytes!("../static/frontary/pumpkin/modal-divider.svg");
        rtn.push(("modal-divider.svg", modal_divider));

        let modal_close = include_bytes!("../static/frontary/pumpkin/modal-close.svg");
        rtn.push(("modal-close.svg", modal_close));

        let list_add = include_bytes!("../static/frontary/pumpkin/list-add.svg");
        rtn.push(("list-add.svg", list_add));

        let list_sort_recently =
            include_bytes!("../static/frontary/pumpkin/list-sort-recently.svg");
        rtn.push(("list-sort-recently.svg", list_sort_recently));

        let sort_unsorted = include_bytes!("../static/frontary/pumpkin/sort-unsorted.svg");
        rtn.push(("sort-unsorted.svg", sort_unsorted));

        let sort_ascending = include_bytes!("../static/frontary/pumpkin/sort-ascending.svg");
        rtn.push(("sort-ascending.svg", sort_ascending));

        let sort_descending = include_bytes!("../static/frontary/pumpkin/sort-descending.svg");
        rtn.push(("sort-descending.svg", sort_descending));

        let sort_unsorted_hover =
            include_bytes!("../static/frontary/pumpkin/sort-unsorted-hover.svg");
        rtn.push(("sort-unsorted-hover.svg", sort_unsorted_hover));

        let input_close = include_bytes!("../static/frontary/pumpkin/input-close.svg");
        rtn.push(("input-close.svg", input_close));

        let select_down = include_bytes!("../static/frontary/pumpkin/select-down.svg");
        rtn.push(("select-down.svg", select_down));

        let select_down_disabled =
            include_bytes!("../static/frontary/pumpkin/select-down-disabled.svg");
        rtn.push(("select-down-disabled.svg", select_down_disabled));

        let magnifier = include_bytes!("../static/frontary/pumpkin/magnifier.svg");
        rtn.push(("magnifier.svg", magnifier));

        let collapse_list = include_bytes!("../static/frontary/pumpkin/collapse-list.svg");
        rtn.push(("collapse-list.svg", collapse_list));

        let expand_list = include_bytes!("../static/frontary/pumpkin/expand-list.svg");
        rtn.push(("expand-list.svg", expand_list));

        let more_action_dots_hover =
            include_bytes!("../static/frontary/pumpkin/more-action-dots-hover.svg");
        rtn.push(("more-action-dots-hover.svg", more_action_dots_hover));

        let more_action_dots = include_bytes!("../static/frontary/pumpkin/more-action-dots.svg");
        rtn.push(("more-action-dots.svg", more_action_dots));

        let edit = include_bytes!("../static/frontary/pumpkin/edit.svg");
        rtn.push(("edit.svg", edit));

        let delete_trash = include_bytes!("../static/frontary/pumpkin/delete-trash.svg");
        rtn.push(("delete-trash.svg", delete_trash));

        let delete_x = include_bytes!("../static/frontary/pumpkin/delete-x.svg");
        rtn.push(("delete-x.svg", delete_x));

        let mini_select_list_down =
            include_bytes!("../static/frontary/pumpkin/mini-select-list-down.svg");
        rtn.push(("mini-select-list-down.svg", mini_select_list_down));

        let notification_error =
            include_bytes!("../static/frontary/pumpkin/notification-error.svg");
        rtn.push(("notification-error.svg", notification_error));

        let select_down_alert = include_bytes!("../static/frontary/pumpkin/select-down-alert.svg");
        rtn.push(("select-down-alert.svg", select_down_alert));

        let checkbox_link_up_line =
            include_bytes!("../static/frontary/pumpkin/checkbox-link-up-line.svg");
        rtn.push(("checkbox-link-up-line.svg", checkbox_link_up_line));

        let collapse_contents = include_bytes!("../static/frontary/pumpkin/collapse-contents.svg");
        rtn.push(("collapse-contents.svg", collapse_contents));

        let expand_contents = include_bytes!("../static/frontary/pumpkin/expand-contents.svg");
        rtn.push(("expand-contents.svg", expand_contents));

        let complex_select_pop =
            include_bytes!("../static/frontary/pumpkin/complex-select-pop.svg");
        rtn.push(("complex-select-pop.svg", complex_select_pop));

        let complex_select_pop_alert =
            include_bytes!("../static/frontary/pumpkin/complex-select-pop-alert.svg");
        rtn.push(("complex-select-pop-alert.svg", complex_select_pop_alert));

        let tag_input_close = include_bytes!("../static/frontary/pumpkin/tag-input-close.svg");
        rtn.push(("tag-input-close.svg", tag_input_close));

        let tag_select_edit_done_dim =
            include_bytes!("../static/frontary/pumpkin/tag-select-edit-done-dim.svg");
        rtn.push(("tag-select-edit-done-dim.svg", tag_select_edit_done_dim));

        let tag_select_edit_done =
            include_bytes!("../static/frontary/pumpkin/tag-select-edit-done.svg");
        rtn.push(("tag-select-edit-done.svg", tag_select_edit_done));

        let radio_opener_checked =
            include_bytes!("../static/frontary/pumpkin/radio-opener-checked.svg");
        rtn.push(("radio-opener-checked.svg", radio_opener_checked));

        let radio_opener_unchecked =
            include_bytes!("../static/frontary/pumpkin/radio-opener-unchecked.svg");
        rtn.push(("radio-opener-unchecked.svg", radio_opener_unchecked));

        let traffic_direction_dim =
            include_bytes!("../static/frontary/pumpkin/traffic-direction-dim.svg");
        rtn.push(("traffic-direction-dim.svg", traffic_direction_dim));

        let traffic_direction = include_bytes!("../static/frontary/pumpkin/traffic-direction.svg");
        rtn.push(("traffic-direction.svg", traffic_direction));

        let traffic_direction_hover =
            include_bytes!("../static/frontary/pumpkin/traffic-direction-hover.svg");
        rtn.push(("traffic-direction-hover.svg", traffic_direction_hover));

        let off = include_bytes!("../static/frontary/off.png");
        rtn.push(("off.png", off));

        let on = include_bytes!("../static/frontary/on.png");
        rtn.push(("on.png", on));

        let plus_for_add = include_bytes!("../static/frontary/pumpkin/plus-for-add.svg");
        rtn.push(("plus-for-add.svg", plus_for_add));

        let nic_delete = include_bytes!("../static/frontary/pumpkin/nic-delete.svg");
        rtn.push(("nic-delete.svg", nic_delete));

        let notification_close =
            include_bytes!("../static/frontary/pumpkin/notification-close.svg");
        rtn.push(("notification-close.svg", notification_close));

        let close_white = include_bytes!("../static/frontary/pumpkin/close-white.svg");
        rtn.push(("close-white.svg", close_white));

        let group_list_link_line_bottom =
            include_bytes!("../static/frontary/pumpkin/group-list-link-line-bottom.svg");
        rtn.push((
            "group-list-link-line-bottom.svg",
            group_list_link_line_bottom,
        ));

        let group_list_link_line_top =
            include_bytes!("../static/frontary/pumpkin/group-list-link-line-top.svg");
        rtn.push(("group-list-link-line-top.svg", group_list_link_line_top));

        let group_list_link_line_top_long =
            include_bytes!("../static/frontary/pumpkin/group-list-link-line-top-long.svg");
        rtn.push((
            "group-list-link-line-top-long.svg",
            group_list_link_line_top_long,
        ));

        let trash_can = include_bytes!("../static/frontary/pumpkin/trash-can.svg");
        rtn.push(("trash-can.svg", trash_can));

        let addition_symbol = include_bytes!("../static/frontary/pumpkin/addition-symbol.svg");
        rtn.push(("addition-symbol.svg", addition_symbol));
    } else {
        let theme = include_bytes!("../static/frontary/theme.css");
        rtn.push(("theme.css", theme));

        let checkbox_checked_always =
            include_bytes!("../static/frontary/checkbox-checked-always.png");
        rtn.push(("checkbox-checked-always.png", checkbox_checked_always));

        let checkbox_checked = include_bytes!("../static/frontary/checkbox-checked.png");
        rtn.push(("checkbox-checked.png", checkbox_checked));

        let checkbox_indeterminate_always =
            include_bytes!("../static/frontary/checkbox-indeterminate-always.png");
        rtn.push((
            "checkbox-indeterminate-always.png",
            checkbox_indeterminate_always,
        ));

        let checkbox_indeterminate =
            include_bytes!("../static/frontary/checkbox-indeterminate.png");
        rtn.push(("checkbox-indeterminate.png", checkbox_indeterminate));

        let checkbox_link_flat_line =
            include_bytes!("../static/frontary/checkbox-link-flat-line.png");
        rtn.push(("checkbox-link-flat-line.png", checkbox_link_flat_line));

        let checkbox_link_up_line = include_bytes!("../static/frontary/checkbox-link-up-line.png");
        rtn.push(("checkbox-link-up-line.png", checkbox_link_up_line));

        let checkbox_unchecked_always =
            include_bytes!("../static/frontary/checkbox-unchecked-always.png");
        rtn.push(("checkbox-unchecked-always.png", checkbox_unchecked_always));

        let checkbox_unchecked = include_bytes!("../static/frontary/checkbox-unchecked.png");
        rtn.push(("checkbox-unchecked.png", checkbox_unchecked));

        let close_white = include_bytes!("../static/frontary/close-white.png");
        rtn.push(("close-white.png", close_white));

        let close = include_bytes!("../static/frontary/close.png");
        rtn.push(("close.png", close));

        let collapse_contents = include_bytes!("../static/frontary/collapse-contents.png");
        rtn.push(("collapse-contents.png", collapse_contents));

        let collapse_list = include_bytes!("../static/frontary/collapse-list.png");
        rtn.push(("collapse-list.png", collapse_list));

        let complex_select_pop_alert =
            include_bytes!("../static/frontary/complex-select-pop-alert.png");
        rtn.push(("complex-select-pop-alert.png", complex_select_pop_alert));

        let complex_select_pop = include_bytes!("../static/frontary/complex-select-pop.png");
        rtn.push(("complex-select-pop.png", complex_select_pop));

        let custom_select = include_bytes!("../static/frontary/custom-select.js");
        rtn.push(("custom_select.js", custom_select));

        let delete_trash_white = include_bytes!("../static/frontary/delete-trash-white.png");
        rtn.push(("delete-trash-white.png", delete_trash_white));

        let delete_trash = include_bytes!("../static/frontary/delete-trash.png");
        rtn.push(("delete-trash.png", delete_trash));

        let delete_x = include_bytes!("../static/frontary/delete-x.png");
        rtn.push(("delete-x.png", delete_x));

        let edit = include_bytes!("../static/frontary/edit.png");
        rtn.push(("edit.png", edit));

        let expand_contents = include_bytes!("../static/frontary/expand-contents.png");
        rtn.push(("expand-contents.png", expand_contents));

        let expand_list = include_bytes!("../static/frontary/expand-list.png");
        rtn.push(("expand-list.png", expand_list));

        let host_network_close = include_bytes!("../static/frontary/host-network-close.png");
        rtn.push(("host-network-close.png", host_network_close));

        let list_add = include_bytes!("../static/frontary/list-add.png");
        rtn.push(("list-add.png", list_add));

        let list_sort_recently_hover =
            include_bytes!("../static/frontary/list-sort-recently-hover.png");
        rtn.push(("list-sort-recently-hover.png", list_sort_recently_hover));

        let list_sort_recently = include_bytes!("../static/frontary/list-sort-recently.png");
        rtn.push(("list-sort-recently.png", list_sort_recently));

        let magnifier = include_bytes!("../static/frontary/magnifier.png");
        rtn.push(("magnifier.png", magnifier));

        let mini_select_list_down_triangular =
            include_bytes!("../static/frontary/mini-select-list-down-triangular.png");
        rtn.push((
            "mini-select-list-down-triangular.png",
            mini_select_list_down_triangular,
        ));

        let mini_select_list_down = include_bytes!("../static/frontary/mini-select-list-down.png");
        rtn.push(("mini-select-list-down.png", mini_select_list_down));

        let modal_alert = include_bytes!("../static/frontary/modal-alert.png");
        rtn.push(("modal-alert.png", modal_alert));

        let modal_close = include_bytes!("../static/frontary/modal-close.png");
        rtn.push(("modal-close.png", modal_close));

        let modal_info = include_bytes!("../static/frontary/modal-info.png");
        rtn.push(("modal-info.png", modal_info));

        let more_action_dots_hover =
            include_bytes!("../static/frontary/more-action-dots-hover.png");
        rtn.push(("more-action-dots-hover.png", more_action_dots_hover));

        let more_action_dots = include_bytes!("../static/frontary/more-action-dots.png");
        rtn.push(("more-action-dots.png", more_action_dots));

        let nic_delete = include_bytes!("../static/frontary/nic-delete.png");
        rtn.push(("nic-delete.png", nic_delete));

        let notification_close = include_bytes!("../static/frontary/notification-close.png");
        rtn.push(("notification-close.png", notification_close));

        let off = include_bytes!("../static/frontary/off.png");
        rtn.push(("off.png", off));

        let on = include_bytes!("../static/frontary/on.png");
        rtn.push(("on.png", on));

        let page_go = include_bytes!("../static/frontary/page-go.png");
        rtn.push(("page-go.png", page_go));

        let plus_for_add = include_bytes!("../static/frontary/plus-for-add.png");
        rtn.push(("plus-for-add.png", plus_for_add));

        let radio_checked = include_bytes!("../static/frontary/radio-checked.png");
        rtn.push(("radio-checked.png", radio_checked));

        let radio_opener_checked = include_bytes!("../static/frontary/radio-opener-checked.png");
        rtn.push(("radio-opener-checked.png", radio_opener_checked));

        let radio_opener_unchecked =
            include_bytes!("../static/frontary/radio-opener-unchecked.png");
        rtn.push(("radio-opener-unchecked.png", radio_opener_unchecked));

        let radio_unchecked = include_bytes!("../static/frontary/radio-unchecked.png");
        rtn.push(("radio-unchecked.png", radio_unchecked));

        let select_down_alert = include_bytes!("../static/frontary/select-down-alert.png");
        rtn.push(("select-down-alert.png", select_down_alert));

        let select_down = include_bytes!("../static/frontary/select-down.png");
        rtn.push(("select-down.png", select_down));

        let sort_ascending = include_bytes!("../static/frontary/sort-ascending.png");
        rtn.push(("sort-ascending.png", sort_ascending));

        let sort_descending = include_bytes!("../static/frontary/sort-descending.png");
        rtn.push(("sort-descending.png", sort_descending));

        let sort_to_ascending_from_descending =
            include_bytes!("../static/frontary/sort-to-ascending-from-descending.png");
        rtn.push((
            "sort-to-ascending-from-descending.png",
            sort_to_ascending_from_descending,
        ));

        let sort_to_ascending = include_bytes!("../static/frontary/sort-to-ascending.png");
        rtn.push(("sort-to-ascending.png", sort_to_ascending));

        let sort_to_descending = include_bytes!("../static/frontary/sort-to-descending.png");
        rtn.push(("sort-to-descending.png", sort_to_descending));

        let sort_unsorted = include_bytes!("../static/frontary/sort-unsorted.png");
        rtn.push(("sort-unsorted.png", sort_unsorted));

        let tag_input_close = include_bytes!("../static/frontary/tag-input-close.png");
        rtn.push(("tag-input-close.png", tag_input_close));

        let tag_select_bar = include_bytes!("../static/frontary/tag-select-bar.png");
        rtn.push(("tag-select-bar.png", tag_select_bar));

        let tag_select_edit_done_dim =
            include_bytes!("../static/frontary/tag-select-edit-done-dim.png");
        rtn.push(("tag-select-edit-done-dim.png", tag_select_edit_done_dim));

        let tag_select_edit_done = include_bytes!("../static/frontary/tag-select-edit-done.png");
        rtn.push(("tag-select-edit-done.png", tag_select_edit_done));

        let tag_select_edit = include_bytes!("../static/frontary/tag-select-edit.png");
        rtn.push(("tag-select-edit.png", tag_select_edit));

        let tag_select_trash = include_bytes!("../static/frontary/tag-select-trash.png");
        rtn.push(("tag-select-trash.png", tag_select_trash));

        let traffic_direction_dim = include_bytes!("../static/frontary/traffic-direction-dim.png");
        rtn.push(("traffic-direction-dim.png", traffic_direction_dim));

        let traffic_direction = include_bytes!("../static/frontary/traffic-direction.png");
        rtn.push(("traffic-direction.png", traffic_direction));
    }

    rtn
}
