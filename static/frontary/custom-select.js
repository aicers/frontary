export function toggle_visibility(id) {
  var elem = document.getElementById(id);
  if (elem != null) {
    const elemDisplay = window
      .getComputedStyle(elem)
      .getPropertyValue("display");
    if (elemDisplay === "none") {
      // close other selects
      let elems = document.getElementsByClassName(
        "searchable-select-list-down",
      );
      for (let i = 0; i < elems.length; i++) {
        elems[i].style.display = "none";
        remove_listen_click();
      }
      elems = document.getElementsByClassName("mini-select-list-down");
      for (let i = 0; i < elems.length; i++) {
        elems[i].style.display = "none";
        remove_listen_click();
      }
      elems = document.getElementsByClassName("tag-group-input-select");
      for (let i = 0; i < elems.length; i++) {
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
    const elemDisplay = window
      .getComputedStyle(elem)
      .getPropertyValue("display");
    const needFlex =
      window.getComputedStyle(elem).getPropertyValue("flex-direction") ===
      "column";
    if (elemDisplay === "none") {
      elem.style.display = needFlex ? "flex" : "block";
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
    elmnt.target.className === "tag-select-edit" ||
    elmnt.target.className === "tag-select-edit-done" ||
    elmnt.target.className === "tag-input-close"
  ) {
    return;
  }

  let elems = document.getElementsByClassName("searchable-select");
  for (let i = 0; i < elems.length; i++) {
    let clickElem = elmnt.target;
    do {
      if (elems[i] === clickElem) {
        return;
      }
      clickElem = clickElem.parentNode;
    } while (clickElem);
  }

  elems = document.getElementsByClassName("mini-select");
  for (let i = 0; i < elems.length; i++) {
    let clickElem = elmnt.target;
    do {
      if (elems[i] === clickElem) {
        return;
      }
      clickElem = clickElem.parentNode;
    } while (clickElem);
  }

  elems = document.getElementsByClassName("tag-group-input-outer");
  for (let i = 0; i < elems.length; i++) {
    let clickElem = elmnt.target;
    do {
      if (elems[i] === clickElem) {
        return;
      }
      clickElem = clickElem.parentNode;
    } while (clickElem);
  }

  elems = document.getElementsByClassName("searchable-select-list-down");
  for (let i = 0; i < elems.length; i++) {
    elems[i].style.display = "none";
    remove_listen_click();
  }

  elems = document.getElementsByClassName("mini-select-list-down");
  for (let i = 0; i < elems.length; i++) {
    elems[i].style.display = "none";
    remove_listen_click();
  }

  elems = document.getElementsByClassName("tag-group-input-select");
  for (let i = 0; i < elems.length; i++) {
    elems[i].style.display = "none";
    remove_listen_click();
  }
}

function close_custom_select_complex(elmnt) {
  // HIGHLIGHT: in case when elmnt has been removed after it was clicked.
  if (elmnt.target.parentNode == null || !document.body.contains(elmnt.target))
    return;

  let elems = document.getElementsByClassName("complex-select");
  for (let i = 0; i < elems.length; i++) {
    let clickElem = elmnt.target;
    do {
      if (elems[i] === clickElem) {
        return;
      }
      clickElem = clickElem.parentNode;
    } while (clickElem);
  }

  elems = document.getElementsByClassName("complex-select-pop");
  for (let i = 0; i < elems.length; i++) {
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
  const isInput = e.target.tagName === "INPUT";
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
