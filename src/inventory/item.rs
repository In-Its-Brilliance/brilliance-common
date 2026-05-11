use std::collections::BTreeMap;

pub struct Item {
    pub slug: String,
    pub amount: u16,
    pub modifiers: BTreeMap<String, Vec<u8>>,
}
