export function toggle_visibility(id) {
    var elem = document.getElementById(id);
    if (elem != null) {
        var elemDisplay = window
            .getComputedStyle(elem)
            .getPropertyValue("display");
        if (elemDisplay == "none") {
            // close other selects
            var elems = document.getElementsByClassName(
                "searchable-select-list-down"
            );
            for (var i = 0; i < elems.length; i++) {
                elems[i].style.display = "none";
                remove_listen_click();
            }
            var elems = document.getElementsByClassName(
                "mini-select-list-down"
            );
            for (var i = 0; i < elems.length; i++) {
                elems[i].style.display = "none";
                remove_listen_click();
            }
            var elems = document.getElementsByClassName(
                "tag-group-input-select"
            );
            for (var i = 0; i < elems.length; i++) {
                elems[i].style.display = "none";
                remove_listen_click();
            }
            elem.style.display = "block";
            add_listen_click();
        } else {
            elem.style.display = "none";
            remove_listen_click();
        }
    }
}

export function toggle_visibility_complex(id) {
    var elem = document.getElementById(id);
    if (elem != null) {
        var elemDisplay = window
            .getComputedStyle(elem)
            .getPropertyValue("display");
        if (elemDisplay == "none") {
            elem.style.display = "block";
            add_listen_click_complex();
            add_listen_mousedown_complex();
        } else {
            elem.style.display = "none";
            remove_listen_click_complex();
            remove_listen_mousedown_complex();
        }
    }
}

export function visible_tag_select(id) {
    var elem = document.getElementById(id);
    if (elem != null) {
        elem.style.display = "block";
        add_listen_click();
    }
}

function close_custom_select(elmnt) {
    // HIGHLIGHT: in case when elmnt has been removed after it was clicked.
    if (elmnt.target.parentNode == null) return;

    if (
        elmnt.target.className == "tag-select-edit" ||
        elmnt.target.className == "tag-select-edit-done" ||
        elmnt.target.className == "tag-input-close"
    ) {
        return;
    }

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
        elems[i].style.display = "none";
        remove_listen_click();
    }

    var elems = document.getElementsByClassName("mini-select-list-down");
    for (i = 0; i < elems.length; i++) {
        elems[i].style.display = "none";
        remove_listen_click();
    }

    var elems = document.getElementsByClassName("tag-group-input-select");
    for (i = 0; i < elems.length; i++) {
        elems[i].style.display = "none";
        remove_listen_click();
    }
}

function close_custom_select_complex(elmnt) {
    // HIGHLIGHT: in case when elmnt has been removed after it was clicked.
    if (
        elmnt.target.parentNode == null ||
        !document.body.contains(elmnt.target)
    )
        return;

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
        elems[i].style.display = "none";
        remove_listen_click_complex();
        remove_listen_mousedown_complex();
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

function mousedown_handler(e) {
    const isInput = e.target.tagName === 'INPUT';
    if (!isInput) {
        e.stopPropagation();
        e.preventDefault();
    }
}

function add_listen_mousedown_complex() {
    document.addEventListener("mousedown", mousedown_handler);
}

function remove_listen_mousedown_complex() {
    document.removeEventListener("mousedown", mousedown_handler);
}
