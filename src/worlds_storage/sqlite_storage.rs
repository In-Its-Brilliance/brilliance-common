use super::taits::{IWorldStorage, WorldStorageData, WorldStorageSettings};
use crate::{
    chunks::{
        chunk_data::{BlockIndexType, WorldMacroData},
        chunk_position::ChunkPosition,
    },
    utils::compressable::Compressable,
};
use rusqlite::{blob::ZeroBlob, Connection, DatabaseName, OptionalExtension};
use std::{
    collections::BTreeMap,
    fs::{create_dir_all, read_dir, remove_file},
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
};

const SQL_TABLE_EXISTS: &str = "SELECT EXISTS(SELECT name FROM sqlite_master WHERE type='table' AND name='chunks');";

const SQL_CREATE_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS chunks (id INTEGER PRIMARY KEY, x INTEGER, z INTEGER, sections_data BLOB)";
const SQL_CREATE_INDEX: &str = "CREATE INDEX coordinate_index ON chunks (x, z)";

const SQL_CREATE_INFO_TABLE: &str =
    "CREATE TABLE IF NOT EXISTS world_info (seed TEXT, world_generator TEXT, world_macro BLOB);";
const SQL_WORLD_SET_INFO: &str = "INSERT INTO world_info (seed, world_generator, world_macro) VALUES (?1, ?2, ?3)";
const SQL_READ_WORLD_INFO: &str = "SELECT seed, world_generator, world_macro FROM world_info;";

const SQL_SELECT_CHUNK_ID: &str = "SELECT id FROM chunks WHERE x=?1 AND z=?2;";
const SQL_INSERT_CHUNK: &str = "INSERT INTO chunks (x, z, sections_data) VALUES (?1, ?2, ?3);";
const SQL_UPDATE_CHUNK: &str = "UPDATE chunks SET sections_data = ?2 WHERE id=?1";

const SQL_CREATE_TABLE_IDS: &str =
    "CREATE TABLE IF NOT EXISTS world_block_ids (block_id INTEGER UNIQUE, block_slug STRING);";
const SQL_SELECT_IDS: &str = "SELECT block_id, block_slug FROM world_block_ids ORDER BY block_id;";
const SQL_INSERT_ID: &str = "INSERT INTO world_block_ids (block_id, block_slug) VALUES (?1, ?2);";

struct BlockId {
    block_id: BlockIndexType,
    block_slug: String,
}

pub struct SQLiteStorage {
    db_path: PathBuf,
}

impl SQLiteStorage {
    fn open(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.get_db_path()).map_err(|e| e.to_string())?;

        conn.execute_batch("PRAGMA journal_mode = WAL;")
            .map_err(|e| e.to_string())?;

        Ok(conn)
    }

    fn get_db_path(&self) -> &PathBuf {
        &self.db_path
    }
}

impl IWorldStorage for SQLiteStorage {
    type Error = String;
    type PrimaryKey = i64;

    fn init(storage_settings: WorldStorageSettings, slug: impl Into<String>) -> Result<Self, String> {
        let mut db_path = storage_settings.get_data_path().clone();
        db_path.push("worlds");

        if create_dir_all(&db_path).is_err() {
            return Err(format!(
                "Unable to create dir \"{}\"",
                db_path.as_os_str().to_str().unwrap()
            ));
        }

        db_path.push(format!("{}.db", slug.into()));

        let storage = Self { db_path };
        Ok(storage)
    }

    fn create_new(&self, world_data: &WorldStorageData) -> Result<(), String> {
        let db = self.open()?;

        let chunks_exists: bool = db.query_row(SQL_TABLE_EXISTS, [], |row| row.get(0)).unwrap();

        if !chunks_exists {
            if let Err(e) = db.execute(SQL_CREATE_TABLE, ()) {
                return Err(format!("&4world chunks creation SQLite error: &c{}", e));
            }

            if let Err(e) = db.execute(SQL_CREATE_INDEX, ()) {
                return Err(format!("&4index creation SQLite error: &c{}", e));
            }

            if let Err(e) = db.execute(SQL_CREATE_INFO_TABLE, ()) {
                return Err(format!("&4World Info writing SQLite error: &c{}", e));
            }

            if let Err(e) = db.execute(
                SQL_WORLD_SET_INFO,
                (
                    world_data.get_seed().to_string(),
                    world_data.get_world_generator(),
                    world_data.get_world_macro_data().encode(),
                ),
            ) {
                return Err(format!("world seed saving error: &c{}", e));
            }

            log::info!(target: "worlds", "world db &e\"{}\"&r created", self.get_db_path().as_os_str().to_str().unwrap());
        }

        Ok(())
    }

