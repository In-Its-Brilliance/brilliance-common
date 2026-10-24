use serde::{de::DeserializeOwned, Serialize};

pub trait Compressable: Serialize + Sized {
    fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec(&self).unwrap()
    }

    fn decode(encoded: Vec<u8>) -> Result<Self, String>
    where
        Self: DeserializeOwned,
    {
        postcard::from_bytes(&encoded).map_err(|e| format!("Decode error: {}", e))
    }

    #[cfg(feature = "zstd")]
    fn compress(&self) -> Vec<u8> {
        zstd::encode_all(&self.encode()[..], 7).unwrap()
    }

    #[cfg(feature = "zstd")]
    fn decompress(data: Vec<u8>) -> Result<Self, String>
    where
        Self: DeserializeOwned,
    {
        let raw = zstd::decode_all(&data[..]).map_err(|e| format!("Decompress error: {}", e))?;
        Self::decode(raw)
    }
}
