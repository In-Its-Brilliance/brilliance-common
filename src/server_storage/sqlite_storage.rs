use std::path::PathBuf;

use crate::{inventory::inventory::Inventory, utils::srotage_settings::StorageSettings};

use super::taits::{IServerStorage, ServerInventoryOwner};

pub struct SQLiteServerStorage {
    _db_path: PathBuf,
}

impl IServerStorage for SQLiteServerStorage {
    type Error = String;
    type PrimaryKey = i64;

    fn init(storage_settings: StorageSettings) -> Result<Self, String> {
        let mut db_path = storage_settings.get_data_path().to_path_buf();

        db_path.push("data.db");

        let storage = Self { _db_path: db_path };
        Ok(storage)
    }

    fn load_inventories(&self, _owner: &ServerInventoryOwner) -> Result<Vec<Inventory>, Self::Error> {
        todo!()
    }

    fn save_inventory(&self, _inventory: &Inventory) -> Result<(), Self::Error> {
        todo!()
    }

    fn create_inventory(&self, _owner: &ServerInventoryOwner) -> Result<(), Self::Error> {
        todo!()
    }

    fn delete_inventory(&self, _inventory: &Inventory) -> Result<(), Self::Error> {
        todo!()
    }
}
