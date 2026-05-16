use super::taits::{IWorldStorage, WorldStorageData};

use crate::{
    chunks::{chunk_data::BlockIndexType, chunk_position::ChunkPosition, chunk_storage::ChunkStorage},
    utils::{compressable::Compressable, srotage_settings::StorageSettings},
};

use redb::{Database, ReadableDatabase, TableDefinition};

use std::{
    collections::BTreeMap,
    fs::{create_dir_all, read_dir, remove_file},
    path::PathBuf,
};

const TABLE_CHUNKS: TableDefinition<&str, &[u8]> = TableDefinition::new("chunks");
const TABLE_META: TableDefinition<&str, &[u8]> = TableDefinition::new("meta");

const META_SLUG_KEY: &str = "slug";
const META_SEED_KEY: &str = "seed";
const META_WORLD_GENERATOR_KEY: &str = "world_generator";
const META_WORLD_MACRO_DATA_KEY: &str = "world_macro_data";

pub struct RedbWorldStorage {
    db_path: PathBuf,
    db: Database,
    _storage_settings: StorageSettings,
}

impl RedbWorldStorage {
    fn chunk_key(chunk_position: &ChunkPosition) -> String {
        format!("{}:{}", chunk_position.x, chunk_position.z)
    }
}

impl IWorldStorage for RedbWorldStorage {
    type Error = String;
    type PrimaryKey = String;

    fn init(storage_settings: StorageSettings, slug: impl Into<String>) -> Result<Self, Self::Error> {
        let mut db_path = storage_settings.get_data_path().to_path_buf();

        db_path.push("worlds");
        create_dir_all(&db_path).map_err(|e| e.to_string())?;

        db_path.push(format!("{}.redb", slug.into()));

        let db = Database::create(&db_path).map_err(|e| e.to_string())?;

        {
            let write_txn = db.begin_write().map_err(|e| e.to_string())?;

            write_txn.open_table(TABLE_CHUNKS).map_err(|e| e.to_string())?;
            write_txn.open_table(TABLE_META).map_err(|e| e.to_string())?;

            write_txn.commit().map_err(|e| e.to_string())?;
        }

        Ok(Self {
            db_path,
            db,
            _storage_settings: storage_settings,
        })
    }

    fn create_new(&self, world_info: &WorldStorageData) -> Result<(), String> {
        let write_txn = self.db.begin_write().map_err(|e| e.to_string())?;

        {
            let mut table = write_txn.open_table(TABLE_META).map_err(|e| e.to_string())?;

            table
                .insert(META_SLUG_KEY, world_info.get_slug().as_bytes())
                .map_err(|e| e.to_string())?;

            let seed_bytes = world_info.get_seed().to_le_bytes();
            table
                .insert(META_SEED_KEY, seed_bytes.as_slice())
                .map_err(|e| e.to_string())?;

            table
                .insert(META_WORLD_GENERATOR_KEY, world_info.get_world_generator().as_bytes())
                .map_err(|e| e.to_string())?;

            let world_macro_data = serde_json::to_vec(world_info.get_world_macro_data()).map_err(|e| e.to_string())?;

            table
                .insert(META_WORLD_MACRO_DATA_KEY, world_macro_data.as_slice())
                .map_err(|e| e.to_string())?;
        }

        write_txn.commit().map_err(|e| e.to_string())?;

        Ok(())
    }

    fn has_chunk_data(&self, chunk_position: &ChunkPosition) -> Result<Option<Self::PrimaryKey>, String> {
        let read_txn = self.db.begin_read().map_err(|e| e.to_string())?;
        let table = read_txn.open_table(TABLE_CHUNKS).map_err(|e| e.to_string())?;

        let key = Self::chunk_key(chunk_position);
        let exists = table.get(key.as_str()).map_err(|e| e.to_string())?;

        Ok(exists.map(|_| key))
    }

    fn read_chunk_data(&self, chunk_id: Self::PrimaryKey) -> Result<ChunkStorage, String> {
        let read_txn = self.db.begin_read().map_err(|e| e.to_string())?;
        let table = read_txn.open_table(TABLE_CHUNKS).map_err(|e| e.to_string())?;

        let value = table
            .get(chunk_id.as_str())
            .map_err(|e| e.to_string())?
            .ok_or("Chunk not found")?;

        ChunkStorage::decompress(value.value().to_vec())
    }

    fn save_chunk_data(&self, chunk_position: &ChunkPosition, data: &ChunkStorage) -> Result<Self::PrimaryKey, String> {
        let write_txn = self.db.begin_write().map_err(|e| e.to_string())?;
        let key = Self::chunk_key(chunk_position);

        {
            let mut table = write_txn.open_table(TABLE_CHUNKS).map_err(|e| e.to_string())?;
            let encoded = data.compress();

            table
                .insert(key.as_str(), encoded.as_slice())
                .map_err(|e| e.to_string())?;
        }

        write_txn.commit().map_err(|e| e.to_string())?;

        Ok(key)
    }

