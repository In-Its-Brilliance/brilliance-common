pub mod blocks;
pub mod chunks;
pub mod commands;
pub mod default_blocks;
pub mod default_blocks_ids;
pub mod default_resources;
pub mod inventory;
#[cfg(feature = "full")]
pub mod server_storage;
pub mod utils;
pub mod world_generator;
#[cfg(feature = "full")]
pub mod worlds_storage;
#[cfg(feature = "full")]
use server_storage::sqlite_storage::SQLiteServerStorage;
#[cfg(feature = "full")]
use worlds_storage::redb_storage::RedbWorldStorage;

#[cfg(feature = "full")]
pub type WorldStorageManager = RedbWorldStorage;

#[cfg(feature = "full")]
pub type ServerStorageManager = SQLiteServerStorage;

/// Целевой тикрейт сервера (тиков в секунду).
pub const TARGET_TPS: f64 = 64.0;

pub const CHUNK_SIZE: u8 = 16_u8;
pub const CHUNK_SIZE_BOUNDARY: u32 = CHUNK_SIZE as u32 + 2;
pub const SECTION_VOLUME: usize = CHUNK_SIZE as usize * CHUNK_SIZE as usize * CHUNK_SIZE as usize;
pub const VERTICAL_SECTIONS: usize = 16;

pub const INVENTORY_SLOTS: usize = 54;

pub const INVENTORY_BASE: usize = 35;
pub const HOTBAR_SLOTS: usize = 7;
pub const SPECIAL_INVENTORY_HEAD_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS;
pub const SPECIAL_INVENTORY_CHEST_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 1;
pub const SPECIAL_INVENTORY_PANTS_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 2;
pub const SPECIAL_INVENTORY_BOOTS_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 3;
pub const SPECIAL_INVENTORY_NECK_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 4;
pub const SPECIAL_INVENTORY_BRACER_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 5;
pub const SPECIAL_INVENTORY_GLOVES_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 6;
pub const SPECIAL_INVENTORY_OFFHAND_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 7;
pub const SPECIAL_INVENTORY_BELT_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 8;
pub const SPECIAL_INVENTORY_RUNE_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 9;
pub const SPECIAL_INVENTORY_FINGER_0_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 10;
pub const SPECIAL_INVENTORY_FINGER_1_SLOT: usize = INVENTORY_SLOTS + HOTBAR_SLOTS + 11;

pub mod plugin_api;

#[cfg(feature = "wasm-plugin")]
pub use brilliance_macros::event_handler;

#[cfg(feature = "wasm-plugin")]
pub use extism_pdk;

#[cfg(feature = "wasm-plugin")]
pub use serde_json;
