use serde::{Deserialize, Serialize};

use super::item::{ClientItem, Item};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum InventoryAddItemError {
    Full,
}

/// Serves as an index on the client's side
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum InventoryType {
    // Player its own inventory
    PlayerPersonal,

    // Other player. Client id
    OtherPlayer(u64),

    // Inventory id
    WorldInventory(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInventory {
    pub slots: Vec<Option<ClientItem>>,
}

impl ClientInventory {
    pub fn create(slots_count: usize) -> Self {
        Self {
            slots: vec![None; slots_count],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Inventory {
    id: u64,
    slots: Vec<Option<Item>>,
}

impl Inventory {
    pub fn create(slots_count: usize) -> Self {
        Self {
            id: 0,
            slots: vec![None; slots_count],
        }
    }

    pub fn with_id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn slots_len(&self) -> usize {
        self.slots.len()
    }

    pub fn iter_slots(&self) -> impl Iterator<Item = &Option<Item>> {
        self.slots.iter()
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

    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut Item> {
        self.slots.get_mut(index).and_then(Option::as_mut)
    }

    pub fn take_slot(&mut self, index: usize) -> Option<Item> {
        self.slots.get_mut(index).and_then(Option::take)
    }

    pub fn set_slot_option(&mut self, index: usize, item: Option<Item>) {
        if index >= self.slots.len() {
            self.slots.resize_with(index + 1, || None);
        }
        self.slots[index] = item;
    }

    pub fn swap_slots(&mut self, a_index: usize, b_index: usize) {
        if a_index >= self.slots.len() {
            self.slots.resize_with(a_index + 1, || None);
        }
        if b_index >= self.slots.len() {
            self.slots.resize_with(b_index + 1, || None);
        }
        self.slots.swap(a_index, b_index);
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
    }

    pub fn add_item(
        &mut self,
        item: Item,
        max_stack_size: u16,
        mut emit: impl FnMut(usize, Option<&Item>),
    ) -> Result<(), InventoryAddItemError> {
        if item.get_amount() == 0 {
            return Ok(());
        }

        let mut remaining = item.get_amount();
        let mut total_capacity = 0u32;

        for slot in &self.slots {
            match slot {
                Some(existing) if existing.can_stack_with(&item) => {
                    total_capacity += u32::from(max_stack_size.saturating_sub(existing.get_amount()));
                }
                None => {
                    total_capacity += u32::from(max_stack_size);
                }
                _ => {}
            }

            if total_capacity >= u32::from(remaining) {
                break;
            }
        }

        if total_capacity < u32::from(remaining) {
            return Err(InventoryAddItemError::Full);
        }

        for slot_index in 0..self.slots.len() {
            if remaining == 0 {
                break;
            }

            let Some(existing) = self.slots[slot_index].as_ref() else {
                continue;
            };
            if !existing.can_stack_with(&item) {
                continue;
            }

            let space = max_stack_size.saturating_sub(existing.get_amount());
            if space == 0 {
                continue;
            }

            let added = remaining.min(space);
            let updated = existing.clone().amount(existing.get_amount() + added);
            self.slots[slot_index] = Some(updated);
            remaining -= added;
            emit(slot_index, self.slots[slot_index].as_ref());
        }

        for slot_index in 0..self.slots.len() {
            if remaining == 0 {
                break;
            }

            if self.slots[slot_index].is_some() {
                continue;
            }

            let added = remaining.min(max_stack_size);
            let updated = item.clone().amount(added);
            self.slots[slot_index] = Some(updated);
            remaining -= added;
            emit(slot_index, self.slots[slot_index].as_ref());
        }

        Ok(())
    }
}
