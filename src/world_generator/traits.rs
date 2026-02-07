use rand::Rng;
use serde_json::Value;

use crate::chunks::{chunk_data::ChunkData, chunk_position::ChunkPosition};

#[derive(Clone)]
pub struct WorldGeneratorSettings {
    seed: u64,
    method: String,
    settings: Option<Value>,
}

impl WorldGeneratorSettings {
    pub fn create(seed: Option<u64>, method: impl Into<String>, settings: Option<Value>) -> Self {
        let seed = match seed {
            Some(s) => s,
            None => rand::thread_rng().gen(),
        };
        Self { seed, settings, method: method.into() }
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn get_method(&self) -> &String {
        &self.method
    }

    pub fn get_settings(&self) -> &Option<Value> {
        &self.settings
    }
}

pub trait IWorldGenerator: Sized {
    fn generate_chunk_data(world_settings: &WorldGeneratorSettings, chunk_position: &ChunkPosition) -> ChunkData;
}
