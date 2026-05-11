use super::taits::{IWorldStorage, WorldStorageData};
use crate::{
    chunks::{
        block_position::ChunkBlockPosition,
        chunk_data::{BlockIndexType, ChunkData, WorldMacroData},
        chunk_position::ChunkPosition,
        chunk_storage::{BlockInventory, ChunkStorage, WorldItem},
        position::Vector3,
    },
    inventory::{inventory::Inventory, item::Item},
    utils::{compressable::Compressable, srotage_settings::StorageSettings},
};
use rusqlite::{Connection, DatabaseName, OptionalExtension};
use std::{
    collections::BTreeMap,
    fs::{create_dir_all, read_dir, remove_file},
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

const SQL_CREATE_ITEM_TYPES_TABLE: &str = "
CREATE TABLE IF NOT EXISTS world_item_types (
    type_id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE
);
";
const SQL_CREATE_BLOCK_INVENTORIES_TABLE: &str = "
CREATE TABLE IF NOT EXISTS block_inventories (
    id INTEGER PRIMARY KEY,
    chunk_id INTEGER NOT NULL,

    section INTEGER NOT NULL,

    block_x INTEGER NOT NULL,
    block_y INTEGER NOT NULL,
    block_z INTEGER NOT NULL,

    FOREIGN KEY(chunk_id) REFERENCES chunks(id) ON DELETE CASCADE,

    UNIQUE(chunk_id, block_x, block_y, block_z)
);
";
const SQL_CREATE_BLOCK_INVENTORY_SLOTS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS block_inventory_slots (
    inventory_id INTEGER NOT NULL,
    slot_index INTEGER NOT NULL,

    type_id INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    modifiers BLOB NOT NULL,

    PRIMARY KEY(inventory_id, slot_index),

    FOREIGN KEY(inventory_id) REFERENCES block_inventories(id) ON DELETE CASCADE,

    FOREIGN KEY(type_id) REFERENCES world_item_types(type_id)
);
";
const SQL_CREATE_WORLD_ITEMS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS world_items (
    id INTEGER PRIMARY KEY,
    chunk_id INTEGER NOT NULL,

    x REAL NOT NULL,
    y REAL NOT NULL,
    z REAL NOT NULL,

    type_id INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    modifiers BLOB NOT NULL,

    FOREIGN KEY(chunk_id) REFERENCES chunks(id) ON DELETE CASCADE,

    FOREIGN KEY(type_id) REFERENCES world_item_types(type_id));
";

const SQL_CREATE_WORLD_ITEMS_CHUNK_INDEX: &str = "
CREATE INDEX IF NOT EXISTS world_items_chunk_index ON world_items(chunk_id);
";
const SQL_SELECT_BLOCK_INVENTORIES: &str = "
SELECT id, section, block_x, block_y, block_z
FROM block_inventories
WHERE chunk_id = ?1;
";
const SQL_SELECT_BLOCK_INVENTORY_SLOTS: &str = "
SELECT
    s.slot_index,
    t.slug,
    s.amount,
    s.modifiers
FROM block_inventory_slots s
INNER JOIN world_item_types t ON t.type_id = s.type_id
WHERE s.inventory_id = ?1
ORDER BY s.slot_index;
";
const SQL_SELECT_WORLD_ITEMS: &str = "
SELECT
    wi.x,
    wi.y,
    wi.z,
    t.slug,
    wi.amount,
    wi.modifiers
FROM world_items wi
INNER JOIN world_item_types t ON t.type_id = wi.type_id
WHERE wi.chunk_id = ?1;
";

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

        conn.execute_batch("PRAGMA journal_mode = WAL;")
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

        let storage = Self { db_path };
        Ok(storage)
    }

    fn create_new(&self, world_data: &WorldStorageData) -> Result<(), String> {
        let db = self.open()?;

        let table_exists: bool = db.query_row(SQL_TABLE_EXISTS, [], |row| row.get(0)).unwrap();

        if !table_exists {
            if let Err(e) = db.execute(SQL_CREATE_TABLE, ()) {
                return Err(format!("&4world chunks creation SQLite error: &c{}", e));
            }

            if let Err(e) = db.execute(SQL_CREATE_INDEX, ()) {
                return Err(format!("&4index creation SQLite error: &c{}", e));
            }

            if let Err(e) = db.execute(SQL_CREATE_INFO_TABLE, ()) {
                return Err(format!("&4World Info writing SQLite error: &c{}", e));
            }

            db.execute(SQL_CREATE_ITEM_TYPES_TABLE, ())
                .map_err(|e| format!("&4world item types table creation SQLite error: &c{}", e))?;
            db.execute(SQL_CREATE_BLOCK_INVENTORIES_TABLE, ())
                .map_err(|e| format!("&4block inventories table creation SQLite error: &c{}", e))?;
            db.execute(SQL_CREATE_BLOCK_INVENTORY_SLOTS_TABLE, ())
                .map_err(|e| format!("&4block inventory slots table creation SQLite error: &c{}", e))?;
            db.execute(SQL_CREATE_WORLD_ITEMS_TABLE, ())
                .map_err(|e| format!("&4world items table creation SQLite error: &c{}", e))?;
            db.execute(SQL_CREATE_WORLD_ITEMS_CHUNK_INDEX, ())
                .map_err(|e| format!("&4world items chunk index creation SQLite error: &c{}", e))?;

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

    fn read_chunk_data(&self, chunk_id: Self::PrimaryKey) -> Result<ChunkStorage, String> {
        let db = self.open()?;

        let blob = db
            .blob_open(DatabaseName::Main, "chunks", "sections_data", chunk_id, true)
            .map_err(|e| e.to_string())?;

        let mut encoded = vec![0u8; blob.size() as usize];

        blob.read_at_exact(&mut encoded, 0).map_err(|e| e.to_string())?;

        let chunk_data = ChunkData::decompress(encoded)?;

        let mut inventories = Vec::new();

        let mut inventories_stmt = db.prepare(SQL_SELECT_BLOCK_INVENTORIES).map_err(|e| e.to_string())?;

        let inventory_rows = inventories_stmt
            .query_map((chunk_id,), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, u32>(1)?,
                    ChunkBlockPosition::new(row.get::<_, u8>(2)?, row.get::<_, u8>(3)?, row.get::<_, u8>(4)?),
                ))
            })
            .map_err(|e| e.to_string())?;

        for inventory_row in inventory_rows {
            let (inventory_id, section, position) = inventory_row.map_err(|e| e.to_string())?;

            let mut slots = Vec::<Option<Item>>::new();

            let mut slots_stmt = db
                .prepare(SQL_SELECT_BLOCK_INVENTORY_SLOTS)
                .map_err(|e| e.to_string())?;

            let slot_rows = slots_stmt
                .query_map((inventory_id,), |row| {
                    let modifiers_bytes = row.get::<_, Vec<u8>>(3)?;

                    let modifiers = bincode::deserialize(&modifiers_bytes)
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                    Ok((
                        row.get::<_, u16>(0)?,
                        Item::create(row.get::<_, String>(1)?, row.get::<_, u16>(2)?, modifiers),
                    ))
                })
                .map_err(|e| e.to_string())?;

            for slot_row in slot_rows {
                let (slot_index, item) = slot_row.map_err(|e| e.to_string())?;
                let slot_index = slot_index as usize;

                if slots.len() <= slot_index {
                    slots.resize_with(slot_index + 1, || None);
                }

                slots[slot_index] = Some(item);
            }

            inventories.push(BlockInventory::create(section, position, Inventory::create(slots)));
        }

        let mut items = Vec::new();

        let mut items_stmt = db.prepare(SQL_SELECT_WORLD_ITEMS).map_err(|e| e.to_string())?;

        let item_rows = items_stmt
            .query_map((chunk_id,), |row| {
                let modifiers_bytes = row.get::<_, Vec<u8>>(5)?;

                let modifiers = bincode::deserialize(&modifiers_bytes)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

                Ok(WorldItem::create(
                    Vector3::new(row.get::<_, f32>(0)?, row.get::<_, f32>(1)?, row.get::<_, f32>(2)?),
                    Item::create(row.get::<_, String>(3)?, row.get::<_, u16>(4)?, modifiers),
                ))
            })
            .map_err(|e| e.to_string())?;

        for item_row in item_rows {
            items.push(item_row.map_err(|e| e.to_string())?);
        }

        Ok(ChunkStorage::create(chunk_data).inventories(inventories).items(items))
    }

    fn save_chunk_data(&self, chunk_position: &ChunkPosition, data: &ChunkStorage) -> Result<Self::PrimaryKey, String> {
        let mut db = self.open()?;
        let tx = db.transaction().map_err(|e| e.to_string())?;

        let chunk_id = tx
            .query_row(SQL_SELECT_CHUNK_ID, (chunk_position.x, chunk_position.z), |row| {
                row.get::<_, i64>(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;

        let encoded = data.get_chunk_data().compress();

        let chunk_id = match chunk_id {
            Some(id) => {
                tx.execute(SQL_UPDATE_CHUNK, (&id, encoded.as_slice()))
                    .map_err(|e| format!("&4Chunk update SQLite error: &c{}", e))?;

                id
            }

            None => {
                tx.execute(
                    SQL_INSERT_CHUNK,
                    (chunk_position.x, chunk_position.z, encoded.as_slice()),
                )
                .map_err(|e| format!("&4Chunk insert SQLite error: &c{}", e))?;

                tx.last_insert_rowid()
            }
        };

        tx.commit().map_err(|e| e.to_string())?;

        Ok(chunk_id)
    }

    fn scan_worlds(storage_settings: StorageSettings) -> Result<Vec<WorldStorageData>, String> {
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
