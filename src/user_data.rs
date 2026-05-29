use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod config_data;
pub mod profile;

#[async_trait]
pub trait DataCollector {
    type Data: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    fn create_from_local(name: &str, location: impl AsRef<Path>) -> Self;
    fn name(&self) -> Cow<'_, str>;
    async fn read_local(&self) -> Self::Data;
    async fn write_local(&self, data: Self::Data);
}

pub type BackupItem = PathBuf;

pub struct UserData<T: DataCollector> {
    bak_dir: PathBuf,
    collectors: Vec<T>,
}

impl<T: DataCollector> UserData<T> {
    pub fn new(bak_dir: impl AsRef<Path>) -> Self {
        Self {
            bak_dir: bak_dir.as_ref().to_path_buf(),
            collectors: vec![],
        }
    }

    pub fn push(&mut self, collector: T) {
        self.collectors.push(collector);
    }

    pub fn remove(&mut self, name: &str) {
        let mut loc: usize = 0;
        for (i, item) in self.collectors.iter().enumerate() {
            if item.name() == name {
                loc = i;
                break;
            }
        }

        _ = self.collectors.remove(loc);
    }

    pub async fn create_bakup() -> BackupItem {}
}
