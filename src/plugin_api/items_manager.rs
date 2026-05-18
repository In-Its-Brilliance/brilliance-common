use serde::{Deserialize, Serialize};

use crate::inventory::item::{BodyPart, WeaponKind};

#[cfg(feature = "wasm-plugin")]
#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn add_item_raw(item_json: String) -> String;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemInfo {
    slug: String,
    item_type: ItemType,
    title: String,
    description: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ItemType {
    #[serde(rename = "armor")]
    Armor {
        body_part: BodyPart,
        icon: String,
        model: String,
    },
    #[serde(rename = "weapon")]
    Weapon {
        weapon_kind: WeaponKind,
        icon: String,
        model: String,
    },
}

pub struct ItemsManager;

impl ItemType {
    pub fn armor(body_part: BodyPart, icon: impl Into<String>, model: impl Into<String>) -> Self {
        Self::Armor {
            body_part,
            icon: icon.into(),
            model: model.into(),
        }
    }

    pub fn weapon(weapon_kind: WeaponKind, icon: impl Into<String>, model: impl Into<String>) -> Self {
        Self::Weapon {
            weapon_kind,
            icon: icon.into(),
            model: model.into(),
        }
    }
}

impl ItemInfo {
    pub fn create(
        slug: impl Into<String>,
        item_type: ItemType,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            slug: slug.into(),
            item_type,
            title: title.into(),
            description: description.into(),
        }
    }

    pub fn get_slug(&self) -> &String {
        &self.slug
    }

    pub fn get_item_type(&self) -> &ItemType {
        &self.item_type
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_description(&self) -> &String {
        &self.description
    }
}

impl ItemsManager {
    pub fn singleton() -> &'static Self {
        static INSTANCE: ItemsManager = ItemsManager;
        &INSTANCE
    }

    #[cfg(feature = "wasm-plugin")]
    pub fn add_item(&self, item: ItemInfo) -> Result<(), extism_pdk::Error> {
        let item_json = serde_json::to_string(&item)
            .map_err(|e| extism_pdk::Error::msg(format!("Failed to serialize item info: {}", e)))?;
        unsafe {
            add_item_raw(item_json)?;
        }
        Ok(())
    }
}
