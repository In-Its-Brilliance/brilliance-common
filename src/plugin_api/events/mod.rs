pub mod plugin_load;
pub mod plugin_unload;
pub mod generage_chunk;
pub mod generage_world_macro;

pub trait PluginEvent: Sized {
    const EXPORT_NAME: &'static str;
}
