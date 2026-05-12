use super::taits::{IWorldStorage, WorldStorageData};
use crate::{
    chunks::{
        chunk_data::{BlockIndexType, WorldMacroData},
        chunk_position::ChunkPosition,
        chunk_storage::ChunkStorage,
    },
    utils::{compressable::Compressable, srotage_settings::StorageSettings},
};
use rusqlite::{Connection, DatabaseName, OptionalExtension};
use std::{
    collections::BTreeMap,
    fs::{create_dir_all, read_dir, remove_file},
    path::PathBuf,
};

const SQL_TABLE_EXISTS: &str = "SELECT EXISTS(SELECT name FROM sqlite_master WHERE type='table' AND name='chunks');";

const SQL_CREATE_TABLE: &str = "
CREATE TABLE IF NOT EXISTS chunks (
    id INTEGER PRIMARY KEY,
    x INTEGER,
    z INTEGER,
    data BLOB
)";
const SQL_CREATE_INDEX: &str = "CREATE INDEX coordinate_index ON chunks (x, z)";

const SQL_CREATE_INFO_TABLE: &str = "
CREATE TABLE IF NOT EXISTS world_info (
    seed TEXT,
    world_generator TEXT,
    world_macro BLOB
);
";

const SQL_WORLD_SET_INFO: &str = "INSERT INTO world_info (seed, world_generator, world_macro) VALUES (?1, ?2, ?3)";

const SQL_READ_WORLD_INFO: &str = "SELECT seed, world_generator, world_macro FROM world_info;";

const SQL_SELECT_CHUNK_ID: &str = "SELECT id FROM chunks WHERE x=?1 AND z=?2;";

const SQL_INSERT_CHUNK: &str = "INSERT INTO chunks (x, z, data) VALUES (?1, ?2, ?3);";

const SQL_UPDATE_CHUNK: &str = "UPDATE chunks SET data = ?2 WHERE id=?1";

const SQL_CREATE_TABLE_IDS: &str = "
CREATE TABLE IF NOT EXISTS world_block_ids (
    block_id INTEGER UNIQUE,
    block_slug STRING
);
";

const SQL_SELECT_IDS: &str = "SELECT block_id, block_slug FROM world_block_ids ORDER BY block_id;";

const SQL_INSERT_ID: &str = "INSERT INTO world_block_ids (block_id, block_slug) VALUES (?1, ?2);";

struct BlockId {
    block_id: BlockIndexType,
    block_slug: String,
}

pub struct SQLiteWorldStorage {
    db_path: PathBuf,
}

impl SQLiteWorldStorage {
    fn open(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.get_db_path()).map_err(|e| e.to_string())?;

        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
        ",
        )
        .map_err(|e| e.to_string())?;

        Ok(conn)
    }

    fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }
}

impl IWorldStorage for SQLiteWorldStorage {
    type Error = String;
    type PrimaryKey = i64;

    fn init(storage_settings: StorageSettings, slug: impl Into<String>) -> Result<Self, String> {
        let mut db_path = storage_settings.get_data_path().clone();

        db_path.push("worlds");

        if create_dir_all(&db_path).is_err() {
            return Err(format!(
                "Unable to create dir \"{}\"",
                db_path.as_os_str().to_str().unwrap()
            ));
        }

        db_path.push(format!("{}.db", slug.into()));

        Ok(Self { db_path })
    }

    fn create_new(&self, world_data: &WorldStorageData) -> Result<(), String> {
        let db = self.open()?;

        let table_exists: bool = db.query_row(SQL_TABLE_EXISTS, [], |row| row.get(0)).unwrap();

        if !table_exists {
            db.execute(SQL_CREATE_TABLE, ())
                .map_err(|e| format!("&4world chunks creation SQLite error: &c{}", e))?;

            db.execute(SQL_CREATE_INDEX, ())
                .map_err(|e| format!("&4index creation SQLite error: &c{}", e))?;

            db.execute(SQL_CREATE_INFO_TABLE, ())
                .map_err(|e| format!("&4World Info writing SQLite error: &c{}", e))?;

            db.execute(
                SQL_WORLD_SET_INFO,
                (
                    world_data.get_seed().to_string(),
                    world_data.get_world_generator(),
                    world_data.get_world_macro_data().encode(),
                ),
            )
            .map_err(|e| format!("world seed saving error: &c{}", e))?;

            log::info!(
                target: "worlds",
                "world db &e\"{}\"&r created",
                self.get_db_path()
                    .as_os_str()
                    .to_str()
                    .unwrap()
            );
        }

        Ok(())
    }

    fn has_chunk_data(&self, chunk_position: &ChunkPosition) -> Result<Option<Self::PrimaryKey>, String> {
        let db = self.open()?;

        let result: rusqlite::Result<i64> =
            db.query_row(SQL_SELECT_CHUNK_ID, (chunk_position.x, chunk_position.z), |row| {
                row.get(0)
            });

        result
            .optional()
            .map_err(|e| format!("World chunk lookup error: &c{}", e))
    }

    fn read_chunk_data(&self, chunk_id: Self::PrimaryKey) -> Result<ChunkStorage, String> {
        let db = self.open()?;

        let blob = db
            .blob_open(DatabaseName::Main, "chunks", "data", chunk_id, true)
            .map_err(|e| e.to_string())?;

        let mut encoded = vec![0u8; blob.size() as usize];

        blob.read_at_exact(&mut encoded, 0).map_err(|e| e.to_string())?;

        ChunkStorage::decompress(encoded)
    }

