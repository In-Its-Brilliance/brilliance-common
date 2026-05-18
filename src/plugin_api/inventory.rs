use serde::{Deserialize, Serialize};

use crate::{
    inventory::{
        item::Item,
    },
    plugin_api::player::Player,
    serde_json,
};

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
    fn open_inventory_raw(request_json: String) -> ();
    fn add_inventory_item_raw(inventory_id: u64, item_json: String) -> String;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AddItemError {
    Full,
    NotFound,
}

impl Inventory {
    pub(crate) fn from_existing_id(id: u64) -> Self {
        Self { id }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    #[cfg(feature = "wasm-plugin")]
    pub fn add_item(&self, item: Item) -> Result<(), AddItemError> {
        let item_json = serde_json::to_string(&item)
            .unwrap_or_else(|e| panic!("Failed to serialize item: {}", e));
        let response = unsafe { add_inventory_item_raw(self.id, item_json) }
            .unwrap_or_else(|e| panic!("Failed to add item to inventory: {}", e));
        match response.as_str() {
            "ok" => Ok(()),
            "full" => Err(AddItemError::Full),
            "not_found" => Err(AddItemError::NotFound),
            other => panic!("Invalid inventory add item response: {}", other),
        }
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
