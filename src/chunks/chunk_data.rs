use crate::{blocks::block_info::BlockFace, utils::compressable::Compressable, SECTION_VOLUME};
use serde::{Deserialize, Serialize};

use super::block_position::ChunkBlockPosition;

pub type BlockIndexType = u16;
pub type BlockColorType = u8;

// Contains block id and rotation, color
#[derive(Clone, Eq, Copy, Serialize, Deserialize)]
pub struct BlockDataInfo {
    id: BlockIndexType,
    face: Option<BlockFace>,
    color: Option<BlockColorType>,
}

impl std::fmt::Debug for BlockDataInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let face = match self.face {
            Some(f) => format!(".face:{:?}", f),
            None => "".to_string(),
        };
        let color = match self.face {
            Some(f) => format!(".color:{:?}", f),
            None => "".to_string(),
        };
        write!(f, "b#{}{}{}", self.id, face, color)
    }
}

impl PartialEq for BlockDataInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.face == other.face
    }
}

impl BlockDataInfo {
    pub fn create(id: BlockIndexType) -> Self {
        Self {
            id,
            face: None,
            color: None,
        }
    }

    pub fn face(mut self, face: BlockFace) -> Self {
        self.face = Some(face);
        self
    }

    pub fn color(mut self, color: BlockColorType) -> Self {
        self.color = Some(color);
        self
    }

    pub fn get_id(&self) -> BlockIndexType {
        self.id
    }

    pub fn get_face(&self) -> &Option<BlockFace> {
        &self.face
    }

    pub fn set_face(&mut self, face: Option<BlockFace>) {
        self.face = face;
    }

    pub fn get_color(&self) -> &Option<BlockColorType> {
        &self.color
    }
}

// Contains all chunk block data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChunkSectionData {
    data: Vec<Option<BlockDataInfo>>,
}

impl Default for ChunkSectionData {
    fn default() -> Self {
        Self {
            data: vec![None; SECTION_VOLUME],
        }
    }
}

impl ChunkSectionData {
    pub fn change(&mut self, pos: &ChunkBlockPosition, block: Option<BlockDataInfo>) {
        let idx = pos.linearize() as usize;
        self.data[idx] = block;
    }

    pub fn insert(&mut self, pos: &ChunkBlockPosition, block: BlockDataInfo) -> Option<BlockDataInfo> {
        let idx = pos.linearize() as usize;
        let old = self.data[idx];
        self.data[idx] = Some(block);
        old
    }

    pub fn get(&self, pos: &ChunkBlockPosition) -> Option<&BlockDataInfo> {
        let idx = pos.linearize() as usize;
        self.data[idx].as_ref()
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &BlockDataInfo)> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|b| (i, b)))
    }

    pub fn len(&self) -> usize {
        self.data.iter().filter(|v| v.is_some()).count()
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct WorldMacroData {
    data: serde_yaml::Value,
}

impl WorldMacroData {
    pub fn create(data: serde_yaml::Value) -> Self {
        Self { data }
    }

    pub fn get_data(&self) -> &serde_yaml::Value {
        &self.data
    }
}

impl Compressable for WorldMacroData {
    fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    fn decode(encoded: Vec<u8>) -> Result<Self, String> {
        serde_json::from_slice(&encoded).map_err(|e| format!("Decode error: {}", e))
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ChunkData {
    data: Vec<Box<ChunkSectionData>>,
}

impl Compressable for ChunkData {}

impl ChunkData {
    pub fn change_block(&mut self, section: u32, pos: &ChunkBlockPosition, block: Option<BlockDataInfo>) {
        if section > crate::VERTICAL_SECTIONS as u32 {
            panic!(
                "Tried to change block in section {section} more than max {}",
                crate::VERTICAL_SECTIONS
            );
        }
        if section >= self.data.len() as u32 {
            panic!(
                "Tried to change block in section {} but only {} sections exist",
                section,
                self.data.len()
            );
        }
        self.data[section as usize].change(&pos, block);
    }

    pub fn get(&self, index: usize) -> Option<&Box<ChunkSectionData>> {
        self.data.get(index)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_block_info(&self, block_position: &super::block_position::BlockPosition) -> Option<BlockDataInfo> {
        let (section, chunk_block_position) = block_position.get_block_position();
        match self.data[section as usize].get(&chunk_block_position) {
            Some(b) => Some(b.clone()),
            None => None,
        }
    }

    pub fn push_section(&mut self, data: ChunkSectionData) {
        if self.data.len() >= crate::VERTICAL_SECTIONS {
            panic!("Tried to insert sections more than max {}", crate::VERTICAL_SECTIONS);
        }
        self.data.push(Box::new(data));
    }
}

#[cfg(feature = "full")]
#[cfg(test)]
mod tests {
    use crate::{
        chunks::{
            block_position::ChunkBlockPosition,
            chunk_data::{BlockDataInfo, ChunkData, ChunkSectionData},
        },
        utils::compressable::Compressable,
    };

    #[test]
    fn test_chunks_data() {
        let mut sections = ChunkData::default();
        sections.push_section(ChunkSectionData::default());
        sections.change_block(0, &ChunkBlockPosition::new(0, 0, 0), Some(BlockDataInfo::create(0)));
        sections.change_block(0, &ChunkBlockPosition::new(1, 1, 1), Some(BlockDataInfo::create(0)));

        let encoded = sections.encode();
        assert!(encoded.len() < 5000);

        let encoded = sections.compress();
        let target_max = 200;
        assert!(
            encoded.len() <= target_max,
            "{}",
            format!(
                "compressed result len: {} more than target max {}",
                encoded.len(),
                target_max
            )
        );

        let decoded_chunk_data = ChunkData::decompress(encoded).unwrap();
        assert_eq!(sections.get(0).unwrap().len(), decoded_chunk_data.get(0).unwrap().len());
    }
}
