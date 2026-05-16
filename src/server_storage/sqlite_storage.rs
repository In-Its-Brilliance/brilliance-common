use std::{fs::create_dir_all, path::PathBuf};

use crate::{inventory::inventory::Inventory, utils::srotage_settings::StorageSettings};
use rusqlite::{Connection, OptionalExtension};
use serde_json::Value as JsonValue;

use super::taits::{current_unix_timestamp, IServerStorage, PlayerData};

const SQL_CREATE_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS players (
    username TEXT PRIMARY KEY,
    json TEXT NOT NULL CHECK (json_valid(json)),
    inventory TEXT NOT NULL CHECK (json_valid(inventory)),
    created_at INTEGER NOT NULL,
    last_login_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
";

const SQL_SELECT_PLAYER: &str =
    "SELECT username, json, inventory, created_at, last_login_at, updated_at FROM players WHERE username = ?1;";
const SQL_INSERT_PLAYER: &str =
    "INSERT INTO players (username, json, inventory, created_at, last_login_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6);";
const SQL_UPSERT_PLAYER: &str = "
INSERT INTO players (username, json, inventory, created_at, last_login_at, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)
ON CONFLICT(username) DO UPDATE SET
    json = excluded.json,
    inventory = excluded.inventory,
    last_login_at = excluded.last_login_at,
    updated_at = excluded.updated_at;
";
const SQL_SELECT_PLAYER_ROWID: &str = "SELECT rowid FROM players WHERE username = ?1;";

pub struct SQLiteServerStorage {
    db_path: PathBuf,
    _storage_settings: StorageSettings,
}

impl SQLiteServerStorage {
    fn open(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path).map_err(|e| e.to_string())
    }

    fn read_player(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlayerData> {
        let username = row.get::<_, String>(0)?;
        let json = serde_json::from_str::<JsonValue>(&row.get::<_, String>(1)?)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let inventory = serde_json::from_str::<Inventory>(&row.get::<_, String>(2)?)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let created_at = row.get::<_, u64>(3)?;
        let last_login_at = row.get::<_, u64>(4)?;
        let updated_at = row.get::<_, u64>(5)?;

        Ok(PlayerData::create_with_timestamps(
            username,
            json,
            inventory,
            created_at,
            last_login_at,
            updated_at,
        ))
    }
}

impl IServerStorage for SQLiteServerStorage {
    type Error = String;
    type PrimaryKey = i64;

    fn init(storage_settings: StorageSettings) -> Result<Self, String> {
        let mut db_path = storage_settings.get_data_path().to_path_buf();

        create_dir_all(&db_path).map_err(|e| e.to_string())?;
        db_path.push("data.db");

        let db = Connection::open(&db_path).map_err(|e| e.to_string())?;
        db.execute_batch(SQL_CREATE_SCHEMA).map_err(|e| e.to_string())?;

        let storage = Self {
            db_path,
            _storage_settings: storage_settings,
        };
        Ok(storage)
    }

    fn get_or_create_player_data(&self, username: impl Into<String>) -> Result<PlayerData, Self::Error> {
        let username = username.into();
        let db = self.open()?;

        let player_data = db
            .query_row(SQL_SELECT_PLAYER, (&username,), Self::read_player)
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some(player_data) = player_data {
            return Ok(player_data);
        }

        let player_data = PlayerData::create(username, JsonValue::Object(Default::default()), Inventory::default());
        let json = serde_json::to_string(player_data.get_json()).map_err(|e| e.to_string())?;
        let inventory = serde_json::to_string(player_data.get_inventory()).map_err(|e| e.to_string())?;

        db.execute(
            SQL_INSERT_PLAYER,
            (
                player_data.get_username(),
                json,
                inventory,
                player_data.get_created_at(),
                player_data.get_last_login_at(),
                player_data.get_updated_at(),
            ),
        )
        .map_err(|e| e.to_string())?;

        Ok(player_data)
    }

    fn save_player_data(&self, player_data: &PlayerData) -> Result<Self::PrimaryKey, Self::Error> {
        let db = self.open()?;
        let json = serde_json::to_string(player_data.get_json()).map_err(|e| e.to_string())?;
        let inventory = serde_json::to_string(player_data.get_inventory()).map_err(|e| e.to_string())?;
        let now = current_unix_timestamp();

        db.execute(
            SQL_UPSERT_PLAYER,
            (
                player_data.get_username(),
                json,
                inventory,
                player_data.get_created_at(),
                player_data.get_last_login_at(),
                now,
            ),
        )
        .map_err(|e| e.to_string())?;

        db.query_row(SQL_SELECT_PLAYER_ROWID, (player_data.get_username(),), |row| row.get(0))
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        inventory::item::Item,
        server_storage::{
            sqlite_storage::SQLiteServerStorage,
            taits::{IServerStorage, PlayerData},
        },
        utils::srotage_settings::StorageSettings,
    };
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn saves_and_loads_player_data_from_temp_storage() {
        let storage_settings = StorageSettings::in_memory();
        let storage = SQLiteServerStorage::init(storage_settings.clone()).unwrap();

        let mut player_data = storage.get_or_create_player_data("player").unwrap();
        *player_data.get_json_mut() = json!({
            "level": 7,
            "class": "mage"
        });
        player_data
            .get_inventory_mut()
            .set_slot(0, Item::create("stone", 32, BTreeMap::new()));

        let player_id = storage.save_player_data(&player_data).unwrap();
        assert!(player_id > 0);

        let storage = SQLiteServerStorage::init(storage_settings).unwrap();
        let saved_player_data = storage.get_or_create_player_data("player").unwrap();

        assert_eq!(saved_player_data.get_username(), "player");
        assert_eq!(saved_player_data.get_json()["level"], 7);
        assert_eq!(saved_player_data.get_json()["class"], "mage");
        assert_eq!(saved_player_data.get_inventory().slots_len(), 1);
        assert!(saved_player_data.get_created_at() > 0);
        assert!(saved_player_data.get_last_login_at() >= saved_player_data.get_created_at());
        assert!(saved_player_data.get_updated_at() >= saved_player_data.get_created_at());

        let saved_item = saved_player_data.get_inventory().get_slot(0).unwrap();
        assert_eq!(saved_item.slug, "stone");
        assert_eq!(saved_item.amount, 32);
    }

    #[test]
    fn save_player_data_creates_missing_player() {
        let storage = SQLiteServerStorage::init(StorageSettings::in_memory()).unwrap();
        let player_data = PlayerData::create("new_player", json!({ "level": 1 }), Default::default());

        storage.save_player_data(&player_data).unwrap();

        let saved_player_data = storage.get_or_create_player_data("new_player").unwrap();
        assert_eq!(saved_player_data.get_json()["level"], 1);
        assert!(saved_player_data.get_created_at() > 0);
        assert!(saved_player_data.get_updated_at() >= saved_player_data.get_created_at());
    }
}
