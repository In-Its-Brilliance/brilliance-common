use std::collections::BTreeMap;
use strum_macros::Display;

use serde::{Deserialize, Serialize};

use crate::chunks::chunk_data::BlockIndexType;

#[derive(Display, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BodyPart {
    Chest,
    Hands,
    Pants,
    Boots,
    Head,
}

#[derive(Display, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WeaponKind {
    Sword,
}

/// For client item display in UI, and client UI only
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientItemKind {
    Block(BlockIndexType),

    // Non block or model item.
    // Used png resource texture as an icon.
    Icon(String),

    NotFound,
}

/// Client item info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientItem {
    item_kind: ClientItemKind,
    amount: u16,
    icon: Option<String>,
    title: Option<String>,
    description: Option<String>,
}

impl ClientItem {
    pub fn create(
        item_kind: ClientItemKind,
        amount: u16,
        icon: Option<String>,
        title: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            item_kind,
            amount,
            icon,
            title,
            description,
        }
    }

    pub fn get_item_kind(&self) -> &ClientItemKind {
        &self.item_kind
    }

    pub fn get_amount(&self) -> u16 {
        self.amount
    }

    pub fn amount(mut self, amount: u16) -> Self {
        self.amount = amount;
        self
    }

    pub fn get_icon(&self) -> Option<&String> {
        self.icon.as_ref()
    }

    pub fn get_title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    pub fn get_description(&self) -> Option<&String> {
        self.description.as_ref()
    }
}

/// For server item type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemKind {
    Block(BlockIndexType),

    // For server: type slug
    CustomItem(String),
}

impl From<String> for ItemKind {
    fn from(value: String) -> Self {
        ItemKind::CustomItem(value)
    }
}

impl From<&str> for ItemKind {
    fn from(value: &str) -> Self {
        ItemKind::CustomItem(value.to_string())
    }
}

impl From<BlockIndexType> for ItemKind {
    fn from(value: BlockIndexType) -> Self {
        ItemKind::Block(value)
    }
}

/// Server item data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    item_kind: ItemKind,
    amount: u16,
    modifiers: BTreeMap<String, Vec<u8>>,
}

impl Item {
    pub fn create(item_kind: impl Into<ItemKind>) -> Self {
        Self {
            item_kind: item_kind.into(),
            amount: 1,
            modifiers: Default::default(),
        }
    }

    pub fn amount(mut self, amount: u16) -> Self {
        self.amount = amount;
        self
    }

    pub fn modifiers(mut self, modifiers: BTreeMap<String, Vec<u8>>) -> Self {
        self.modifiers = modifiers;
        self
    }

    pub fn get_item_kind(&self) -> &ItemKind {
        &self.item_kind
    }

    pub fn get_amount(&self) -> u16 {
        self.amount
    }

    pub fn can_stack_with(&self, other: &Item) -> bool {
        self.item_kind == other.item_kind && self.modifiers == other.modifiers
    }
}
