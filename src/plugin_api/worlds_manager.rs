use crate::{
    chunks::{block_position::BlockPosition, chunk_data::BlockDataInfo},
    plugin_api::inventory::Inventory,
    serde_json,
};

#[cfg(feature = "wasm-plugin")]
#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn has_world_raw(slug: String) -> String;
    fn create_world_raw(slug: String) -> ();
    fn edit_world_block_raw(world_slug: String, position_json: String, new_block_info_json: String) -> ();
    fn get_or_create_inventory_raw(world_slug: String, position_json: String, slots_count: u64) -> String;
}

#[derive(Default)]
pub struct WorldsManager;

#[derive(Clone)]
pub struct WorldManager {
    slug: String,
}

#[derive(Clone)]
pub struct ChunksMap {
    world_slug: String,
}

impl WorldsManager {
    pub fn has_world(&self, slug: &str) -> Result<bool, extism_pdk::Error> {
        let result = unsafe { has_world_raw(slug.to_string())? };
        Ok(result == "true")
    }

    pub fn create_world(&self, slug: &str) -> Result<(), extism_pdk::Error> {
        unsafe { create_world_raw(slug.to_string()) }
    }

    pub fn get_world_manager(&self, slug: &str) -> Result<WorldManager, extism_pdk::Error> {
        if !self.has_world(slug)? {
            return Err(extism_pdk::Error::msg(format!("World \"{}\" not found", slug)));
        }
        Ok(WorldManager::create(slug.to_string()))
    }
}

impl WorldManager {
    pub fn create(slug: String) -> Self {
        Self { slug }
    }

    pub fn get_slug(&self) -> &String {
        &self.slug
    }

    pub fn get_chunks_map(&self) -> ChunksMap {
        ChunksMap::create(self.slug.clone())
    }
}

impl ChunksMap {
    pub fn create(world_slug: String) -> Self {
        Self { world_slug }
    }

    pub fn edit_block(
        &self,
        position: BlockPosition,
        new_block_info: Option<BlockDataInfo>,
    ) -> Result<(), extism_pdk::Error> {
        let position_json = serde_json::to_string(&position)
            .map_err(|e| extism_pdk::Error::msg(format!("Failed to serialize block position: {}", e)))?;
        let new_block_info_json = serde_json::to_string(&new_block_info)
            .map_err(|e| extism_pdk::Error::msg(format!("Failed to serialize block data: {}", e)))?;

        unsafe {
            edit_world_block_raw(self.world_slug.clone(), position_json, new_block_info_json)?;
        }
        Ok(())
    }

    pub fn get_or_create_inventory(
        &self,
        position: BlockPosition,
        slots_count: usize,
    ) -> Result<Inventory, extism_pdk::Error> {
        let position_json = serde_json::to_string(&position)
            .map_err(|e| extism_pdk::Error::msg(format!("Failed to serialize block position: {}", e)))?;
        let inventory_id = unsafe { get_or_create_inventory_raw(self.world_slug.clone(), position_json, slots_count as u64)? };
        let id = inventory_id
            .parse::<u64>()
            .map_err(|e| extism_pdk::Error::msg(format!("Invalid inventory id: {}", e)))?;
        Ok(Inventory::from_id(id))
    }
}
