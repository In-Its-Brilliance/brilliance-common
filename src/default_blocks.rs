use crate::blocks::block_type::{BlockType, BlockTypeManifest};

/// Их необходимо хранить в общей библиотеки, т.к. их используют клиент и сервер
/// Клиент загружает их по дефолту
/// Сервер сохраняет id в соответствии со slug блоков и передает на клиент

pub fn generate_default_blocks() -> Result<Vec<BlockType>, String> {
    let text = include_str!("default_blocks.yml");
    let m: Result<Vec<BlockTypeManifest>, serde_yaml::Error> = serde_yaml::from_str(text);

    let m = match m {
        Ok(m) => m,
        Err(e) => return Err(format!("&cyaml parsing error: {}", e)),
    };
    // println!("{}", serde_yaml::to_string(&m).unwrap());

    let m: Vec<BlockType> = m.iter().map(|m| m.to_block()).collect();
    Ok(m)
}
