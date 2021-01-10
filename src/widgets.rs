use druid::{
    kurbo::Line, piet::StrokeStyle, Env, Event, LifeCycle, Point, Selector, Target, Widget,
    WidgetId, WidgetPod,
};
use rustdoc_types::{Id, Item, ItemSummary};

pub struct ItemsWidget {
    items: Option<Vec<Id>>,
    func: Box<dyn Fn(Vec<(Item, Option<ItemSummary>)>, &Env) -> Box<dyn Widget<()>>>,
    widget: Option<WidgetPod<(), Box<dyn Widget<()>>>>,
}

impl ItemsWidget {
    pub fn new(
        items: Vec<Id>,
        func: impl Fn(Vec<(Item, Option<ItemSummary>)>, &Env) -> Box<dyn Widget<()>> + 'static,
    ) -> Self {
        Self {
            items: Some(items),
            func: Box::new(func),
            widget: None,
        }
    }
}

pub const ID_QUERY: Selector<(Vec<Id>, WidgetId)> = Selector::new("druid-rustdoc.id-query");
pub const ID_QUERY_RESPONSE: Selector<Vec<(Item, Option<ItemSummary>)>> = Selector::new("druid-rustdoc.id-query-response");

impl Widget<()> for ItemsWidget {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &Event, data: &mut (), env: &druid::Env) {
        if let Event::Command(c) = event {
            if let Some(data) = c.get(ID_QUERY_RESPONSE) {
                let widget = (self.func)(data.clone(), env);
                self.widget = Some(WidgetPod::new(widget));
                ctx.children_changed();
                return;
            }
        }
        if let Some(w) = &mut self.widget {
            w.event(ctx, event, data, env);
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &LifeCycle,
        data: &(),
        env: &druid::Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            ctx.submit_command(
                ID_QUERY
                    .with((self.items.take().unwrap(), ctx.widget_id()))
                    .to(Target::Global),
            );
        }
        if let Some(w) = &mut self.widget {
            w.lifecycle(ctx, event, &(), env);
        }
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &(), data: &(), env: &druid::Env) {
        if let Some(w) = &mut self.widget {
            w.update(ctx, &(), env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &(),
        env: &druid::Env,
    ) -> druid::Size {
        if let Some(w) = &mut self.widget {
            let size = w.layout(ctx, bc, &(), env);
            w.set_origin(ctx, data, env, Point::new(0.0, 0.0));
            size
        } else {
            bc.constrain((0., 0.))
        }
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &(), env: &druid::Env) {
        if let Some(w) = &mut self.widget {
            w.paint(ctx, &(), env);
        }
    }
}

use druid::{theme, widget::prelude::*};
use druid::{Color, KeyOrValue};

pub struct Seperator {
    size: KeyOrValue<f64>,
    color: KeyOrValue<Color>,
    orientation: Orientation,
    stroke_style: StrokeStyle,
}

impl Seperator {
    pub fn new() -> Self {
        Seperator {
            size: theme::BUTTON_BORDER_WIDTH.into(),
            color: theme::BORDER_LIGHT.into(),
            orientation: Orientation::Horizontal,
            stroke_style: StrokeStyle::new(),
        }
    }
    pub fn with_size(mut self, size: impl Into<KeyOrValue<f64>>) -> Self {
        self.size = size.into();
        self
    }

    pub fn set_size(&mut self, size: impl Into<KeyOrValue<f64>>) {
        self.size = size.into();
    }

    pub fn with_color(mut self, color: impl Into<KeyOrValue<Color>>) -> Self {
        self.color = color.into();
        self
    }

    pub fn with_stroke_style(mut self, stroke_style: StrokeStyle) -> Self {
        self.stroke_style = stroke_style;
        self
    }

    pub fn set_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
        self.color = color.into();
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}

pub enum Orientation {
    Vertical,
    Horizontal,
}

impl<T> Widget<T> for Seperator {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut T, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        let size = self.size.resolve(env);
        let size = match self.orientation {
            Orientation::Vertical => (size, f64::INFINITY),
            Orientation::Horizontal => (f64::INFINITY, size),
        };
        bc.constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        let line = Line::new((0., 0.), ctx.size().to_vec2().to_point());

        let color = self.color.resolve(env);
        ctx.stroke_styled(line, &color, self.size.resolve(env), &self.stroke_style);
    }
}
