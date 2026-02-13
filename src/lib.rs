pub mod blocks;
pub mod chunks;
pub mod commands;
pub mod default_blocks;
pub mod default_blocks_ids;
pub mod default_resources;
pub mod utils;
pub mod world_generator;
pub mod worlds_storage;

#[cfg(feature = "full")]
use worlds_storage::sqlite_storage::SQLiteStorage;

#[cfg(feature = "full")]
pub type WorldStorageManager = SQLiteStorage;

pub const CHUNK_SIZE: u8 = 16_u8;
pub const CHUNK_SIZE_BOUNDARY: u32 = CHUNK_SIZE as u32 + 2;
pub const SECTION_VOLUME: usize = CHUNK_SIZE as usize * CHUNK_SIZE as usize * CHUNK_SIZE as usize;
pub const VERTICAL_SECTIONS: usize = 16;

#[cfg(feature = "wasm-plugin")]
pub mod plugin_api;

#[cfg(feature = "wasm-plugin")]
pub use brilliance_macros::event_handler;

#[cfg(feature = "wasm-plugin")]
pub use extism_pdk;

#[cfg(feature = "wasm-plugin")]
pub use serde_json;
