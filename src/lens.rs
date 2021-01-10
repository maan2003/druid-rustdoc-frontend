use druid::Data;
use druid::Lens;
use rustdoc_types::{Item, ItemSummary};

use crate::AppData;
pub struct IdLens;

impl Lens<AppData, (Item, Option<ItemSummary>)> for IdLens {
    fn with<V, F: FnOnce(&(Item, Option<ItemSummary>)) -> V>(&self, data: &AppData, f: F) -> V {
        let value = data.krate.index.get(&data.current_item).unwrap();
        let sum = data.krate.paths.get(&data.current_item);
        f(&(value.clone(), sum.cloned()))
    }

    fn with_mut<V, F: FnOnce(&mut (Item, Option<ItemSummary>)) -> V>(&self, data: &mut AppData, f: F) -> V {
        let value = data.krate.index.get(&data.current_item).unwrap();
        let mut value_clone = value.clone();
        let sum = data.krate.paths.get(&data.current_item);

        f(&mut (value.clone(), sum.cloned()))
    }
}
