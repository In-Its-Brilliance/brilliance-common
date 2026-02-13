use super::PluginEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GenerateWorldMacroEvent {
    seed: u64,
    method: String,
    settings: Option<serde_yaml::Value>,
}

impl PluginEvent for GenerateWorldMacroEvent {
    const EXPORT_NAME: &'static str = "on_generate_world_macro";
}

impl GenerateWorldMacroEvent {
    pub fn create(seed: u64, method: impl Into<String>, settings: Option<serde_yaml::Value>) -> Self {
        Self { seed, method: method.into(), settings }
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn get_method(&self) -> &String {
        &self.method
    }

    pub fn get_settings(&self) -> &Option<serde_yaml::Value> {
        &self.settings
    }
}
