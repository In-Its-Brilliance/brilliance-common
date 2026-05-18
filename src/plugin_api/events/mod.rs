#[cfg(feature = "wasm-plugin")]
pub mod client_script_event;
pub mod generage_chunk;
pub mod generage_world_macro;
#[cfg(feature = "wasm-plugin")]
pub mod plugin_load;
#[cfg(feature = "wasm-plugin")]
pub mod player_spawn;
pub mod plugin_unload;

pub trait PluginEvent: Sized {
    const EXPORT_NAME: &'static str;
}
