use crate::inventory::{inventory::Inventory, item::Item};

use super::{block_position::ChunkBlockPosition, chunk_data::ChunkData, position::Vector3};

pub struct ChunkStorage {
    world_data: Box<ChunkData>,
    inventories: Vec<BlockInventory>,
    items: Vec<WorldItem>,
}

impl ChunkStorage {
    pub(crate) fn create(world_data: ChunkData) -> Self {
        Self {
            world_data: Box::new(world_data),
            inventories: Default::default(),
            items: Default::default(),
        }
    }

    pub(crate) fn inventories(mut self, inventories: Vec<BlockInventory>) -> Self {
        self.inventories = inventories;
        self
    }

    pub(crate) fn items(mut self, items: Vec<WorldItem>) -> Self {
        self.items = items;
        self
    }

    pub fn get_chunk_data(&self) -> Box<ChunkData> {
        self.world_data.clone()
    }

    pub fn add_inventory(&mut self, block_inventory: BlockInventory) {
        self.inventories.push(block_inventory)
    }

    pub fn add_item(&mut self, world_item: WorldItem) {
        self.items.push(world_item)
    }
}

pub struct BlockInventory {
    section: u32,
    position: ChunkBlockPosition,
    inventory: Inventory,
}

impl BlockInventory {
    pub fn create(section: u32, position: ChunkBlockPosition, inventory: Inventory) -> Self {
        Self { section, position, inventory }
    }
}

pub struct WorldItem {
    position: Vector3,
    item: Item,
}

impl WorldItem {
    pub fn create(position: Vector3, item: Item) -> Self {
        Self { position, item }
    }
}
