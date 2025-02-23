use std::time::Duration;

use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};

use crate::{debug_warn, event_delegation, is_server};

thread_local! {
    pub static WINDOW: web_sys::Window = web_sys::window().unwrap_throw();

    pub static DOCUMENT: web_sys::Document = web_sys::window().unwrap_throw().document().unwrap_throw();
}

pub fn window() -> web_sys::Window {
    WINDOW.with(|window| window.clone())
}

pub fn document() -> web_sys::Document {
    DOCUMENT.with(|document| document.clone())
}

pub fn body() -> Option<web_sys::HtmlElement> {
    document().body()
}

pub fn create_element(tag_name: &str) -> web_sys::Element {
    document().create_element(tag_name).unwrap_throw()
}

pub fn create_text_node(data: &str) -> web_sys::Text {
    document().create_text_node(data)
}

pub fn create_fragment() -> web_sys::DocumentFragment {
    document().create_document_fragment()
}

pub fn create_comment_node() -> web_sys::Node {
    document().create_comment("").unchecked_into()
}

pub fn create_template(html: &str) -> web_sys::HtmlTemplateElement {
    let template = create_element("template");
    template.set_inner_html(html);
    template.unchecked_into()
}

pub fn clone_template(template: &web_sys::HtmlTemplateElement) -> web_sys::Element {
    template
        .content()
        .first_element_child()
        .unwrap_throw()
        .clone_node_with_deep(true)
        .unwrap_throw()
        .unchecked_into()
}

pub fn append_child(parent: &web_sys::Element, child: &web_sys::Node) -> web_sys::Node {
    parent.append_child(child).unwrap_throw()
}

pub fn remove_child(parent: &web_sys::Element, child: &web_sys::Node) {
    _ = parent.remove_child(child);
}

pub fn replace_child(parent: &web_sys::Element, new: &web_sys::Node, old: &web_sys::Node) {
    _ = parent.replace_child(new, old);
}

pub fn insert_before(
    parent: &web_sys::Element,
    new: &web_sys::Node,
    existing: Option<&web_sys::Node>,
) -> web_sys::Node {
    if parent.node_type() != 1 {
        debug_warn!("insert_before: trying to insert on a parent node that is not an element");
        new.clone()
    } else if let Some(existing) = existing {
        let parent = existing.parent_node().unwrap_throw();
        match parent.insert_before(new, Some(existing)) {
            Ok(c) => c,
            Err(e) => {
                debug_warn!("{:?}", e.as_string());
                new.clone()
            }
        }
    } else {
        parent.append_child(new).unwrap_throw()
    }
}

pub fn replace_with(old_node: &web_sys::Element, new_node: &web_sys::Node) {
    _ = old_node.replace_with_with_node_1(new_node);
}

pub fn set_data(node: &web_sys::Text, value: &str) {
    node.set_data(value);
}

pub fn set_attribute(el: &web_sys::Element, attr_name: &str, value: &str) {
    _ = el.set_attribute(attr_name, value);
}

pub fn remove_attribute(el: &web_sys::Element, attr_name: &str) {
    _ = el.remove_attribute(attr_name);
}

pub fn set_property(el: &web_sys::Element, prop_name: &str, value: &Option<JsValue>) {
    let key = JsValue::from_str(prop_name);
    match value {
        Some(value) => _ = js_sys::Reflect::set(el, &key, value),
        None => _ = js_sys::Reflect::delete_property(el, &key),
    };
}

pub fn location() -> web_sys::Location {
    window().location()
}

pub fn descendants(el: &web_sys::Element) -> impl Iterator<Item = web_sys::Node> {
    let children = el.child_nodes();
    (0..children.length()).flat_map({
        move |idx| {
            let child = children.get(idx);
            if let Some(child) = child {
                // if an Element, send children
                if child.node_type() == 1 {
                    Box::new(descendants(&child.unchecked_into()))
                        as Box<dyn Iterator<Item = web_sys::Node>>
                }
                // otherwise, just the node
                else {
                    Box::new(std::iter::once(child)) as Box<dyn Iterator<Item = web_sys::Node>>
                }
            } else {
                Box::new(std::iter::empty()) as Box<dyn Iterator<Item = web_sys::Node>>
            }
        }
    })
}

