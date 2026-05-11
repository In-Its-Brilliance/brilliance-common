use super::item::Item;

#[derive(Default, Clone)]
pub struct Inventory {
    pub slots: Vec<Option<Item>>,
}

impl Inventory {
    pub fn create(slots: Vec<Option<Item>>) -> Self {
        Self { slots }
    }

    pub fn non_empty_slots(&self) {
    }
}
