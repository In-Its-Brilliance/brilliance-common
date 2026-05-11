use super::item::Item;

pub struct Inventory {
    pub slots: Vec<Option<Item>>,
}
