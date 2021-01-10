use druid::{
    commands::{CLOSE_WINDOW, CONFIGURE_WINDOW},
    widget::{Controller, Flex, Image, Label, MainAxisAlignment},
    Color, Event, ImageBuf, Widget, WidgetExt, WindowConfig,
};

use crate::{GO_BACK, theme::ICONS_FONT};

#[cfg(target_os = "windows")]
pub(crate) fn title_bar() -> impl Widget<()> {
    // let rust_logo = include_bytes!("..\\assets\\rust-logo-white.png");
    // let logo = ImageBuf::from_file("assets/rust-logo-white.png").unwrap();
    // let logo = Image::new(logo);
    // let logo = Label::new("R");
    let title_bar = Flex::row()
        // .with_child(logo)
        .with_child(Label::new("Rustdoc").padding((5., 5.)))
        .expand_width()
        .controller(TBar);
    // let x: char = std::char::from_u32(0xE7AC).unwrap();

    Flex::row()
        .with_child(
            Label::new("\u{E72B}")
                .with_font(ICONS_FONT)
                .padding((15., 5.))
                .on_click(|ctx, _, _| {
                    ctx.submit_command(GO_BACK)
                }),
        )
        .with_flex_child(title_bar, 1.0)
        .with_child(
            Label::new("\u{E921}")
                .with_font(ICONS_FONT)
                .padding((15., 5.))
                .on_click(|ctx, _, _| {
                    ctx.submit_command(CONFIGURE_WINDOW.with(
                        WindowConfig::default().set_window_state(druid::WindowState::MINIMIZED),
                    ))
                }),
        )
        .with_child(
            Label::new("\u{E922}")
                .with_font(ICONS_FONT)
                .padding((15., 5.))
                .on_click(|ctx, _, _| {
                    ctx.submit_command(CONFIGURE_WINDOW.with(
                        WindowConfig::default().set_window_state(druid::WindowState::MAXIMIZED),
                    ))
                }),
        )
        .with_child(
            Label::new("\u{E8BB}")
                .with_font(ICONS_FONT)
                .padding((15., 5.))
                .on_click(|ctx, _, _| ctx.submit_command(CLOSE_WINDOW)),
        )
        .background(Color::Rgba32(0x14191fff))
}

struct TBar;
impl<T, W: Widget<T>> Controller<T, W> for TBar {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        match event {
            // Event::MouseDown(_) => {
            // }
            // Event::MouseUp(_) => {}
            Event::MouseMove(_) => {
                ctx.window().handle_titlebar(true);
            }
            _ => {} // Event::Wheel(_) => {}
        }
        child.event(ctx, event, data, env)
    }
}