    fn delete(&self) -> Result<(), String> {
        remove_file(self.db_path.clone()).map_err(|e| e.to_string())
    }

    fn scan_worlds(storage_settings: StorageSettings) -> Result<Vec<WorldStorageData>, String> {
        let mut worlds = Vec::new();

        let mut worlds_path = storage_settings.get_data_path().to_path_buf();
        worlds_path.push("worlds");

        create_dir_all(&worlds_path).map_err(|e| e.to_string())?;

        let paths = read_dir(worlds_path).map_err(|e| e.to_string())?;

        for path in paths {
            let path = path.map_err(|e| e.to_string())?.path();

            if !path.to_string_lossy().ends_with(".redb") {
                continue;
            }

            let db = Database::create(&path).map_err(|e| e.to_string())?;
            let read_txn = db.begin_read().map_err(|e| e.to_string())?;
            let table = read_txn.open_table(TABLE_META).map_err(|e| e.to_string())?;

            let slug = String::from_utf8(
                table
                    .get(META_SLUG_KEY)
                    .map_err(|e| e.to_string())?
                    .ok_or("Missing slug")?
                    .value()
                    .to_vec(),
            )
            .map_err(|e| e.to_string())?;

            let seed_value = table
                .get(META_SEED_KEY)
                .map_err(|e| e.to_string())?
                .ok_or("Missing seed")?;

            let seed_bytes: [u8; 8] = seed_value
                .value()
                .try_into()
                .map_err(|_| "Invalid seed bytes".to_string())?;

            let seed = u64::from_le_bytes(seed_bytes);

            let world_generator = String::from_utf8(
                table
                    .get(META_WORLD_GENERATOR_KEY)
                    .map_err(|e| e.to_string())?
                    .ok_or("Missing world_generator")?
                    .value()
                    .to_vec(),
            )
            .map_err(|e| e.to_string())?;

            let world_macro_data = serde_json::from_slice(
                table
                    .get(META_WORLD_MACRO_DATA_KEY)
                    .map_err(|e| e.to_string())?
                    .ok_or("Missing world_macro_data")?
                    .value(),
            )
            .map_err(|e| e.to_string())?;

            worlds.push(WorldStorageData::create(slug, seed, world_generator, world_macro_data));
        }

        Ok(worlds)
    }

    fn validate_block_id_map(&self, _block_id_map: &BTreeMap<BlockIndexType, String>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        chunks::{
            block_position::ChunkBlockPosition,
            chunk_data::{BlockDataInfo, ChunkData, ChunkSectionData},
            chunk_position::ChunkPosition,
            chunk_storage::{BlockInventory, ChunkStorage},
        },
        inventory::inventory::Inventory,
        utils::srotage_settings::StorageSettings,
        worlds_storage::{
            redb_storage::RedbWorldStorage,
            taits::{IWorldStorage, WorldStorageData},
        },
    };
    use rand::random;

    #[test]
    fn test_worlds() {
        let mut sections = ChunkData::default();
        sections.push_section(ChunkSectionData::default());
        sections.change_block(0, &ChunkBlockPosition::new(0, 0, 0), Some(BlockDataInfo::create(0)));

        let storage_data = WorldStorageData::default();

        let storage_settings = StorageSettings::in_memory();

        let storage = RedbWorldStorage::init(storage_settings, "default").unwrap();
        storage.create_new(&storage_data).unwrap();

        let chunk_position = ChunkPosition::new(0, 0);

        // Confirm that there is not chunk
        assert_eq!(storage.has_chunk_data(&chunk_position).unwrap(), None);

        // Save new chunk
        let sections = ChunkData::default();
        let data = ChunkStorage::create(sections);
        let chunk_id = storage.save_chunk_data(&chunk_position, &data).unwrap();
        let has_chunk_id = storage.has_chunk_data(&chunk_position).unwrap().unwrap();
        assert_eq!(has_chunk_id, chunk_id);

        // Save new chunk
        let mut sections = ChunkData::default();
        sections.push_section(ChunkSectionData::default());
        sections.change_block(0, &ChunkBlockPosition::new(0, 0, 0), Some(BlockDataInfo::create(2)));

        let mut data = ChunkStorage::create(sections);
        data.add_inventory(BlockInventory::create(
            0,
            ChunkBlockPosition::new(0, 0, 0),
            Inventory::create(0).with_id(random()),
        ));

        let updated_chunk_id = storage.save_chunk_data(&chunk_position, &data).unwrap();
        assert_eq!(has_chunk_id, updated_chunk_id);

        let chunk_storage = storage.read_chunk_data(has_chunk_id).unwrap();
        let loaded_sections = chunk_storage.get_chunk_data();

        assert_eq!(
            loaded_sections.get(0).unwrap().len(),
            data.get_chunk_data().get(0).unwrap().len()
        );

        storage.delete().unwrap();
    }
}
