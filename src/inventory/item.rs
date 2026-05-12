use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub slug: String,
    pub amount: u16,
    pub modifiers: BTreeMap<String, Vec<u8>>,
}

impl Item {
    pub fn create(slug: impl Into<String>, amount: u16, modifiers: BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            slug: slug.into(),
            amount,
            modifiers,
        }
    }
}
