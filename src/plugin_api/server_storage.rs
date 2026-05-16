use crate::{serde_json, server_storage::taits::PlayerData};

#[cfg(feature = "wasm-plugin")]
#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn get_or_create_player_data_raw(username: String) -> String;
    fn save_player_data_raw(player_data_json: String) -> String;
}

#[derive(Default)]
pub struct ServerStorage;

impl ServerStorage {
    pub fn get_or_create_player_data(&self, username: &str) -> Result<PlayerData, extism_pdk::Error> {
        let player_data_json = unsafe { get_or_create_player_data_raw(username.to_string())? };

        serde_json::from_str(&player_data_json)
            .map_err(|e| extism_pdk::Error::msg(format!("Failed to deserialize player data: {}", e)))
    }

    pub fn save_player_data(&self, player_data: &PlayerData) -> Result<i64, extism_pdk::Error> {
        let player_data_json = serde_json::to_string(player_data)
            .map_err(|e| extism_pdk::Error::msg(format!("Failed to serialize player data: {}", e)))?;
        let player_id = unsafe { save_player_data_raw(player_data_json)? };

        player_id
            .parse::<i64>()
            .map_err(|e| extism_pdk::Error::msg(format!("Invalid player id: {}", e)))
    }
}
