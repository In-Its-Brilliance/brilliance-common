use serde::{Deserialize, Serialize};

use super::PluginEvent;

#[derive(Serialize, Deserialize)]
pub struct PluginLoadEvent {
    pub plugin_slug: String,
}

impl PluginEvent for PluginLoadEvent {
    const EXPORT_NAME: &'static str = "on_plugin_load";
}
