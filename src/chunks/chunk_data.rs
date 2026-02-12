use crate::{blocks::block_info::BlockFace, utils::compressable::Compressable, SECTION_VOLUME};
use serde::{Deserialize, Serialize};

use super::block_position::ChunkBlockPosition;

pub type BlockIndexType = u16;

// Contains block id and rotation
#[derive(Clone, Eq, Copy, Serialize, Deserialize)]
pub struct BlockDataInfo {
    id: BlockIndexType,
    face: Option<BlockFace>,
}

impl std::fmt::Debug for BlockDataInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let face = match self.face {
            Some(f) => format!(" face:{:?}", f),
            None => "".to_string(),
        };
        write!(f, "b#{}{}", self.id, face)
    }
}

impl PartialEq for BlockDataInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.face == other.face
    }
}

impl BlockDataInfo {
    pub fn create(id: BlockIndexType, face: Option<BlockFace>) -> Self {
        Self { id, face }
    }

    pub fn get_id(&self) -> BlockIndexType {
        self.id
    }

    pub fn get_face(&self) -> Option<&BlockFace> {
        self.face.as_ref()
    }

    pub fn set_face(&mut self, face: Option<BlockFace>) {
        self.face = face;
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
    data: serde_json::Value,
}

impl Compressable for WorldMacroData {}

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
                section, self.data.len()
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
        sections.change_block(
            0,
            &ChunkBlockPosition::new(0, 0, 0),
            Some(BlockDataInfo::create(0, None)),
        );
        sections.change_block(
            0,
            &ChunkBlockPosition::new(1, 1, 1),
            Some(BlockDataInfo::create(0, None)),
        );

        let encoded = sections.encode();
        assert_eq!(encoded.len(), 4118);

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
