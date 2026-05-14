use serde::{Deserialize, Serialize};

use crate::plugin_api::player::Player;

use super::PluginEvent;

#[derive(Serialize, Deserialize)]
pub struct ClientScriptEvent {
    script_slug: String,
    slug: String,
    json: String,
    player: Player,
}

impl PluginEvent for ClientScriptEvent {
    const EXPORT_NAME: &'static str = "on_client_script_event";
}

impl ClientScriptEvent {
    pub fn create(
        script_slug: impl Into<String>,
        slug: impl Into<String>,
        json: impl Into<String>,
        client_id: u64,
    ) -> Self {
        Self {
            script_slug: script_slug.into(),
            slug: slug.into(),
            json: json.into(),
            player: Player::create(client_id),
        }
    }

    pub fn get_script_slug(&self) -> &String {
        &self.script_slug
    }

    pub fn get_slug(&self) -> &String {
        &self.slug
    }

    pub fn get_json(&self) -> &String {
        &self.json
    }

    pub fn get_player(&self) -> Player {
        self.player.clone()
    }
}
