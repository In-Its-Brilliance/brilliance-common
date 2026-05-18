use serde::{Deserialize, Serialize};

use crate::plugin_api::player::Player;

use super::PluginEvent;

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerSpawnEvent {
    player: Player,
}

impl PluginEvent for PlayerSpawnEvent {
    const EXPORT_NAME: &'static str = "on_player_spawn";
}

impl PlayerSpawnEvent {
    pub fn create(client_id: u64) -> Self {
        Self {
            player: Player::create(client_id),
        }
    }

    pub fn get_player(&self) -> Player {
        self.player.clone()
    }
}
