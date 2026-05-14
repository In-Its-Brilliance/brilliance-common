use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct StorageSettings {
    data_path: PathBuf,
    _temp_dir: Option<std::sync::Arc<tempfile::TempDir>>,
}

impl StorageSettings {
    pub fn from_path(data_path: PathBuf) -> Self {
        Self {
            data_path,
            _temp_dir: None,
        }
    }

    pub fn in_memory() -> Self {
        let temp_dir = std::sync::Arc::new(tempfile::tempdir().expect("Failed to create temporary storage directory"));

        Self {
            data_path: temp_dir.path().to_path_buf(),
            _temp_dir: Some(temp_dir),
        }
    }

    pub fn get_data_path(&self) -> &Path {
        &self.data_path
    }

    pub fn is_in_memory(&self) -> bool {
        self._temp_dir.is_some()
    }
}
