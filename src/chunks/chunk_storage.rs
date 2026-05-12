use serde::{Deserialize, Serialize};

use crate::{
    inventory::{inventory::Inventory, item::Item},
    utils::compressable::Compressable,
};

use super::{block_position::ChunkBlockPosition, chunk_data::ChunkData, position::Vector3};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ChunkStorage {
    chunk_data: Box<ChunkData>,
    inventories: Vec<BlockInventory>,
    items: Vec<WorldItem>,
}

impl Compressable for ChunkStorage {}

impl ChunkStorage {
    pub fn create(chunk_data: ChunkData) -> Self {
        Self {
            chunk_data: Box::new(chunk_data),
            inventories: Default::default(),
            items: Default::default(),
        }
    }

    pub fn inventories(mut self, inventories: Vec<BlockInventory>) -> Self {
        self.inventories = inventories;
        self
    }

    pub fn items(mut self, items: Vec<WorldItem>) -> Self {
        self.items = items;
        self
    }

    pub fn get_chunk_data(&self) -> &ChunkData {
        &self.chunk_data
    }

    pub fn get_chunk_data_mut(&mut self) -> &mut ChunkData {
        &mut self.chunk_data
    }

    pub fn add_inventory(&mut self, block_inventory: BlockInventory) {
        self.inventories.push(block_inventory)
    }

    pub fn add_item(&mut self, world_item: WorldItem) {
        self.items.push(world_item)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockInventory {
    section: u32,
    position: ChunkBlockPosition,
    inventory: Inventory,
}

impl BlockInventory {
    pub fn create(section: u32, position: ChunkBlockPosition, inventory: Inventory) -> Self {
        Self {
            section,
            position,
            inventory,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldItem {
    position: Vector3,
    item: Item,
}

impl WorldItem {
    pub fn create(position: Vector3, item: Item) -> Self {
        Self { position, item }
    }
}
