use serde::{Deserialize, Serialize};

use super::item::Item;

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct Inventory {
    pub slots: Vec<Option<Item>>,
}

impl Inventory {
    pub fn create(slots: Vec<Option<Item>>) -> Self {
        Self { slots }
    }
}
