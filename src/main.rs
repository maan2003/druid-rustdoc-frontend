use std::env;

use druid::{AppLauncher, Selector, WindowDesc};
use rustdoc_types::Id;
use theme::configure_env;
mod data;
mod delegate;
mod format;
mod md;
mod theme;
mod view;
mod widgets;

const GOTO_ITEM: Selector<Id> = Selector::new("druid-rustdoc.goto-item");

fn main() {
    let json_path = env::args().nth(1).unwrap();

    let del = delegate::Delegate::new(json_path.as_ref());
    let data = del.data();
    let window = WindowDesc::new(view::ui_builder()).show_titlebar(false);

    AppLauncher::with_window(window)
        .log_to_console()
        .delegate(del)
        .configure_env(|env, _| configure_env(env))
        .launch(data)
        .unwrap();
}
