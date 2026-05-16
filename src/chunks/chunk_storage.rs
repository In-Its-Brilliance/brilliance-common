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

    pub fn get_inventories(&self) -> &[BlockInventory] {
        &self.inventories
    }

    pub fn get_inventories_mut(&mut self) -> &mut [BlockInventory] {
        &mut self.inventories
    }

    pub fn get_inventory_mut(&mut self, inventory_id: u64) -> Option<&mut BlockInventory> {
        self.inventories
            .iter_mut()
            .find(|block_inventory| block_inventory.get_inventory().get_id() == inventory_id)
    }

    pub fn get_inventory_by_position_mut(
        &mut self,
        section: u32,
        position: &ChunkBlockPosition,
    ) -> Option<&mut BlockInventory> {
        self.inventories
            .iter_mut()
            .find(|block_inventory| block_inventory.get_section() == section && block_inventory.get_position() == position)
    }

    pub fn get_or_create_inventory_by_position_mut(
        &mut self,
        section: u32,
        position: ChunkBlockPosition,
        slots_count: usize,
        inventory_id: u64,
    ) -> &mut BlockInventory {
        if let Some(index) = self
            .inventories
            .iter()
            .position(|block_inventory| block_inventory.get_section() == section && block_inventory.get_position() == &position)
        {
            return &mut self.inventories[index];
        }

        self.inventories.push(BlockInventory::create(
            section,
            position,
            Inventory::create(slots_count).with_id(inventory_id),
        ));
        self.inventories.last_mut().unwrap()
    }

    pub fn get_or_create_inventory_mut(
        &mut self,
        inventory_id: u64,
        section: u32,
        position: ChunkBlockPosition,
        slots_count: usize,
    ) -> &mut BlockInventory {
        if let Some(index) = self
            .inventories
            .iter()
            .position(|block_inventory| block_inventory.get_inventory().get_id() == inventory_id)
        {
            return &mut self.inventories[index];
        }

        self.inventories.push(BlockInventory::create(
            section,
            position,
            Inventory::create(slots_count).with_id(inventory_id),
        ));
        self.inventories.last_mut().unwrap()
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

    pub fn get_section(&self) -> u32 {
        self.section
    }

    pub fn get_position(&self) -> &ChunkBlockPosition {
        &self.position
    }

    pub fn get_inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub fn get_inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
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
