#![allow(warnings)]
use std::{env::args, fs, time::Instant};

use druid::{im::Vector, AppDelegate, AppLauncher, Data, Selector, Target, WindowDesc};
use rustdoc_types::{Crate, Id};
use theme::configure_env;
use widgets::{ID_QUERY, ID_QUERY_RESPONSE};
mod format;
mod lens;
mod md;
mod theme;
mod title_bar;
mod view;
mod widgets;

fn main() {
    let data = fs::read_to_string(format!("{}.json", args().nth(1).unwrap())).unwrap();
    let krate = rustdoc_types::parse(&data);
    drop(data);
    let data = AppData {
        current_item: krate.root.clone(),
        krate,
        stack: Vector::new(),
    };

    let window = WindowDesc::new(view::ui_builder).show_titlebar(false);
    AppLauncher::with_window(window)
        .use_simple_logger()
        .delegate(Delegate)
        .configure_env(|env, _| configure_env(env))
        .launch(data)
        .unwrap();
}

#[derive(Data)]
pub struct AppData {
    current_item: Id,
    krate: Crate,
    stack: Vector<Id>,
}

impl Clone for AppData {
    fn clone(&self) -> Self {
        println!("Clone");
        let time = Instant::now();
        let x = Self {
            current_item: self.current_item.clone(),
            krate: self.krate.clone(),
            stack: self.stack.clone(),
        };
        dbg!(time.elapsed());
        x
    }
}

struct Delegate;

const GOTO_ITEM: Selector<Id> = Selector::new("druid-rustdoc.goto-item");
const GO_BACK: Selector<()> = Selector::new("druid-rustdoc.go-back");

impl AppDelegate<AppData> for Delegate {
    fn command(
        &mut self,
        ctx: &mut druid::DelegateCtx,
        target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppData,
        env: &druid::Env,
    ) -> druid::Handled {
        if let Some((ids, widget_id)) = cmd.get(ID_QUERY) {
            let items = ids
                .iter()
                .map(|id| {
                    (
                        data.krate.index.get(id).unwrap().clone(),
                        data.krate.paths.get(id).cloned(),
                    )
                })
                .collect();
            ctx.submit_command(ID_QUERY_RESPONSE.with(items).to(Target::Widget(*widget_id)));
            return druid::Handled::Yes;
        }
        if let Some(id) = cmd.get(GOTO_ITEM) {
            data.stack
                .push_back(std::mem::replace(&mut data.current_item, id.clone()));
        }
        if cmd.is(GO_BACK) {
            if let Some(id) = data.stack.pop_front() {
                dbg!("here");
                data.current_item = id;
            }
        }
        druid::Handled::No
    }
}
