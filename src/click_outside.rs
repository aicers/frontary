use std::cell::RefCell;

use gloo_events::EventListener;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{AddEventListenerOptions, Document, Element, HtmlElement, MouseEvent, Window};

type MousedownHandler = Closure<dyn FnMut(MouseEvent)>;

thread_local! {
    static CLICK_LISTENER: RefCell<Option<EventListener>> = const { RefCell::new(None) };
    static CLICK_COMPLEX_LISTENER: RefCell<Option<EventListener>> = const { RefCell::new(None) };
    static MOUSEDOWN_COMPLEX_HANDLER: RefCell<Option<MousedownHandler>> =
        const { RefCell::new(None) };
}

/// Helper function to get window and document
fn get_window_and_document() -> Result<(Window, Document), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("No document found"))?;
    Ok((window, document))
}

/// Helper function to get an HTML element by ID with proper error handling
fn get_html_element_by_id(document: &Document, id: &str) -> Result<HtmlElement, JsValue> {
    let element = document
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str(&format!("Element with id '{id}' not found")))?;

    element
        .dyn_into::<HtmlElement>()
        .map_err(|_| JsValue::from_str("Element is not an HtmlElement"))
}

/// Helper function to check if an element is hidden (display: none)
fn is_element_hidden(window: &Window, element: &HtmlElement) -> Result<bool, JsValue> {
    let computed_style = window.get_computed_style(element)?;
    let display = computed_style
        .ok_or_else(|| JsValue::from_str("Could not get computed style"))?
        .get_property_value("display")
        .map_err(|_| JsValue::from_str("Could not get display property"))?;

    Ok(display == "none")
}

/// Toggle the visibility of an element by ID.
///
/// # Errors
///
/// This function will return an error if:
/// * The window or document cannot be accessed
/// * The element with the given ID cannot be found
pub fn toggle_visibility(id: &str) -> Result<(), JsValue> {
    let (window, document) = get_window_and_document()?;
    let element = get_html_element_by_id(&document, id)?;

    if is_element_hidden(&window, &element)? {
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
pub fn toggle_visibility_complex(id: &str) -> Result<(), JsValue> {
    let (window, document) = get_window_and_document()?;
    let element = get_html_element_by_id(&document, id)?;
    let display = {
        #[cfg(feature = "pumpkin")]
        {
            "flex"
        }
        #[cfg(not(feature = "pumpkin"))]
        {
            "block"
        }
    };

    if is_element_hidden(&window, &element)? {
        element
            .style()
            .set_property("display", display)
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
pub fn visible_tag_select(id: &str) -> Result<(), JsValue> {
    let (_window, document) = get_window_and_document()?;
    let element = get_html_element_by_id(&document, id)?;

    element
        .style()
        .set_property("display", "block")
        .map_err(|_| JsValue::from_str("Could not set display property"))?;

    add_listen_click(&document);

    Ok(())
}

/// Handle for managing click-outside event listeners.
/// This struct provides lifetime management for click-outside event listeners,
/// allowing proper cleanup when the listener is no longer needed.
pub struct ClickOutsideHandle {
    listener: Option<EventListener>,
}

impl ClickOutsideHandle {
    /// Stop listening for clicks outside the element.
    pub fn stop(&mut self) {
        self.listener = None;
    }
}

/// API to enable click-outside detection for custom areas.
/// When a user clicks outside the specified element, the callback will be invoked.
/// The listener fetches the target element on every event so it remains accurate
/// even if the DOM node is replaced by the renderer (for example, after a Yew
/// re-render or a component toggle).
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
pub fn listen_click_outside<F>(element_id: &str, callback: F) -> Result<ClickOutsideHandle, JsValue>
where
    F: Fn(&MouseEvent) + 'static,
{
    let (_window, document) = get_window_and_document()?;

    if document.get_element_by_id(element_id).is_none() {
        return Err(JsValue::from_str(&format!(
            "Element with id '{element_id}' not found"
        )));
    }

    let element_id = element_id.to_string();
    let document_for_closure = document.clone();
    let listener = EventListener::new(&document, "click", move |event| {
        let Some(mouse_event) = event.dyn_ref::<MouseEvent>() else {
            return;
        };

        let Some(target_root) = document_for_closure.get_element_by_id(&element_id) else {
            return;
        };

        if let Some(click_target) = mouse_event.target()
            && let Some(click_element) = click_target.dyn_ref::<Element>()
            && !target_root.contains(Some(click_element))
        {
            callback(mouse_event);
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

    let options = {
        let opts = AddEventListenerOptions::new();
        opts.set_passive(false);
        opts
    };

    let handler = Closure::wrap(Box::new(move |event: MouseEvent| {
        if let Some(element) = event.target().and_then(|t| t.dyn_into::<Element>().ok())
            && element.tag_name() != "INPUT"
        {
            event.stop_propagation();
            event.prevent_default();
        }
    }) as Box<dyn FnMut(MouseEvent)>);

    document
        .add_event_listener_with_callback_and_add_event_listener_options(
            "mousedown",
            handler.as_ref().unchecked_ref(),
            &options,
        )
        .expect("register mousedown listener");

    // keep handler and options alive for removal
    MOUSEDOWN_COMPLEX_HANDLER.with(|slot| *slot.borrow_mut() = Some(handler));
}

fn remove_listen_mousedown_complex() {
    MOUSEDOWN_COMPLEX_HANDLER.with(|slot| {
        if let Some(handler) = slot.borrow_mut().take()
            && let Some(document) = web_sys::window().and_then(|w| w.document())
        {
            let _ = document.remove_event_listener_with_callback_and_bool(
                "mousedown",
                handler.as_ref().unchecked_ref(),
                false,
            );
        }
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
                    && select_elem.contains(Some(element))
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

        // Skip if the event target was already removed from the DOM.
        if let Some(body) = document.body()
            && !body.contains(Some(element))
        {
            return;
        }

        // Check if click is inside any complex select
        let elements = document.get_elements_by_class_name("complex-select");
        for i in 0..elements.length() {
            if let Some(select_elem) = elements.item(i)
                && select_elem.contains(Some(element))
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
