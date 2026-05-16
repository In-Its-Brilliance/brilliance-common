use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::chunks::chunk_data::BlockIndexType;
use crate::default_blocks_ids::BlockID;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemKind {
    Block(BlockIndexType),
    Icon(String),
}

impl From<BlockID> for ItemKind {
    fn from(value: BlockID) -> Self {
        ItemKind::Block(value.id())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientItem {
    pub slug: String,
    pub amount: u16,
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub item_kind: ItemKind,
    pub amount: u16,
    pub modifiers: BTreeMap<String, Vec<u8>>,
}

impl Item {
    pub fn create(item_kind: impl Into<ItemKind>) -> Self {
        Self {
            item_kind: item_kind.into(),
            amount: 0,
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
}

impl From<&Item> for ClientItem {
    fn from(value: &Item) -> Self {
        let slug = match &value.item_kind {
            ItemKind::Block(block_id) => block_id_to_slug(*block_id).to_string(),
            ItemKind::Icon(_) => unimplemented!("item icons are not implemented yet"),
        };

        Self { slug, amount: value.amount, title: None, description: None }
    }
}

impl From<Item> for ClientItem {
    fn from(value: Item) -> Self {
        ClientItem::from(&value)
    }
}

fn block_id_to_slug(block_id: BlockIndexType) -> &'static str {
    match BlockID::from_id(&block_id) {
        Some(BlockID::Grass) => "grass",
        Some(BlockID::Stone) => "stone",
        Some(BlockID::SmoothStone) => "smooth_stone",
        Some(BlockID::StoneBricks) => "stone_bricks",
        Some(BlockID::CrackedStoneBricks) => "cracked_stone_bricks",
        Some(BlockID::MossyStoneBricks) => "mossy_stone_bricks",
        Some(BlockID::Gravel) => "gravel",
        Some(BlockID::CoarseDirt) => "coarse_dirt",
        Some(BlockID::Bedrock) => "bedrock",
        Some(BlockID::Sand) => "sand",
        Some(BlockID::AmethystBlock) => "amethyst_block",
        Some(BlockID::OakPlanks) => "oak_planks",
        Some(BlockID::IronBlock) => "iron_block",
        Some(BlockID::Sandstone) => "sandstone",
        Some(BlockID::ChiseledSandstone) => "chiseled_sandstone",
        Some(BlockID::Podzol) => "podzol",
        Some(BlockID::Blackstone) => "blackstone",
        Some(BlockID::PolishedBlackstone) => "polished_blackstone",
        Some(BlockID::Andesite) => "andesite",
        Some(BlockID::Deepslate) => "deepslate",
        Some(BlockID::DeepslateBricks) => "deepslate_bricks",
        Some(BlockID::CrackedDeepslateBricks) => "cracked_deepslate_bricks",
        Some(BlockID::PolishedDeepslate) => "polished_deepslate",
        Some(BlockID::Diorite) => "diorite",
        Some(BlockID::PolishedDiorite) => "polished_diorite",
        Some(BlockID::Granite) => "granite",
        Some(BlockID::PolishedGranite) => "polished_granite",
        Some(BlockID::Cobblestone) => "cobblestone",
        Some(BlockID::MossyCobblestone) => "mossy_cobblestone",
        Some(BlockID::AcaciaLog) => "acacia_log",
        Some(BlockID::AcaciaLeaves) => "acacia_leaves",
        Some(BlockID::AcaciaPlanks) => "acacia_planks",
        Some(BlockID::BirchLog) => "birch_log",
        Some(BlockID::BirchLeaves) => "birch_leaves",
        Some(BlockID::BirchPlanks) => "birch_planks",
        Some(BlockID::DarkOak) => "dark_oak",
        Some(BlockID::DarkOakLeaves) => "dark_oak_leaves",
        Some(BlockID::DarkOakPlanks) => "dark_oak_planks",
        Some(BlockID::JungleLog) => "jungle_log",
        Some(BlockID::JungleLeaves) => "jungle_leaves",
        Some(BlockID::JunglePlanks) => "jungle_planks",
        Some(BlockID::OakLog) => "oak_log",
        Some(BlockID::OakLeaves) => "oak_leaves",
        Some(BlockID::SpruceLog) => "spruce_log",
        Some(BlockID::SpruceLeaves) => "spruce_leaves",
        Some(BlockID::SprucePlanks) => "spruce_planks",
        Some(BlockID::BushSmall) => "bush_small",
        Some(BlockID::FlowerLupin) => "flower_lupin",
        Some(BlockID::FlowerLupin2) => "flower_lupin2",
        Some(BlockID::FlowerOrchid) => "flower_orchid",
        Some(BlockID::FlowerRose) => "flower_rose",
        Some(BlockID::FlowerWhite) => "flower_white",
        Some(BlockID::FlowerWhite2) => "flower_white2",
        Some(BlockID::FlowerYellow) => "flower_yellow",
        Some(BlockID::FlowerYellow2) => "flower_yellow2",
        Some(BlockID::FlowerYellow3) => "flower_yellow3",
        Some(BlockID::Grass1) => "grass1",
        Some(BlockID::Grass2) => "grass2",
        Some(BlockID::Grass3) => "grass3",
        Some(BlockID::Grass4) => "grass4",
        Some(BlockID::GroundMoss1) => "ground_moss1",
        Some(BlockID::GroundMoss2) => "ground_moss2",
        Some(BlockID::GroundMoss3) => "ground_moss3",
        Some(BlockID::TallGrass1) => "tall_grass1",
        Some(BlockID::TallGrass2) => "tall_grass2",
        Some(BlockID::Water) => "water",
        None => unreachable!("invalid block id"),
    }
}
