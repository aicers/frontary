use std::cell::RefCell;
use std::rc::Rc;

use gloo_events::EventListener;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, MouseEvent};

thread_local! {
    static CLICK_LISTENER: RefCell<Option<EventListener>> = const { RefCell::new(None) };
    static CLICK_COMPLEX_LISTENER: RefCell<Option<EventListener>> = const { RefCell::new(None) };
    static MOUSEDOWN_COMPLEX_LISTENER: RefCell<Option<EventListener>> = const { RefCell::new(None) };
}

/// Toggle the visibility of an element by ID.
///
/// # Errors
///
/// This function will return an error if:
/// * The window or document cannot be accessed
/// * The element with the given ID cannot be found
#[wasm_bindgen]
pub fn toggle_visibility(id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document found"))?;
    let element = document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element with id '{id}' not found")))?;

    let element = element
        .dyn_into::<HtmlElement>()
        .map_err(|_| JsValue::from_str("Element is not an HtmlElement"))?;

    let computed_style = window.get_computed_style(&element)?;
    let display = computed_style
        .ok_or_else(|| JsValue::from_str("Could not get computed style"))?
        .get_property_value("display")
        .map_err(|_| JsValue::from_str("Could not get display property"))?;

    if display == "none" {
        // Close other selects
        close_all_selects(&document);

        element
            .style()
            .set_property("display", "block")
            .map_err(|_| JsValue::from_str("Could not set display property"))?;

        add_listen_click(&document);
    } else {
        element
            .style()
            .set_property("display", "none")
            .map_err(|_| JsValue::from_str("Could not set display property"))?;

        remove_listen_click();
    }

    Ok(())
}

/// Toggle the visibility of a complex select element by ID.
///
/// # Errors
///
/// This function will return an error if:
/// * The window or document cannot be accessed
/// * The element with the given ID cannot be found
#[wasm_bindgen]
pub fn toggle_visibility_complex(id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document found"))?;
    let element = document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element with id '{id}' not found")))?;

    let element = element
        .dyn_into::<HtmlElement>()
        .map_err(|_| JsValue::from_str("Element is not an HtmlElement"))?;

    let computed_style = window.get_computed_style(&element)?;
    let display = computed_style
        .ok_or_else(|| JsValue::from_str("Could not get computed style"))?
        .get_property_value("display")
        .map_err(|_| JsValue::from_str("Could not get display property"))?;

    if display == "none" {
        element
            .style()
            .set_property("display", "block")
            .map_err(|_| JsValue::from_str("Could not set display property"))?;

        add_listen_click_complex(&document);
        add_listen_mousedown_complex(&document);
    } else {
        element
            .style()
            .set_property("display", "none")
            .map_err(|_| JsValue::from_str("Could not set display property"))?;

        remove_listen_click_complex();
        remove_listen_mousedown_complex();
    }

    Ok(())
}

/// Make a tag select element visible by ID.
///
/// # Errors
///
/// This function will return an error if:
/// * The window or document cannot be accessed
/// * The element with the given ID cannot be found
#[wasm_bindgen]
pub fn visible_tag_select(id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document found"))?;
    let element = document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element with id '{id}' not found")))?;

    let element = element
        .dyn_into::<HtmlElement>()
        .map_err(|_| JsValue::from_str("Element is not an HtmlElement"))?;

    element
        .style()
        .set_property("display", "block")
        .map_err(|_| JsValue::from_str("Could not set display property"))?;

    add_listen_click(&document);

    Ok(())
}

/// Handle for managing click-outside event listeners.
/// Call `stop()` to remove the listener when no longer needed.
#[wasm_bindgen]
pub struct ClickOutsideHandle {
    listener: Option<EventListener>,
}

#[wasm_bindgen]
impl ClickOutsideHandle {
    /// Stop listening for clicks outside the element.
    pub fn stop(&mut self) {
        self.listener = None;
    }
}

/// API to enable click-outside detection for custom areas.
/// When a user clicks outside the specified element, the callback will be invoked.
///
/// # Arguments
///
/// * `element_id` - The ID of the element to monitor clicks outside of
/// * `callback` - The callback function to invoke when a click outside occurs
///
/// # Returns
///
/// Returns a handle that can be used to stop listening for clicks outside.
///
/// # Errors
///
/// This function will return an error if:
/// * The window or document cannot be accessed
/// * The element with the given ID cannot be found
#[wasm_bindgen]
pub fn listen_click_outside(
    element_id: &str,
    callback: &js_sys::Function,
) -> Result<ClickOutsideHandle, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document found"))?;

    let target_element = document
        .get_element_by_id(element_id)
        .ok_or_else(|| JsValue::from_str(&format!("Element with id '{element_id}' not found")))?;

    let callback = callback.clone();
    let target_element = Rc::new(target_element);

    let listener = EventListener::new(&document, "click", move |event| {
        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>()
            && let Some(click_target) = mouse_event.target()
            && let Some(click_element) = click_target.dyn_ref::<Element>()
        {
            // Check if the click was outside the target element
            if !is_descendant_of(click_element, &target_element) {
                let this = JsValue::NULL;
                let _ = callback.call1(&this, &JsValue::from(mouse_event));
            }
        }
    });

    Ok(ClickOutsideHandle {
        listener: Some(listener),
    })
}

