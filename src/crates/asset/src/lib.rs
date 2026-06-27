use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::Arc,
};

static MANAGER: std::sync::LazyLock<AssetManager> =
    std::sync::LazyLock::new(|| AssetManager::new());

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AssetId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AssetKind {
    Mesh,
    Shader,
    Texture,
    Audio,
    Animation,
    GLTF,
}

impl AssetKind {
    pub fn from_path_ext(path: &String) -> Result<AssetKind, String> {
        let index = match path.rfind(".") {
            Some(i) => i,
            None => return Err(String::from("no extension in path")),
        };

        match &path[index..path.len()] {
            ".obj" => Ok(AssetKind::Mesh),
            ".mtl" => Ok(AssetKind::Mesh),
            ".gltf" => Ok(AssetKind::GLTF),
            ".glb" => Ok(AssetKind::GLTF),
            ".mp3" => Ok(AssetKind::Audio),
            ".wav" => Ok(AssetKind::Audio),
            ".ogg" => Ok(AssetKind::Audio),
            ".jpg" => Ok(AssetKind::Texture),
            ".jpeg" => Ok(AssetKind::Texture),
            ".png" => Ok(AssetKind::Texture),
            ".webp" => Ok(AssetKind::Texture),
            ext => Err(format!("unsupported extension {}", ext)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Asset {
    pub path: String,
    pub kind: AssetKind,
    pub raw: Vec<u8>,
}

impl Asset {
    fn new(path: String, kind: AssetKind, bytes: Vec<u8>) -> Asset {
        Asset {
            path,
            kind,
            raw: bytes,
        }
    }
}

#[derive(Debug)]
pub struct AssetManager {
    next_asset_id: u32,
    assets: Vec<Arc<Asset>>,
    kinds: HashMap<AssetKind, HashSet<AssetId>>,
}

impl AssetManager {
    fn new() -> AssetManager {
        AssetManager {
            next_asset_id: 0,
            assets: Vec::with_capacity(100),
            kinds: HashMap::with_capacity(6),
        }
    }

    pub fn load(path: String) -> Result<Arc<Asset>, String> {
        match fs::read(&path) {
            Ok(bytes) => {
                let kind = match AssetKind::from_path_ext(&path) {
                    Ok(k) => k,
                    Err(e) => return Err(e),
                };

                Ok(Arc::new(Asset::new(path, kind, bytes)))
            }
            Err(e) => Err(format!("unable to read file {}: {}", path, e)),
        }
    }

    pub fn get(id: AssetId) -> Option<Arc<Asset>> {
        if MANAGER.assets.len() < id.0 as usize {
            Some(MANAGER.assets[id.0 as usize].clone())
        } else {
            None
        }
    }

    pub fn get_by_kind(kind: &AssetKind) -> HashSet<AssetId> {
        if let Some(v) = MANAGER.kinds.get(kind) {
            v.clone()
        } else {
            HashSet::new()
        }
    }
}