    fn has_chunk_data(&self, chunk_position: &ChunkPosition) -> Result<Option<Self::PrimaryKey>, String> {
        let db = self.open()?;

        let chunks_exists: rusqlite::Result<i64> =
            db.query_row(SQL_SELECT_CHUNK_ID, (chunk_position.x, chunk_position.z), |row| {
                row.get(0)
            });
        let r = match chunks_exists.optional() {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("World seed save error: &c{}", e));
            }
        };
        return Ok(r);
    }

    fn read_chunk_data(&self, chunk_id: Self::PrimaryKey) -> Result<Vec<u8>, String> {
        let db = self.open()?;
        let blob = db
            .blob_open(DatabaseName::Main, "chunks", "sections_data", chunk_id.clone(), true)
            .unwrap();
        let mut encoded = vec![0u8; blob.size() as usize];
        blob.read_at_exact(&mut encoded, 0).unwrap();
        Ok(encoded)
    }

    fn save_chunk_data(&self, chunk_position: &ChunkPosition, data: &Vec<u8>) -> Result<Self::PrimaryKey, String> {
        let db = self.open()?;
        let id = match self.has_chunk_data(chunk_position) {
            Ok(id) => id,
            Err(e) => return Err(e),
        };
        let chunk_id = match id {
            Some(id) => {
                if let Err(e) = db.execute(SQL_UPDATE_CHUNK, (&id, ZeroBlob(data.len() as i32))) {
                    return Err(format!("&4Chunk update SQLite error: &c{}", e));
                }
                id
            }
            None => {
                if let Err(e) = db.execute(
                    SQL_INSERT_CHUNK,
                    (chunk_position.x, chunk_position.z, ZeroBlob(data.len() as i32)),
                ) {
                    return Err(format!("&4Chunk insert SQLite error: &c{}", e));
                }
                let id = db.last_insert_rowid();
                id
            }
        };

        let mut blob = db
            .blob_open(DatabaseName::Main, "chunks", "sections_data", chunk_id.clone(), false)
            .unwrap();
        let bytes_written = blob.write(data.as_slice()).unwrap();
        assert_eq!(data.len(), bytes_written);
        blob.seek(SeekFrom::Start(0)).unwrap();

        Ok(chunk_id)
    }

    fn scan_worlds(storage_settings: WorldStorageSettings) -> Result<Vec<WorldStorageData>, String> {
        let mut worlds: Vec<WorldStorageData> = Default::default();

        let mut folder_path = storage_settings.get_data_path().clone();
        folder_path.push("worlds");
        if let Err(e) = std::fs::create_dir_all(folder_path.clone()) {
            return Err(format!(
                "&ccreate directory &4\"{}\"&r error:\n&c{}",
                folder_path.as_os_str().to_str().unwrap(),
                e
            ));
        }

        let paths = match read_dir(folder_path.clone()) {
            Ok(p) => p,
            Err(e) => {
                return Err(format!(
                    "&cread directory &4\"{}\"&r error:\n&c{}",
                    folder_path.as_os_str().to_str().unwrap(),
                    e
                ));
            }
        };
        for path in paths {
            let path = path.unwrap().path();
            let filename = path.file_name().unwrap().to_str().unwrap();
            let path = path.as_os_str().to_str().unwrap();
            if !path.ends_with(".db") {
                continue;
            }
            let db = match Connection::open(path) {
                Ok(c) => c,
                Err(e) => return Err(format!("&cdatabase creation error: {}", e)),
            };
            let world_data = match db.query_row(SQL_READ_WORLD_INFO, [], |row| {
                let macro_bytes = row.get::<_, Vec<u8>>(2)?;
                let macro_data =
                    WorldMacroData::decode(macro_bytes).map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
                Ok(WorldStorageData::create(
                    filename.replace(".db", ""),
                    row.get::<_, String>(0)?.parse::<u64>().unwrap(),
                    row.get::<_, String>(1)?,
                    macro_data,
                ))
            }) {
                Ok(s) => s,
                Err(e) => {
                    return Err(format!(
                        "&cworld &4\"{}\"\n&4World Info SQLite reading error: &c{}",
                        path, e
                    ))
                }
            };
            worlds.push(world_data);
        }

        Ok(worlds)
    }

    fn delete(&self) -> Result<(), String> {
        if let Err(e) = remove_file(self.get_db_path().clone()) {
            return Err(format!(
                "world delete &e\"{}\"&r error: {}",
                self.get_db_path().as_os_str().to_str().unwrap(),
                e
            ));
        };
        log::info!(target: "worlds", "World db &e\"{}\"&r deleted", self.get_db_path().to_str().unwrap());
        Ok(())
    }

    fn validate_block_id_map(&self, block_id_map: &BTreeMap<BlockIndexType, String>) -> Result<(), String> {
        let db = self.open()?;

        if let Err(e) = db.execute(SQL_CREATE_TABLE_IDS, ()) {
            return Err(format!("World block ids table create error: &c{}", e));
        }

        let mut stmt = db.prepare(SQL_SELECT_IDS).unwrap();
        let ids_result = stmt
            .query_map([], |row| {
                Ok(BlockId {
                    block_id: row.get(0).unwrap(),
                    block_slug: row.get(1).unwrap(),
                })
            })
            .unwrap();

        let mut existing_blocks: Vec<String> = Default::default();
        for block_row in ids_result {
            let block_row = block_row.unwrap();

            // Check that saved id map contains all block from world
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

        // Check that all blocks exists inside world and write if not
        for (block_id, block_slug) in block_id_map.iter() {
            if !existing_blocks.contains(&block_slug) {
                // Block id is not exists in the world;
                if let Err(e) = db.execute(SQL_INSERT_ID, (block_id.clone(), block_slug.clone())) {
                    return Err(format!(
                        "Block id #{} \"{}\" insert error: &c{}",
                        block_id, block_slug, e
                    ));
                }
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
        },
        utils::compressable::Compressable,
        worlds_storage::{
            sqlite_storage::SQLiteStorage,
            taits::{IWorldStorage, WorldStorageData, WorldStorageSettings},
        },
    };

    #[test]
    fn test_worlds() {
        let mut sections = ChunkData::default();
        sections.push_section(ChunkSectionData::default());
        sections.change_block(
            0,
            &ChunkBlockPosition::new(0, 0, 0),
            Some(BlockDataInfo::create(0)),
        );

        let storage_data = WorldStorageData::default();

        let storage_settings = WorldStorageSettings::in_memory();

        let storage = SQLiteStorage::init(storage_settings, "default").unwrap();
        storage.create_new(&storage_data).unwrap();

        let chunk_position = ChunkPosition::new(0, 0);

        // Confirm that there is not chunk
        assert_eq!(storage.has_chunk_data(&chunk_position).unwrap(), None);

        // Save new chunk
        let chunk_id = storage.save_chunk_data(&chunk_position, &sections.compress()).unwrap();
        let has_chunk_id = storage.has_chunk_data(&chunk_position).unwrap().unwrap();
        assert_eq!(has_chunk_id, chunk_id);

        // Save new chunk
        let mut sections = ChunkData::default();
        sections.push_section(ChunkSectionData::default());
        sections.change_block(0, &ChunkBlockPosition::new(0, 0, 0), Some(BlockDataInfo::create(2)));

        let updated_chunk_id = storage.save_chunk_data(&chunk_position, &sections.compress()).unwrap();
        assert_eq!(has_chunk_id, updated_chunk_id);

        let encoded = storage.read_chunk_data(has_chunk_id).unwrap();
        let loaded_sections = ChunkData::decompress(encoded).unwrap();

        assert_eq!(loaded_sections.get(0).unwrap().len(), sections.get(0).unwrap().len());

        storage.delete().unwrap();
    }
}
