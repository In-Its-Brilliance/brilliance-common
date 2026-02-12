use serde::{Deserialize, Serialize};

use crate::{chunks::chunk_position::ChunkPosition, world_generator::traits::WorldGeneratorSettings};

use super::PluginEvent;

#[derive(Serialize, Deserialize)]
pub struct ChunkGenerateEvent {
    chunk_position: ChunkPosition,
    world_settings: WorldGeneratorSettings,
}

impl PluginEvent for ChunkGenerateEvent {
    const EXPORT_NAME: &'static str = "on_chunk_generate";
}

impl ChunkGenerateEvent {
    pub fn create(chunk_position: ChunkPosition, world_settings: WorldGeneratorSettings) -> Self {
        Self {
            chunk_position,
            world_settings,
        }
    }

    pub fn get_chunk_position(&self) -> &ChunkPosition {
        &self.chunk_position
    }

    pub fn get_world_settings(&self) -> &WorldGeneratorSettings {
        &self.world_settings
    }
}
