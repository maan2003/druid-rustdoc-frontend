use druid::piet::StrokeStyle;
use druid::text::TextStorage;
use druid::widget::{Either, Flex, LineBreaking, Maybe, RawLabel, SizedBox};
use druid::{Color, Data, Env, KeyOrValue, Widget, WidgetExt};
use druid_widget_nursery::{ComputedWidget, Seperator};

use crate::theme;

pub trait ComputedExt<T: Data>: Widget<T> + Sized + 'static {
    fn computed<U>(self, f: impl Fn(&U) -> T + 'static) -> ComputedWidget<U, T> {
        ComputedWidget::new(self, f)
    }
    fn or_empty(self) -> Maybe<T>
    where
        Self: Clone,
    {
        Maybe::or_empty(move || self.clone())
    }

    fn empty_if(self, f: impl Fn(&T, &Env) -> bool + 'static) -> Either<T> {
        Either::new(f, SizedBox::empty(), self)
    }
}

impl<T: Data, W: Widget<T> + 'static> ComputedExt<T> for W {}

pub trait SeperatorExt<T: Data> {
    fn seperator(self, level: u8) -> Self;
}

impl<T: Data> SeperatorExt<T> for Flex<T> {
    fn seperator(self, level: u8) -> Self {
        self.with_child(seperator(level))
    }
}
pub trait RawLabelExt<T: Data> {
    fn code() -> Self;
    fn wrap_text(self) -> Self;
    fn color(self, color: impl Into<KeyOrValue<Color>>) -> Self;
}

impl<T: Data + TextStorage> RawLabelExt<T> for RawLabel<T> {
    fn code() -> Self {
        RawLabel::new().with_font(theme::CODE_FONT)
    }
    fn wrap_text(self) -> Self {
        self.with_line_break_mode(LineBreaking::WordWrap)
    }
    fn color(self, color: impl Into<KeyOrValue<Color>>) -> Self {
        self.with_text_color(color)
    }
}

fn seperator<T: Data>(level: u8) -> impl Widget<T> {
    match level {
        1 => Seperator::new()
            .with_size(1.)
            .with_stroke_style(StrokeStyle::new().dash_pattern(&[2.]))
            .padding((0., 5.)),
        2 => Seperator::new().with_size(1.).padding((0., 5., 0., 10.)),
        _ => unreachable!(),
    }
}