fn close_all_selects(document: &Document) {
    let class_names = [
        "searchable-select-list-down",
        "mini-select-list-down",
        "tag-group-input-select",
    ];

    for class_name in &class_names {
        let elements = document.get_elements_by_class_name(class_name);
        for i in 0..elements.length() {
            if let Some(elem) = elements.item(i)
                && let Ok(html_elem) = elem.dyn_into::<HtmlElement>()
            {
                let _ = html_elem.style().set_property("display", "none");
            }
        }
    }

    remove_listen_click();
}

fn add_listen_click(document: &Document) {
    remove_listen_click();

    let document_for_closure = document.clone();
    let listener = EventListener::new(document, "click", move |event| {
        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
            close_custom_select(mouse_event, &document_for_closure);
        }
    });

    CLICK_LISTENER.with(|l| {
        *l.borrow_mut() = Some(listener);
    });
}

fn remove_listen_click() {
    CLICK_LISTENER.with(|l| {
        *l.borrow_mut() = None;
    });
}

fn add_listen_click_complex(document: &Document) {
    remove_listen_click_complex();

    let document_for_closure = document.clone();
    let listener = EventListener::new(document, "click", move |event| {
        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
            close_custom_select_complex(mouse_event, &document_for_closure);
        }
    });

    CLICK_COMPLEX_LISTENER.with(|l| {
        *l.borrow_mut() = Some(listener);
    });
}

fn remove_listen_click_complex() {
    CLICK_COMPLEX_LISTENER.with(|l| {
        *l.borrow_mut() = None;
    });
}

fn add_listen_mousedown_complex(document: &Document) {
    remove_listen_mousedown_complex();

    let listener = EventListener::new(document, "mousedown", |event| {
        if let Some(mouse_event) = event.dyn_ref::<MouseEvent>()
            && let Some(target) = mouse_event.target()
            && let Some(element) = target.dyn_ref::<Element>()
        {
            let is_input = element.tag_name() == "INPUT";
            if !is_input {
                mouse_event.stop_propagation();
                mouse_event.prevent_default();
            }
        }
    });

    MOUSEDOWN_COMPLEX_LISTENER.with(|l| {
        *l.borrow_mut() = Some(listener);
    });
}

fn remove_listen_mousedown_complex() {
    MOUSEDOWN_COMPLEX_LISTENER.with(|l| {
        *l.borrow_mut() = None;
    });
}

fn close_custom_select(event: &MouseEvent, document: &Document) {
    // Check if the target's parent node exists
    if let Some(target) = event.target()
        && let Some(element) = target.dyn_ref::<Element>()
    {
        // Check if element has been removed from DOM
        if element.parent_node().is_none() {
            return;
        }

        // Check for specific class names to ignore
        let class_name = element.class_name();
        if class_name == "tag-select-edit"
            || class_name == "tag-select-edit-done"
            || class_name == "tag-input-close"
        {
            return;
        }

        // Check if click is inside any of the select elements
        let select_classes = ["searchable-select", "mini-select", "tag-group-input-outer"];
        for class in &select_classes {
            let elements = document.get_elements_by_class_name(class);
            for i in 0..elements.length() {
                if let Some(select_elem) = elements.item(i)
                    && is_descendant_of(element, &select_elem)
                {
                    return;
                }
            }
        }

        // Close all dropdowns
        let dropdown_classes = [
            "searchable-select-list-down",
            "mini-select-list-down",
            "tag-group-input-select",
        ];
        for class in &dropdown_classes {
            let elements = document.get_elements_by_class_name(class);
            for i in 0..elements.length() {
                if let Some(elem) = elements.item(i)
                    && let Ok(html_elem) = elem.dyn_into::<HtmlElement>()
                {
                    let _ = html_elem.style().set_property("display", "none");
                }
            }
        }

        remove_listen_click();
    }
}

fn close_custom_select_complex(event: &MouseEvent, document: &Document) {
    // Check if the target's parent node exists
    if let Some(target) = event.target()
        && let Some(element) = target.dyn_ref::<Element>()
    {
        // Check if element has been removed from DOM or doesn't exist in body
        if element.parent_node().is_none() {
            return;
        }

        // Check if click is inside any complex select
        let elements = document.get_elements_by_class_name("complex-select");
        for i in 0..elements.length() {
            if let Some(select_elem) = elements.item(i)
                && is_descendant_of(element, &select_elem)
            {
                return;
            }
        }

        // Close all complex select popups
        let elements = document.get_elements_by_class_name("complex-select-pop");
        for i in 0..elements.length() {
            if let Some(elem) = elements.item(i)
                && let Ok(html_elem) = elem.dyn_into::<HtmlElement>()
            {
                let _ = html_elem.style().set_property("display", "none");
            }
        }

        remove_listen_click_complex();
        remove_listen_mousedown_complex();
    }
}

fn is_descendant_of(element: &Element, ancestor: &Element) -> bool {
    let mut current = Some(element.clone());

    while let Some(elem) = current {
        if elem == *ancestor {
            return true;
        }
        current = elem.parent_element();
    }

    false
}
