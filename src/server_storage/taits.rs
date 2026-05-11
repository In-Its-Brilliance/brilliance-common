use crate::{
    inventory::inventory::Inventory,
    utils::srotage_settings::StorageSettings,
};

pub enum ServerInventoryOwner {
    Player {
        player_id: u64,
    },
}

pub trait IServerStorage: Sized {
    type Error;
    type PrimaryKey;

    fn init(storage_settings: StorageSettings) -> Result<Self, Self::Error>;

    fn load_inventories(&self, owner: &ServerInventoryOwner) -> Result<Vec<Inventory>, Self::Error>;
    fn save_inventory(&self, inventory: &Inventory) -> Result<(), Self::Error>;
    fn create_inventory(&self, owner: &ServerInventoryOwner) -> Result<(), Self::Error>;
    fn delete_inventory(&self, inventory: &Inventory) -> Result<(), Self::Error>;
}
