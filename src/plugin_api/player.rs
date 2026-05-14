use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    client_id: u64,
}

#[cfg(feature = "wasm-plugin")]
#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn get_player_world_slug_raw(client_id: u64) -> String;
}

impl Player {
    pub fn create(client_id: u64) -> Self {
        Self { client_id }
    }

    pub fn get_client_id(&self) -> u64 {
        self.client_id
    }

    #[cfg(feature = "wasm-plugin")]
    pub fn get_world_slug(&self) -> Result<Option<String>, extism_pdk::Error> {
        let world_slug = unsafe { get_player_world_slug_raw(self.client_id)? };
        if world_slug.is_empty() {
            Ok(None)
        } else {
            Ok(Some(world_slug))
        }
    }
}