    fn save_chunk_data(&self, chunk_position: &ChunkPosition, data: &ChunkStorage) -> Result<Self::PrimaryKey, String> {
        let db = self.open()?;

        let chunk_id = db
            .query_row(SQL_SELECT_CHUNK_ID, (chunk_position.x, chunk_position.z), |row| {
                row.get::<_, i64>(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;

        let encoded = data.compress();

        let chunk_id = match chunk_id {
            Some(id) => {
                db.execute(SQL_UPDATE_CHUNK, (&id, encoded.as_slice()))
                    .map_err(|e| format!("&4Chunk update SQLite error: &c{}", e))?;

                id
            }

            None => {
                db.execute(
                    SQL_INSERT_CHUNK,
                    (chunk_position.x, chunk_position.z, encoded.as_slice()),
                )
                .map_err(|e| format!("&4Chunk insert SQLite error: &c{}", e))?;

                db.last_insert_rowid()
            }
        };

        Ok(chunk_id)
    }

    fn scan_worlds(storage_settings: StorageSettings) -> Result<Vec<WorldStorageData>, String> {
        let mut worlds = Vec::new();

        let mut folder_path = storage_settings.get_data_path().clone();

        folder_path.push("worlds");

        create_dir_all(folder_path.clone()).map_err(|e| {
            format!(
                "&ccreate directory &4\"{}\"&r error:\n&c{}",
                folder_path.as_os_str().to_str().unwrap(),
                e
            )
        })?;

        let paths = read_dir(folder_path.clone()).map_err(|e| {
            format!(
                "&cread directory &4\"{}\"&r error:\n&c{}",
                folder_path.as_os_str().to_str().unwrap(),
                e
            )
        })?;

        for path in paths {
            let path = path.unwrap().path();

            let filename = path.file_name().unwrap().to_str().unwrap();

            let path_str = path.as_os_str().to_str().unwrap();

            if !path_str.ends_with(".db") {
                continue;
            }

            let db = Connection::open(path_str).map_err(|e| format!("&cdatabase creation error: {}", e))?;

            let world_data = db
                .query_row(SQL_READ_WORLD_INFO, [], |row| {
                    let macro_bytes = row.get::<_, Vec<u8>>(2)?;

                    let macro_data =
                        WorldMacroData::decode(macro_bytes).map_err(|e| rusqlite::Error::InvalidParameterName(e))?;

                    Ok(WorldStorageData::create(
                        filename.replace(".db", ""),
                        row.get::<_, String>(0)?.parse::<u64>().unwrap(),
                        row.get::<_, String>(1)?,
                        macro_data,
                    ))
                })
                .map_err(|e| format!("&cworld &4\"{}\"\n&4World Info SQLite reading error: &c{}", path_str, e))?;

            worlds.push(world_data);
        }

        Ok(worlds)
    }

    fn delete(&self) -> Result<(), String> {
        remove_file(self.get_db_path().clone()).map_err(|e| {
            format!(
                "world delete &e\"{}\"&r error: {}",
                self.get_db_path().as_os_str().to_str().unwrap(),
                e
            )
        })?;

        log::info!(
            target: "worlds",
            "World db &e\"{}\"&r deleted",
            self.get_db_path().to_str().unwrap()
        );

        Ok(())
    }

    fn validate_block_id_map(&self, block_id_map: &BTreeMap<BlockIndexType, String>) -> Result<(), String> {
        let db = self.open()?;

        db.execute(SQL_CREATE_TABLE_IDS, ())
            .map_err(|e| format!("World block ids table create error: &c{}", e))?;

        let mut stmt = db.prepare(SQL_SELECT_IDS).unwrap();

        let ids_result = stmt
            .query_map([], |row| {
                Ok(BlockId {
                    block_id: row.get(0).unwrap(),
                    block_slug: row.get(1).unwrap(),
                })
            })
            .unwrap();

        let mut existing_blocks = Vec::<String>::new();

        for block_row in ids_result {
            let block_row = block_row.unwrap();

            let mut block_exists = false;

            for (block_id, block_slug) in block_id_map.iter() {
                if *block_slug == block_row.block_slug {
                    if *block_id != block_row.block_id {
                        return Err(format!(
                            "&cblock &4\"{}\"&c id is not match; world_id:{} saved_id:{}",
                            block_slug, block_row.block_id, block_id
                        ));
                    }

                    block_exists = true;
                }
            }

            if !block_exists {
                return Err(format!(
                    "&cblock &4\"{}\"&c doesn't exists in resources",
                    block_row.block_slug
                ));
            }

            existing_blocks.push(block_row.block_slug.clone());
        }

        for (block_id, block_slug) in block_id_map.iter() {
            if !existing_blocks.contains(block_slug) {
                db.execute(SQL_INSERT_ID, (block_id.clone(), block_slug.clone()))
                    .map_err(|e| format!("Block id #{} \"{}\" insert error: &c{}", block_id, block_slug, e))?;
            }
        }

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
            sqlite_storage::SQLiteWorldStorage,
            taits::{IWorldStorage, WorldStorageData},
        },
    };

    #[test]
    fn test_worlds() {
        let mut sections = ChunkData::default();
        sections.push_section(ChunkSectionData::default());
        sections.change_block(0, &ChunkBlockPosition::new(0, 0, 0), Some(BlockDataInfo::create(0)));

        let storage_data = WorldStorageData::default();

        let storage_settings = StorageSettings::in_memory();

        let storage = SQLiteWorldStorage::init(storage_settings, "default").unwrap();
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
            Inventory::default(),
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
