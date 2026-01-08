use crate::BlockId;
use crate::CHUNK_LENGTH;
use crate::Chunk;
use crate::deserialize_buffer;
use crate::serialize_buffer;
use bevy::math::Vec3;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

impl Serialize for Chunk {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data_as_u8: Vec<u8> = self
            .data
            .iter()
            .map(|block_id| {
                let block_byte: u8 = (*block_id).into();
                block_byte
            })
            .collect();
        let serialized_data = serialize_buffer(data_as_u8);
        let mut state = serializer.serialize_struct("Chunk", 2)?;
        state.serialize_field("data", &serialized_data)?;
        state.serialize_field("position", &self.position)?;
        state.end()
    }
}

struct BytesVec(Vec<u8>);

impl<'de> Deserialize<'de> for BytesVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        Ok(BytesVec(vec))
    }
}

impl<'de> Deserialize<'de> for Chunk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ChunkData {
            data: BytesVec,
            position: Vec3,
        }

        let ChunkData { data, position } = ChunkData::deserialize(deserializer)?;
        let chunk_data_bytes_u8: Vec<u8> = data.0;
        let bytes_slice: &[u8] = &chunk_data_bytes_u8;
        let deserialized_data = deserialize_buffer(bytes_slice);
        let data_as_block_id: [BlockId; CHUNK_LENGTH] = deserialized_data
            .into_iter()
            .map(BlockId::from)
            .collect::<Vec<BlockId>>()
            .try_into()
            .map_err(|_| serde::de::Error::custom("Failed to convert data to BlockId array"))?;

        Ok(Chunk::with_data(data_as_block_id, position))
    }
}
