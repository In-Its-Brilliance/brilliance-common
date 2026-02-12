use crate::chunks::{
    chunk_data::{BlockIndexType, WorldMacroData},
    chunk_position::ChunkPosition,
};
use std::{collections::BTreeMap, path::PathBuf};

/// Essential world metadata and generation parameters
/// required for world creation and chunk generation.
#[derive(Clone, Default)]
pub struct WorldStorageData {
    slug: String,
    seed: u64,
    world_generator: String,
    world_macro_data: WorldMacroData,
}

impl WorldStorageData {
    pub fn create(
        slug: impl Into<String>,
        seed: u64,
        world_generator: impl Into<String>,
        world_macro_data: WorldMacroData,
    ) -> Self {
        Self {
            slug: slug.into(),
            seed,
            world_generator: world_generator.into(),
            world_macro_data,
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

    pub fn get_world_macro_data(&self) -> &WorldMacroData {
        &self.world_macro_data
    }
}

#[derive(Clone)]
pub struct WorldStorageSettings {
    data_path: PathBuf,
}

impl WorldStorageSettings {
    pub fn from_path(data_path: PathBuf) -> Self {
        Self { data_path }
    }

    pub fn in_memory() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let settings = Self {
            data_path: tmp.path().to_path_buf(),
        };
        settings
    }

    pub fn get_data_path(&self) -> &PathBuf {
        &self.data_path
    }
}

pub trait IWorldStorage: Sized {
    type Error;
    type PrimaryKey;

    fn init(storage_settings: WorldStorageSettings, slug: impl Into<String>) -> Result<Self, Self::Error>;

    fn create_new(&self, world_info: &WorldStorageData) -> Result<(), String>;

    fn has_chunk_data(&self, chunk_position: &ChunkPosition) -> Result<Option<Self::PrimaryKey>, String>;
    fn read_chunk_data(&self, chunk_id: Self::PrimaryKey) -> Result<Vec<u8>, String>;
    fn save_chunk_data(&self, chunk_position: &ChunkPosition, data: &Vec<u8>) -> Result<Self::PrimaryKey, String>;
    fn delete(&self) -> Result<(), String>;

    fn scan_worlds(storage_settings: WorldStorageSettings) -> Result<Vec<WorldStorageData>, String>;

    fn validate_block_id_map(&self, block_id_map: &BTreeMap<BlockIndexType, String>) -> Result<(), String>;
}
