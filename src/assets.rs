//! Structs related to asset management.

#![allow(dead_code)]

use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt::Debug,
    path::{Path, PathBuf},
};

use crate::graphics::Tilesheet;

/// Any object that is loaded from a path, may be reused and requires management outside of its
/// users' scope, such as tilesheets or textures.
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
        #[allow(irrefutable_let_patterns)]
        if let Asset::Tilesheet(t) = value {
            Ok(t)
        } else {
            Err(())
        }
    }
}

/// A simple asset container that allows inserting and obtaining them.
pub struct AssetManager {
    assets: HashMap<PathBuf, Asset>,
}

impl AssetManager {
    /// Creates a new asset manager.
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    /// Obtains an asset from a specified path if already loaded and it exists.
    pub fn get_asset(&self, path: &Path) -> Option<&Asset> {
        self.assets.get(path)
    }

    /// Gets an asset associated to a path within the asset manager or otherwise inserts it in and
    /// then returns it.
    ///
    /// # Panics
    /// Panics if an asset already existed with the given path and its type doesn't match to the
    /// given one.
    pub fn get_or_insert_asset<'a, T>(&'a mut self, path: &Path, asset: T) -> &'a T
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
