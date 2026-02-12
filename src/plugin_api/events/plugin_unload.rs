use serde::{Deserialize, Serialize};

use super::PluginEvent;

#[derive(Serialize, Deserialize)]
pub struct PluginUnloadEvent {}

impl PluginEvent for PluginUnloadEvent {
    const EXPORT_NAME: &'static str = "on_plugin_unload";
}
