use rand::Rng;

use crate::chunks::{chunk_data::BlockIndexType, chunk_position::ChunkPosition};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone)]
pub struct WorldInfo {
    slug: String,
    seed: u64,
    world_generator: String,
}

impl WorldInfo {
    pub fn create(slug: impl Into<String>, seed: Option<u64>, world_generator: impl Into<String>) -> Self {
        let seed = match seed {
            Some(s) => s,
            None => rand::thread_rng().gen(),
        };
        Self {
            slug: slug.into(),
            seed,
            world_generator: world_generator.into(),
        }
    }

    pub fn get_slug(&self) -> &String {
        &self.slug
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn get_world_generator(&self) -> &String {
        &self.world_generator
    }
}

#[derive(Clone)]
pub struct WorldStorageSettings {
    data_path: PathBuf,
}

impl WorldStorageSettings {
    pub fn create(data_path: PathBuf) -> Self {
        Self { data_path }
    }

    pub fn get_data_path(&self) -> &PathBuf {
        &self.data_path
    }
}

pub trait IWorldStorage: Sized {
    type Error;
    type PrimaryKey;

    fn create(storage_settings: WorldStorageSettings, world_info: &WorldInfo) -> Result<Self, Self::Error>;
    fn has_chunk_data(&self, chunk_position: &ChunkPosition) -> Result<Option<Self::PrimaryKey>, String>;
    fn read_chunk_data(&self, chunk_id: Self::PrimaryKey) -> Result<Vec<u8>, String>;
    fn save_chunk_data(&self, chunk_position: &ChunkPosition, data: &Vec<u8>) -> Result<Self::PrimaryKey, String>;
    fn delete(&self) -> Result<(), String>;

    fn scan_worlds(storage_settings: WorldStorageSettings) -> Result<Vec<WorldInfo>, String>;

    fn validate_block_id_map(
        world_slug: String,
        storage_settings: WorldStorageSettings,
        block_id_map: &BTreeMap<BlockIndexType, String>,
    ) -> Result<(), String>;
}