/// Current window.location.hash without the beginning #
pub fn location_hash() -> Option<String> {
    if is_server!() {
        None
    } else {
        location().hash().ok().map(|hash| hash.replace('#', ""))
    }
}

pub fn location_pathname() -> Option<String> {
    location().pathname().ok()
}

pub fn event_target<T>(event: &web_sys::Event) -> T
where
    T: JsCast,
{
    event.target().unwrap_throw().unchecked_into::<T>()
}

pub fn event_target_value(event: &web_sys::Event) -> String {
    event
        .target()
        .unwrap_throw()
        .unchecked_into::<web_sys::HtmlInputElement>()
        .value()
}

pub fn event_target_checked(ev: &web_sys::Event) -> bool {
    ev.target()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>()
        .checked()
}

pub fn event_target_selector(ev: &web_sys::Event, selector: &str) -> bool {
    matches!(
        ev.target().and_then(|target| {
            target
                .dyn_ref::<web_sys::Element>()
                .map(|el| el.closest(selector))
        }),
        Some(Ok(Some(_)))
    )
}

pub fn request_animation_frame(cb: impl Fn() + 'static) {
    let cb = Closure::wrap(Box::new(cb) as Box<dyn Fn()>).into_js_value();
    _ = window().request_animation_frame(cb.as_ref().unchecked_ref());
}

pub fn request_idle_callback(cb: impl Fn() + 'static) {
    let cb = Closure::wrap(Box::new(cb) as Box<dyn Fn()>).into_js_value();
    _ = window().request_idle_callback(cb.as_ref().unchecked_ref());
}

pub fn set_timeout(cb: impl FnOnce() + 'static, duration: Duration) {
    let cb = Closure::once_into_js(Box::new(cb) as Box<dyn FnOnce()>);
    _ = window().set_timeout_with_callback_and_timeout_and_arguments_0(
        cb.as_ref().unchecked_ref(),
        duration.as_millis().try_into().unwrap_throw(),
    );
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntervalHandle(i32);

impl IntervalHandle {
    pub fn clear(&self) {
        window().clear_interval_with_handle(self.0);
    }
}

pub fn set_interval(
    cb: impl Fn() + 'static,
    duration: Duration,
) -> Result<IntervalHandle, JsValue> {
    let cb = Closure::wrap(Box::new(cb) as Box<dyn Fn()>).into_js_value();
    let handle = window().set_interval_with_callback_and_timeout_and_arguments_0(
        cb.as_ref().unchecked_ref(),
        duration.as_millis().try_into().unwrap_throw(),
    )?;
    Ok(IntervalHandle(handle))
}

pub fn add_event_listener(
    target: &web_sys::Element,
    event_name: &'static str,
    cb: impl FnMut(web_sys::Event) + 'static,
) {
    let cb = Closure::wrap(Box::new(cb) as Box<dyn FnMut(web_sys::Event)>).into_js_value();
    let key = event_delegation::event_delegation_key(event_name);
    _ = js_sys::Reflect::set(target, &JsValue::from_str(&key), &cb);
    event_delegation::add_event_listener(event_name);
}

pub fn add_event_listener_undelegated(
    target: &web_sys::Element,
    event_name: &'static str,
    cb: impl FnMut(web_sys::Event) + 'static,
) {
    let cb = Closure::wrap(Box::new(cb) as Box<dyn FnMut(web_sys::Event)>).into_js_value();
    _ = target.add_event_listener_with_callback(event_name, cb.unchecked_ref());
}

#[inline(always)]
pub fn ssr_event_listener(_cb: impl FnMut(web_sys::Event) + 'static) {
    // this function exists only for type inference in templates for SSR
}

pub fn window_event_listener(event_name: &str, cb: impl Fn(web_sys::Event) + 'static) {
    if !is_server!() {
        let handler = Box::new(cb) as Box<dyn FnMut(web_sys::Event)>;

        let cb = Closure::wrap(handler).into_js_value();
        _ = window().add_event_listener_with_callback(event_name, cb.unchecked_ref());
    }
}

pub fn remove_event_listeners(el: &web_sys::Element) {
    let clone = el.clone_node().unwrap_throw();
    replace_with(el, clone.unchecked_ref());
}
