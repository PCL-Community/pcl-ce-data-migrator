use std::{env, path::PathBuf};

use chrono::Local;
use dialoguer::Select;
use serde_json::Value;
use tokio::fs;

use crate::{config_reader::BakData, errors::AppError};

mod config_reader;
mod errors;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    println!("PCL CE 数据迁移工具 v1.0 by tangge233");

    let base_dir = env::current_dir()?;
    if base_dir.read_dir()?.skip(2).any(|_| true) {
        println!("> 建议将程序放在空目录中运行，以避免不必要的文件覆盖行为");
    }

    let mut bak_dir = base_dir;
    bak_dir.push("baks");
    if !bak_dir.exists() {
        fs::create_dir(&bak_dir).await?;
    }

    let mut bak_files: Vec<PathBuf> = vec![];

    for bak_file in bak_dir.read_dir()? {
        let bak_file = bak_file?;
        let meta = bak_file.metadata()?;
        let file_name = bak_file.file_name();
        let file_name = file_name.to_string_lossy();
        if meta.is_file() && file_name.ends_with("bak") {
            bak_files.push(bak_file.path());
        }
    }

    println!("在当前目录下存储有 {0} 个备份文件", bak_files.len());

    let choice = Select::new()
        .with_prompt("选择操作")
        .item("创建新数据备份")
        .item("使用数据备份")
        .item("取消")
        .default(0)
        .interact()
        .unwrap();

    let mut config_file = env::home_dir().ok_or(AppError::EnvNotFound)?;
    config_file.push("AppData\\Roaming\\PCLCE\\config.v1.json");

    match choice {
        0 => {
            if !config_file.exists() {
                println!("没有找到配置文件，退出。");
            } else {
                println!("配置文件路径：{:?}", config_file);
                let content = std::fs::read_to_string(&config_file)?;
                let content: Value = serde_json::from_str(&content)?;

                let bak_data = BakData {
                    comp_favs: content
                        .get("CompFavorites")
                        .map(|x| x.as_str().unwrap().to_string()),
                };
                println!("已完成数据读取");

                let save_bak_content = serde_json::to_string(&bak_data)?;
                let bak_file_name =
                    format!("ce-config-{0}.bak", Local::now().format("%Y-%m-%d %H%M%S"));
                let mut bak_file_location = bak_dir.clone();
                bak_file_location.push(bak_file_name);
                std::fs::write(&bak_file_location, save_bak_content)?;
                print!("数据备份文件已保存到 {:?}", bak_file_location);
            }
        }
        1 => {
            println!("请选择需要使用的文件");
            for (i, item) in bak_files.iter().enumerate() {
                println!("[{i}] {:?}", item);
            }

            let files: Vec<&str> = bak_files.iter().map(|x| x.to_str().unwrap()).collect();
            let choice = Select::new()
                .with_prompt("选择操作")
                .item("取消")
                .items(&files)
                .default(0)
                .interact()
                .unwrap();

            if choice != 0 && choice <= bak_files.len() {
                let selected_file = &bak_files[choice - 1];
                println!("读取数据：{:?}", selected_file);
                let bak_content = std::fs::read_to_string(selected_file)?;

                let bak_content = serde_json::from_str::<BakData>(&bak_content)?;

                let cfg_data = std::fs::read_to_string(&config_file)?;
                let mut cfg_data: Value = serde_json::from_str(&cfg_data)?;

                if let Some(comp_fav) = bak_content.comp_favs {
                    let new_val = Value::String(comp_fav);
                    if let Some(comp_fav_item) = cfg_data.get_mut("CompFavorites") {
                        *comp_fav_item = new_val;
                    } else {
                        _ = cfg_data
                            .as_object_mut()
                            .unwrap()
                            .insert("CompFavorites".to_string(), new_val);
                    }
                }

                std::fs::write(config_file, serde_json::to_string(&cfg_data)?)?;
                println!("备份已应用");
            }
        }
        2 => {}
        _ => {
            println!("操作无效");
        }
    }

    Ok(())
}
