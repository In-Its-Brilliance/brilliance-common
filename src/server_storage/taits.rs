use std::time::{SystemTime, UNIX_EPOCH};

use crate::{inventory::inventory::Inventory, utils::srotage_settings::StorageSettings};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlayerData {
    username: String,
    json: JsonValue,
    inventory: Inventory,
    created_at: u64,
    last_login_at: u64,
    updated_at: u64,
}

impl PlayerData {
    pub fn create(username: impl Into<String>, json: JsonValue, inventory: Inventory) -> Self {
        let now = current_unix_timestamp();

        Self::create_with_timestamps(username, json, inventory, now, now, now)
    }

    pub fn create_with_timestamps(
        username: impl Into<String>,
        json: JsonValue,
        inventory: Inventory,
        created_at: u64,
        last_login_at: u64,
        updated_at: u64,
    ) -> Self {
        Self {
            username: username.into(),
            json,
            inventory,
            created_at,
            last_login_at,
            updated_at,
        }
    }

    pub fn get_username(&self) -> &String {
        &self.username
    }

    pub fn get_json(&self) -> &JsonValue {
        &self.json
    }

    pub fn get_json_mut(&mut self) -> &mut JsonValue {
        &mut self.json
    }

    pub fn get_inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub fn get_inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
    }

    pub fn get_created_at(&self) -> u64 {
        self.created_at
    }

    pub fn get_last_login_at(&self) -> u64 {
        self.last_login_at
    }

    pub fn get_updated_at(&self) -> u64 {
        self.updated_at
    }
}

pub fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerInventoryOwner {
    Player(String),
}

impl ServerInventoryOwner {
    pub fn player(username: impl Into<String>) -> Self {
        Self::Player(username.into())
    }
}

pub trait IServerStorage: Sized {
    type Error;
    type PrimaryKey;

    fn init(storage_settings: StorageSettings) -> Result<Self, Self::Error>;

    fn get_or_create_player_data(&self, username: impl Into<String>) -> Result<PlayerData, Self::Error>;
    fn save_player_data(&self, player_data: &PlayerData) -> Result<Self::PrimaryKey, Self::Error>;
}
