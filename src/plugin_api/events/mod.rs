pub mod plugin_load;
pub mod plugin_unload;

pub trait PluginEvent: Sized {
    const EXPORT_NAME: &'static str;
}
