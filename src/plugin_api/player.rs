use serde::{Deserialize, Serialize};

use crate::plugin_api::inventory::Inventory;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    client_id: u64,
}

#[cfg(feature = "wasm-plugin")]
#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn get_player_world_slug_raw(client_id: u64) -> String;
    fn get_player_inventory_raw(client_id: u64) -> String;
}

impl Player {
    pub(crate) fn create(client_id: u64) -> Self {
        Self { client_id }
    }

    pub fn get_client_id(&self) -> u64 {
        self.client_id
    }

    #[cfg(feature = "wasm-plugin")]
    pub fn get_world_slug(&self) -> Option<String> {
        let world_slug = unsafe { get_player_world_slug_raw(self.client_id) }
            .unwrap_or_else(|e| panic!("Failed to load player world slug: {}", e));
        if world_slug.is_empty() {
            None
        } else {
            Some(world_slug)
        }
    }

    #[cfg(feature = "wasm-plugin")]
    pub fn get_inventory(&self) -> Inventory {
        let inventory_id = unsafe { get_player_inventory_raw(self.client_id) }
            .unwrap_or_else(|e| panic!("Failed to load player inventory: {}", e));
        if inventory_id.is_empty() {
            panic!("Player inventory is missing")
        } else {
            let id = inventory_id
                .parse::<u64>()
                .unwrap_or_else(|e| panic!("Invalid inventory id: {}", e));
            Inventory::from_existing_id(id)
        }
    }
}
