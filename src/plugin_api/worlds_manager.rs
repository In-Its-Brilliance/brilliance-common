
#[derive(Default)]
pub struct WorldsManager;

#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn has_world_raw(slug: String) -> String;
    fn create_world_raw(slug: String) -> ();
}

impl WorldsManager {
    pub fn has_world(&self, slug: &str) -> Result<bool, extism_pdk::Error> {
        let result = unsafe { has_world_raw(slug.to_string())? };
        Ok(result == "true")
    }

    pub fn create_world(&self, slug: &str) -> Result<(), extism_pdk::Error> {
        unsafe { create_world_raw(slug.to_string()) }
    }
}
