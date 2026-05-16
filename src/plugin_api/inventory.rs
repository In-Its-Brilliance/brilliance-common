use serde::{Deserialize, Serialize};

use crate::{plugin_api::player::Player, serde_json};

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenInventoryRequest {
    client_id: u64,
    inventory_id: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Inventory {
    id: u64,
}

#[cfg(feature = "wasm-plugin")]
#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn create_inventory_raw() -> String;
    fn open_inventory_raw(request_json: String) -> ();
}

impl Inventory {
    pub fn create() -> Result<Self, extism_pdk::Error> {
        let inventory_id = unsafe { create_inventory_raw()? };
        let id = inventory_id
            .parse::<u64>()
            .map_err(|e| extism_pdk::Error::msg(format!("Invalid inventory id: {}", e)))?;
        Ok(Self { id })
    }

    pub fn from_id(id: u64) -> Self {
        Self { id }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}

impl OpenInventoryRequest {
    pub fn new(client_id: u64, inventory_id: u64) -> Self {
        Self {
            client_id,
            inventory_id,
        }
    }

    pub fn get_client_id(&self) -> u64 {
        self.client_id
    }

    pub fn get_inventory_id(&self) -> u64 {
        self.inventory_id
    }
}

impl Player {
    pub fn open_inventory(&self, inventory: Inventory) -> Result<(), extism_pdk::Error> {
        let request = OpenInventoryRequest::new(self.get_client_id(), inventory.get_id());
        let request_json = serde_json::to_string(&request)
            .map_err(|e| extism_pdk::Error::msg(format!("Failed to serialize open inventory request: {}", e)))?;
        unsafe { open_inventory_raw(request_json) }
    }
}
