use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Can not load json content: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("IO fail: {0}")]
    IOError(#[from] tokio::io::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BakData {
    pub comp_favs: Option<String>,
}

impl BakData {
    pub async fn from_config_content(path_to_config: impl AsRef<Path>) -> Result<Self, DataError> {
        let content = tokio::fs::read_to_string(&path_to_config).await?;
        let content: Value = serde_json::from_str(&content)?;

        let bak_data = BakData {
            comp_favs: content
                .get("CompFavorites")
                .map(|x| x.as_str().unwrap().to_string()),
        };

        Ok(bak_data)
    }

    pub async fn apply_config_content(
        &self,
        path_to_config: impl AsRef<Path>,
    ) -> Result<(), DataError> {
        let cfg_data = tokio::fs::read_to_string(&path_to_config).await?;
        let mut cfg_data: Value = serde_json::from_str(&cfg_data)?;

        if let Some(comp_fav) = &self.comp_favs {
            let new_val = Value::String(comp_fav.clone());
            if let Some(comp_fav_item) = cfg_data.get_mut("CompFavorites") {
                *comp_fav_item = new_val;
            } else {
                _ = cfg_data
                    .as_object_mut()
                    .unwrap()
                    .insert("CompFavorites".to_string(), new_val);
            }
        }

        tokio::fs::write(&path_to_config, serde_json::to_string(&cfg_data)?).await?;

        Ok(())
    }

    pub async fn save_to(&self, path_to_bak_file: impl AsRef<Path>) -> Result<(), DataError> {
        let content = serde_json::to_string(&self)?;
        tokio::fs::write(&path_to_bak_file, &content).await?;

        Ok(())
    }

    pub async fn create_from(path_to_bak: impl AsRef<Path>) -> Result<Self, DataError> {
        let bak_content = tokio::fs::read_to_string(&path_to_bak).await?;
        let bak_content = serde_json::from_str::<Self>(&bak_content)?;

        Ok(bak_content)
    }
}
