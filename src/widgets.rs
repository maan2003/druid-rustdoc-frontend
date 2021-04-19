use druid::Widget;
use druid_widget_nursery::ComputedWidget;

pub trait ComputedExt<T>: Widget<T> + Sized + 'static {
    fn computed<U>(self, f: impl Fn(&U) -> T + 'static) -> ComputedWidget<U, T> {
        ComputedWidget::new(self, f)
    }
}

pub trait ComputedExt2<T>: Widget<T> + Default + Sized + 'static {
    fn computed<U>(f: impl Fn(&U) -> T + 'static) -> ComputedWidget<U, T> {
        ComputedWidget::new(Self::default(), f)
    }
}

impl<T, W: Widget<T> + 'static> ComputedExt<T> for W {}
impl<T, W: Widget<T> + Default + 'static> ComputedExt2<T> for W {}
