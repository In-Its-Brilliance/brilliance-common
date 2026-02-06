use serde_json::Value;

use crate::chunks::{chunk_data::ChunkData, chunk_position::ChunkPosition};

#[derive(Default)]
pub struct WorldGeneratorSettings {
    seed: u64,
    settings: Value,
}

pub trait IWorldGenerator: Sized {
    fn generate_chunk_data(world_settings: &WorldGeneratorSettings, chunk_position: &ChunkPosition) -> ChunkData;
}
