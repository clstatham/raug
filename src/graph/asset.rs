use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};

use rustc_hash::FxHashMap;

use crate::prelude::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Asset {
    Buffer(Buffer<Float>),
}

impl Asset {
    pub fn as_buffer(&self) -> Option<&Buffer<Float>> {
        match self {
            Asset::Buffer(buffer) => Some(buffer),
        }
    }

    pub fn as_buffer_mut(&mut self) -> Option<&mut Buffer<Float>> {
        match self {
            Asset::Buffer(buffer) => Some(buffer),
        }
    }
}

impl From<Buffer<Float>> for Asset {
    fn from(buffer: Buffer<Float>) -> Self {
        Asset::Buffer(buffer)
    }
}

#[derive(Debug)]
pub struct AssetRef<'a>(MutexGuard<'a, Asset>);

impl Deref for AssetRef<'_> {
    type Target = Asset;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AssetRef<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct Assets {
    assets: FxHashMap<String, Arc<Mutex<Asset>>>,
}

impl Assets {
    pub fn new() -> Self {
        Self {
            assets: FxHashMap::default(),
        }
    }

    pub fn get(&self, name: &str) -> Option<AssetRef> {
        self.assets
            .get(name)
            .and_then(|asset| asset.try_lock().ok())
            .map(AssetRef)
    }

    pub fn insert(&mut self, name: String, asset: Asset) {
        self.assets.insert(name, Arc::new(Mutex::new(asset)));
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for Assets {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let assets = self
                .assets
                .iter()
                .map(|(name, asset)| (name, asset.lock().unwrap().clone()))
                .collect::<FxHashMap<_, _>>();

            assets.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for Assets {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let assets = FxHashMap::deserialize(deserializer)?
                .into_iter()
                .map(|(name, asset)| (name, Arc::new(Mutex::new(asset))))
                .collect();

            Ok(Self { assets })
        }
    }
}
