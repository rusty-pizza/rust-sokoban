use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::tilesheet::Tilesheet;

pub enum Asset {
    Tilesheet(Tilesheet),
}

impl From<Tilesheet> for Asset {
    fn from(x: Tilesheet) -> Self {
        Self::Tilesheet(x)
    }
}

impl<'a> TryFrom<&'a Asset> for &'a Tilesheet {
    type Error = ();

    fn try_from(value: &'a Asset) -> Result<Self, Self::Error> {
        if let Asset::Tilesheet(t) = value {
            Ok(t)
        } else {
            Err(())
        }
    }
}

pub struct AssetManager {
    assets: HashMap<PathBuf, Asset>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn get_asset(&self, path: &Path) -> Option<&Asset> {
        self.assets.get(path)
    }

    pub fn get_or_load_asset<'a, T>(&'a mut self, path: &Path, asset: T) -> &'a T
    where
        T: Into<Asset>,
        &'a T: TryFrom<&'a Asset>,
        <&'a T as TryFrom<&'a Asset>>::Error: Debug,
    {
        if self.assets.contains_key(path) {
            return <&'a T>::try_from(&self.assets[path])
                .expect("Tried to load or get asset with different type than expected");
        } else {
            self.assets.insert(path.to_owned(), asset.into());
            self.assets.get(path).unwrap().try_into().unwrap()
        }
    }
}
