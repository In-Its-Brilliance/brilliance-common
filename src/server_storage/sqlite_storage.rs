use std::path::PathBuf;

use crate::{inventory::inventory::Inventory, utils::srotage_settings::StorageSettings};

use super::taits::{IServerStorage, ServerInventoryOwner};

pub struct SQLiteServerStorage {
    db_path: PathBuf,
}

impl IServerStorage for SQLiteServerStorage {
    type Error = String;
    type PrimaryKey = i64;

    fn init(storage_settings: StorageSettings) -> Result<Self, String> {
        let mut db_path = storage_settings.get_data_path().clone();

        db_path.push("data.db");

        let storage = Self { db_path };
        Ok(storage)
    }

    fn load_inventories(&self, owner: &ServerInventoryOwner) -> Result<Vec<Inventory>, Self::Error> {
        todo!()
    }

    fn save_inventory(&self, inventory: &Inventory) -> Result<(), Self::Error> {
        todo!()
    }

    fn create_inventory(&self, owner: &ServerInventoryOwner) -> Result<(), Self::Error> {
        todo!()
    }

    fn delete_inventory(&self, inventory: &Inventory) -> Result<(), Self::Error> {
        todo!()
    }

    
}
