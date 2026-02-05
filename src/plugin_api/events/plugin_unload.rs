use serde::{Deserialize, Serialize};

use super::PluginEvent;

#[derive(Serialize, Deserialize)]
pub struct PluginUnloadEvent {
    pub plugin_slug: String,
}

impl PluginEvent for PluginUnloadEvent {
    const EXPORT_NAME: &'static str = "on_plugin_unload";
}
