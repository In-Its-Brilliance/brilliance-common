use super::PluginEvent;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PluginLoadEvent {}

impl PluginEvent for PluginLoadEvent {
    const EXPORT_NAME: &'static str = "on_plugin_load";
}

#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn register_world_generator_raw(name: String) -> ();
    fn get_plugin_slug_raw() -> String;
}

impl PluginLoadEvent {
    pub fn register_world_generator(&self, name: &str) -> Result<(), extism_pdk::Error> {
        unsafe { register_world_generator_raw(name.to_string()) }
    }

    pub fn get_slug(&self) -> Result<String, extism_pdk::Error> {
        unsafe { get_plugin_slug_raw() }
    }
}
