use crate::{blocks::block_info::BlockFace, SECTION_VOLUME, VERTICAL_SECTIONS};
use serde::{Deserialize, Serialize};

use super::block_position::{BlockPosition, ChunkBlockPosition};

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
pub struct ChunkData {
    data: Vec<Box<ChunkSectionData>>,
}

impl ChunkData {
    pub fn compress(&self) -> Vec<u8> {
        let raw = self.encode();
        zstd::encode_all(&raw[..], 7).unwrap()
    }

    pub fn decompress(data: Vec<u8>) -> Result<Self, String> {
        let raw = zstd::decode_all(&data[..]).map_err(|e| format!("Decompress error: {}", e))?;
        Self::decode(raw)
    }

    pub fn encode(&self) -> Vec<u8> {
        let encoded = bincode::serialize(&self).unwrap();
        encoded
    }

    pub fn decode(encoded: Vec<u8>) -> Result<Self, String> {
        let chunk_data: Self = match bincode::deserialize(&encoded) {
            Ok(d) => d,
            Err(e) => return Err(format!("Decode chunk error: &c{} ", e)),
        };
        Ok(chunk_data)
    }

    pub fn change_block(&mut self, section: u32, pos: &ChunkBlockPosition, block: Option<BlockDataInfo>) {
        if section > VERTICAL_SECTIONS as u32 {
            panic!("Tried to change block in section {section} more than max {VERTICAL_SECTIONS}");
        }

        self.data[section as usize].change(&pos, block);
    }

    pub fn get(&self, index: usize) -> Option<&Box<ChunkSectionData>> {
        self.data.get(index)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_block_info(&self, block_position: &BlockPosition) -> Option<BlockDataInfo> {
        let (section, chunk_block_position) = block_position.get_block_position();
        match self.data[section as usize].get(&chunk_block_position) {
            Some(b) => Some(b.clone()),
            None => None,
        }
    }

    pub fn push_section(&mut self, data: ChunkSectionData) {
        if self.data.len() >= VERTICAL_SECTIONS {
            panic!("Tried to insert sections more than max {VERTICAL_SECTIONS}");
        }
        self.data.push(Box::new(data));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        chunks::{chunk_data::ChunkData, chunk_position::ChunkPosition},
        world_generator::{
            default::{WorldGenerator, WorldGeneratorSettings},
            traits::IWorldGenerator,
        },
    };

    #[test]
    fn test_chunks_data() {
        let generator = WorldGenerator::create(Some(1), WorldGeneratorSettings::default()).unwrap();

        let chunk_position = ChunkPosition::new(0, 0);
        let chunk_data = generator.generate_chunk_data(&chunk_position);

        let encoded = chunk_data.encode();
        assert_eq!(encoded.len(), 110894);

        let encoded = chunk_data.compress();
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
        assert_eq!(
            chunk_data.get(0).unwrap().len(),
            decoded_chunk_data.get(0).unwrap().len()
        );
    }
}
