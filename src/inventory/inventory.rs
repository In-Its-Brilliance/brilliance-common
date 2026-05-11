use super::item::Item;

#[derive(Default)]
pub struct Inventory {
    pub slots: Vec<Option<Item>>,
}

impl Inventory {
    pub fn create(slots: Vec<Option<Item>>) -> Self {
        Self { slots }
    }
}
