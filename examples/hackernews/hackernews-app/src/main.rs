use hackernews_app::*;
use leptos::*;

pub fn main() {
    console_error_panic_hook::set_once();
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|cx| {
        view! { cx, <App/> }
    });
}
