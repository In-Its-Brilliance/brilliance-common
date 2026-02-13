use serde::{Deserialize, Serialize};

use crate::{chunks::{
    chunk_data::{ChunkData, WorldMacroData},
    chunk_position::ChunkPosition,
}, worlds_storage::taits::WorldStorageData};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WorldGeneratorSettings {
    seed: u64,
    method: String,
    settings: Option<serde_yaml::Value>,
    world_macro_data: WorldMacroData,
}

impl WorldGeneratorSettings {
    pub fn create(
        seed: u64,
        method: impl Into<String>,
        settings: Option<serde_yaml::Value>,
        world_macro_data: WorldMacroData,
    ) -> Self {
        Self {
            seed,
            settings,
            method: method.into(),
            world_macro_data,
        }
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn get_method(&self) -> &String {
        &self.method
    }

    pub fn get_settings(&self) -> &Option<serde_yaml::Value> {
        &self.settings
    }

    pub fn get_world_macro_data(&self) -> &WorldMacroData {
        &self.world_macro_data
    }
}

impl From<&WorldStorageData> for WorldGeneratorSettings {
    fn from(data: &WorldStorageData) -> Self {
        Self {
            seed: data.get_seed(),
            method: data.get_world_generator().clone(),
            settings: None,
            world_macro_data: data.get_world_macro_data().clone(),
        }
    }
}

pub trait IWorldGenerator: Sized {
    fn generate_chunk_data(world_settings: &WorldGeneratorSettings, chunk_position: &ChunkPosition) -> ChunkData;
}
