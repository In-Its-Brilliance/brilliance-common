use serde::{Deserialize, Serialize};

use crate::inventory::item::{BodyPart, WeaponKind};

#[cfg(feature = "wasm-plugin")]
#[extism_pdk::host_fn]
extern "ExtismHost" {
    fn add_item_raw(item_json: String) -> String;
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ItemDisplay {
    #[serde(rename = "icon")]
    Icon(String),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemInfo {
    slug: String,
    item_type: ItemType,
    item_display: ItemDisplay,
    title: String,
    description: String,
    #[serde(default = "ItemInfo::default_max_stack_size")]
    max_stack_size: u16,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ItemType {
    #[serde(rename = "armor")]
    Armor {
        body_part: BodyPart,
        model: String,
    },
    #[serde(rename = "weapon")]
    Weapon {
        weapon_kind: WeaponKind,
        model: String,
    },
    #[serde(rename = "other")]
    Other,
}

pub struct ItemsManager;

impl ItemType {
    pub fn armor(body_part: BodyPart, model: impl Into<String>) -> Self {
        Self::Armor {
            body_part,
            model: model.into(),
        }
    }

    pub fn weapon(weapon_kind: WeaponKind, model: impl Into<String>) -> Self {
        Self::Weapon {
            weapon_kind,
            model: model.into(),
        }
    }

    pub fn other() -> Self {
        Self::Other
    }
}

impl ItemInfo {
    fn default_max_stack_size() -> u16 {
        1
    }

    pub fn create(
        slug: impl Into<String>,
        item_type: ItemType,
        item_display: ItemDisplay,
        title: impl Into<String>,
        description: impl Into<String>,
        max_stack_size: u16,
    ) -> Self {
        assert!(max_stack_size >= 1, "max_stack_size must be at least 1");
        Self {
            slug: slug.into(),
            item_type,
            item_display,
            title: title.into(),
            description: description.into(),
            max_stack_size,
        }
    }

    pub fn get_slug(&self) -> &String {
        &self.slug
    }

    pub fn get_item_type(&self) -> &ItemType {
        &self.item_type
    }

    pub fn get_item_display(&self) -> &ItemDisplay {
        &self.item_display
    }

    pub fn get_title(&self) -> &String {
        &self.title
    }

    pub fn get_description(&self) -> &String {
        &self.description
    }

    pub fn get_max_stack_size(&self) -> u16 {
        self.max_stack_size
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
