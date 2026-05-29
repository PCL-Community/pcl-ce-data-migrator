use std::path::{Path, PathBuf};

use chrono::Local;

use crate::bak_data::BakData;

/// 管理备份文件的创建、应用、删除和列表查询
pub struct BackupManager {
    bak_dir: PathBuf,
    config_path: PathBuf,
}

impl BackupManager {
    pub fn new(bak_dir: PathBuf, config_path: PathBuf) -> Self {
        Self {
            bak_dir,
            config_path,
        }
    }

    /// 异步扫描备份目录，返回排序后的 .bak 文件路径列表
    pub async fn list_backups(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut entries = match tokio::fs::read_dir(&self.bak_dir).await {
            Ok(e) => e,
            Err(_) => return files,
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(meta) = entry.metadata().await
                && meta.is_file()
            {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "bak") {
                    files.push(path);
                }
            }
        }
        files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        files
    }

    /// 创建备份，返回 (文件名, 更新后的文件列表)
    pub async fn create_backup(&self) -> Result<(String, Vec<PathBuf>), String> {
        if !self.config_path.exists() {
            return Err("未找到 PCL CE 配置文件".into());
        }

        let bak_data = BakData::from_config_content(&self.config_path)
            .await
            .map_err(|e| format!("读取配置失败：{}", e))?;

        let bak_name = format!("ce-config-{}.bak", Local::now().format("%Y-%m-%d %H%M%S"));
        let bak_path = self.bak_dir.join(&bak_name);

        bak_data
            .save_to(&bak_path)
            .await
            .map_err(|e| format!("保存失败：{}", e))?;

        let files = self.list_backups().await;
        Ok((bak_name, files))
    }

    /// 应用指定备份到配置文件
    pub async fn apply_backup(&self, bak_path: &Path) -> Result<(), String> {
        if !self.config_path.exists() {
            return Err("未找到 PCL CE 配置文件".into());
        }

        let bak_data = BakData::create_from(bak_path)
            .await
            .map_err(|e| format!("读取备份失败：{}", e))?;

        bak_data
            .apply_config_content(&self.config_path)
            .await
            .map_err(|e| format!("应用失败：{}", e))?;

        Ok(())
    }

    /// 删除指定备份文件，返回更新后的文件列表
    pub async fn delete_backup(&self, bak_path: &Path) -> Result<Vec<PathBuf>, String> {
        tokio::fs::remove_file(bak_path)
            .await
            .map_err(|e| format!("删除失败：{}", e))?;

        Ok(self.list_backups().await)
    }
}
