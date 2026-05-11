use std::path::PathBuf;

#[derive(Clone)]
pub struct StorageSettings {
    data_path: PathBuf,
}

impl StorageSettings {
    pub fn from_path(data_path: PathBuf) -> Self {
        Self { data_path }
    }

    pub fn in_memory() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let settings = Self {
            data_path: tmp.path().to_path_buf(),
        };
        settings
    }

    pub fn get_data_path(&self) -> &PathBuf {
        &self.data_path
    }
}
