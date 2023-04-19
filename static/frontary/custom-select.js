export function toggle_visibility(id) {
    var elem = document.getElementById(id);
    if (elem != null) {
        var elemDisplay = window.getComputedStyle(elem).getPropertyValue("visibility");
        if (elemDisplay == "hidden") {
            // close other selects
            var elems = document.getElementsByClassName("searchable-select-list-down");
            for (var i = 0; i < elems.length; i++) {
                elems[i].style.visibility = "hidden";
                remove_listen_click();
            }
            var elems = document.getElementsByClassName("mini-select-list-down");
            for (var i = 0; i < elems.length; i++) {
                elems[i].style.visibility = "hidden";
                remove_listen_click();
            }
            var elems = document.getElementsByClassName("tag-group-input-select");
            for (var i = 0; i < elems.length; i++) {
                elems[i].style.visibility = "hidden";
                remove_listen_click();
            }
            elem.style.visibility = "visible";
            add_listen_click();
        } else {
            elem.style.visibility = "hidden";
            remove_listen_click();
        }
    }
}

export function toggle_visibility_complex(id) {
    var elem = document.getElementById(id);
    if (elem != null) {
        var elemDisplay = window.getComputedStyle(elem).getPropertyValue("visibility");
        if (elemDisplay == "hidden") {
            elem.style.visibility = "visible";
            add_listen_click_complex();
        } else {
            elem.style.visibility = "hidden";
            remove_listen_click_complex();
        }
    }
}

export function visible_tag_select(id) {
    var elem = document.getElementById(id);
    if (elem != null) {
        elem.style.visibility = "visible";
        add_listen_click();
    }
}

function close_custom_select(elmnt) {
    if (elmnt.target.className == "tag-select-edit" || elmnt.target.className == "tag-select-edit-done") {
        return;
    }

    // HIGHLIGHT: in case when elmnt has been removed after it was clicked.
    if (elmnt.target.parentNode == null) return;

    var elems = document.getElementsByClassName("searchable-select");
    var i;
    for (i = 0; i < elems.length; i++) {
        let clickElem = elmnt.target;
        do {
            if (elems[i] == clickElem) {
                return;
            }
            clickElem = clickElem.parentNode;
        } while (clickElem);
    }

    var elems = document.getElementsByClassName("mini-select");
    var i;
    for (i = 0; i < elems.length; i++) {
        let clickElem = elmnt.target;
        do {
            if (elems[i] == clickElem) {
                return;
            }
            clickElem = clickElem.parentNode;
        } while (clickElem);
    }

    var elems = document.getElementsByClassName("tag-group-input-outer");
    var i;
    for (i = 0; i < elems.length; i++) {
        let clickElem = elmnt.target;
        do {
            if (elems[i] == clickElem) {
                return;
            }
            clickElem = clickElem.parentNode;
        } while (clickElem);
    }

    var elems = document.getElementsByClassName("searchable-select-list-down");
    for (i = 0; i < elems.length; i++) {
        elems[i].style.visibility = "hidden";
        remove_listen_click();
    }

    var elems = document.getElementsByClassName("mini-select-list-down");
    for (i = 0; i < elems.length; i++) {
        elems[i].style.visibility = "hidden";
        remove_listen_click();
    }

    var elems = document.getElementsByClassName("tag-group-input-select");
    for (i = 0; i < elems.length; i++) {
        elems[i].style.visibility = "hidden";
        remove_listen_click();
    }
}

function close_custom_select_complex(elmnt) {
    // HIGHLIGHT: in case when elmnt has been removed after it was clicked.
    if (elmnt.target.parentNode == null) return;

    var elems = document.getElementsByClassName("complex-select");
    var i;
    for (i = 0; i < elems.length; i++) {
        let clickElem = elmnt.target;
        do {
            if (elems[i] == clickElem) {
                return;
            }
            clickElem = clickElem.parentNode;
        } while (clickElem);
    }

    var elems = document.getElementsByClassName("complex-select-pop");
    for (i = 0; i < elems.length; i++) {
        elems[i].style.visibility = "hidden";
        remove_listen_click_complex();
    }
}

function add_listen_click() {
    document.addEventListener("click", close_custom_select);
}

function remove_listen_click() {
    document.removeEventListener("click", close_custom_select);
}

function add_listen_click_complex() {
    document.addEventListener("click", close_custom_select_complex);
}

function remove_listen_click_complex() {
    document.removeEventListener("click", close_custom_select_complex);
}
