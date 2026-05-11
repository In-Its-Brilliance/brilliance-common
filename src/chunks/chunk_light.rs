use serde::{Deserialize, Serialize};

use super::block_position::ChunkBlockPosition;
use crate::SECTION_VOLUME;

/// Per-section light data stored as RGB 4-4-4 packed into u16.
///
/// Bit layout: `0000_BBBB_GGGG_RRRR`
///   - bits  0..3  — Red   (0–15)
///   - bits  4..7  — Green (0–15)
///   - bits  8..11 — Blue  (0–15)
///   - bits 12..15 — reserved
#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkLightData {
    data: Vec<u16>,
}

impl std::fmt::Debug for ChunkLightData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let non_zero = self.data.iter().filter(|&&v| v != 0).count();
        write!(f, "ChunkLightData({} lit)", non_zero)
    }
}

impl Default for ChunkLightData {
    fn default() -> Self {
        Self {
            data: vec![0u16; SECTION_VOLUME],
        }
    }
}

impl ChunkLightData {
    /// Pack RGB (each 0–15) into a single u16.
    #[inline]
    pub fn pack(r: u8, g: u8, b: u8) -> u16 {
        (r as u16 & 0xF) | ((g as u16 & 0xF) << 4) | ((b as u16 & 0xF) << 8)
    }

    /// Extract red channel (0–15).
    #[inline]
    pub fn get_r(value: u16) -> u8 {
        (value & 0xF) as u8
    }

    /// Extract green channel (0–15).
    #[inline]
    pub fn get_g(value: u16) -> u8 {
        ((value >> 4) & 0xF) as u8
    }

    /// Extract blue channel (0–15).
    #[inline]
    pub fn get_b(value: u16) -> u8 {
        ((value >> 8) & 0xF) as u8
    }

    /// Unpack u16 into (r, g, b) tuple.
    #[inline]
    pub fn unpack(value: u16) -> (u8, u8, u8) {
        (Self::get_r(value), Self::get_g(value), Self::get_b(value))
    }
}

impl ChunkLightData {
    /// Set light at position (each channel 0–15).
    pub fn set(&mut self, pos: &ChunkBlockPosition, r: u8, g: u8, b: u8) {
        let idx = pos.linearize() as usize;
        self.data[idx] = Self::pack(r, g, b);
    }

    /// Set a raw packed value at position.
    pub fn set_raw(&mut self, pos: &ChunkBlockPosition, value: u16) {
        let idx = pos.linearize() as usize;
        self.data[idx] = value;
    }

    /// Get raw packed value at position.
    pub fn get(&self, pos: &ChunkBlockPosition) -> u16 {
        let idx = pos.linearize() as usize;
        self.data[idx]
    }

    /// Get unpacked (r, g, b) at position.
    pub fn get_rgb(&self, pos: &ChunkBlockPosition) -> (u8, u8, u8) {
        Self::unpack(self.get(pos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunks::block_position::ChunkBlockPosition;

    #[test]
    fn test_pack_unpack() {
        assert_eq!(ChunkLightData::pack(15, 0, 0), 0x00F);
        assert_eq!(ChunkLightData::pack(0, 15, 0), 0x0F0);
        assert_eq!(ChunkLightData::pack(0, 0, 15), 0xF00);
        assert_eq!(ChunkLightData::pack(15, 15, 15), 0xFFF);

        let packed = ChunkLightData::pack(5, 10, 3);
        assert_eq!(ChunkLightData::unpack(packed), (5, 10, 3));
    }

    #[test]
    fn test_set_get() {
        let mut light = ChunkLightData::default();
        let pos = ChunkBlockPosition::new(7, 3, 12);

        light.set(&pos, 12, 8, 4);
        assert_eq!(light.get_rgb(&pos), (12, 8, 4));

        let raw = light.get(&pos);
        assert_eq!(ChunkLightData::get_r(raw), 12);
        assert_eq!(ChunkLightData::get_g(raw), 8);
        assert_eq!(ChunkLightData::get_b(raw), 4);
    }

    #[test]
    fn test_default_is_zero() {
        let light = ChunkLightData::default();
        let pos = ChunkBlockPosition::new(0, 0, 0);
        assert_eq!(light.get(&pos), 0);
        assert_eq!(light.get_rgb(&pos), (0, 0, 0));
    }
}
