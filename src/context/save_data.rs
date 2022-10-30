use std::{collections::HashSet, fs::File, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[cfg_attr(feature = "editor", derive(guiedit_derive::Inspectable))]
pub struct SaveData {
    set: HashSet<PathBuf>,
}

impl SaveData {
    pub fn from_savefile() -> anyhow::Result<Self> {
        Ok(ron::de::from_reader::<_, Self>(File::open(
            Self::save_file_path(),
        )?)?)
    }

    /// Get a reference to the level completion db's internal set.
    pub fn internal_set(&self) -> &HashSet<PathBuf> {
        &self.set
    }

    pub fn complete_lvl(&mut self, level_completed: PathBuf) {
        if level_completed.is_absolute() {
            log::warn!("added absolute path to level completion db, this should not happen");
        }

        self.set.insert(level_completed);
        let path_to_save_to = Self::save_file_path();
        std::fs::create_dir_all(&path_to_save_to.parent().unwrap())
            .expect("could not create dirs up to project data dir");
        let file = match File::create(&path_to_save_to) {
            Ok(file) => file,
            Err(err) => {
                log::error!("could not create save file: {}", err);
                return;
            }
        };
        if let Err(err) = ron::ser::to_writer(file, &self) {
            log::error!("could not save progress: {}", err);
        } else {
            log::info!("updated savefile at {:?}", path_to_save_to);
        }
    }

    pub fn save_file_path() -> PathBuf {
        ProjectDirs::from("", "rusty-pizza", env!("CARGO_PKG_NAME"))
            .expect("could not obtain project directories")
            .data_dir()
            .join("levels.ron")
    }
}
