use serde::{Deserialize, Serialize};

use super::item::{ClientItem, Item};

/// Serves as an index on the client's side
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum InventoryType {
    // Player its own inventory
    PlayerPersonal,

    // Other player login layout
    // OtherPlayer(String),

    // Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInventory {
    pub slots: Vec<Option<ClientItem>>,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct Inventory {
    slots: Vec<Option<Item>>,
}

impl Inventory {
    pub fn create(slots_count: usize) -> Self {
        Self {
            slots: vec![None; slots_count],
            ..Default::default()
        }
    }

    pub fn slots_len(&self) -> usize {
        self.slots.len()
    }

    pub fn set_slot(&mut self, index: usize, item: Item) {
        if index >= self.slots.len() {
            self.slots.resize_with(index + 1, || None);
        }
        self.slots[index] = Some(item);
    }

    pub fn get_slot(&self, index: usize) -> Option<&Item> {
        self.slots.get(index).and_then(Option::as_ref)
    }
}
