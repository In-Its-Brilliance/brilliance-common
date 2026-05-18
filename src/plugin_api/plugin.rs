#[cfg(feature = "wasm-plugin")]
#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn read_dir_raw(path: String) -> String;
    fn read_file_raw(path: String) -> String;
}

pub struct Plugin;

impl Plugin {
    pub fn singleton() -> &'static Self {
        static INSTANCE: Plugin = Plugin;
        &INSTANCE
    }

    #[cfg(feature = "wasm-plugin")]
    pub fn read_dir(&self, path: impl Into<String>) -> Vec<String> {
        let entries_json = unsafe { read_dir_raw(path.into()) }
            .unwrap_or_else(|e| panic!("Failed to read plugin directory: {}", e));
        serde_json::from_str(&entries_json)
            .unwrap_or_else(|e| panic!("Invalid plugin directory list json: {}", e))
    }

    #[cfg(feature = "wasm-plugin")]
    pub fn read_file(&self, path: impl Into<String>) -> String {
        unsafe { read_file_raw(path.into()) }
            .unwrap_or_else(|e| panic!("Failed to read plugin file: {}", e))
    }

    #[cfg(feature = "wasm-plugin")]
    pub fn read_yaml<T>(&self, path: impl Into<String>) -> Result<T, extism_pdk::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let yaml_text = self.read_file(path);
        serde_yaml::from_str(&yaml_text)
            .map_err(|e| extism_pdk::Error::msg(format!("Invalid YAML: {}", e)))
    }
}
