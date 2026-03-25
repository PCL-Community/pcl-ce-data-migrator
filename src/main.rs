use std::{env, path::PathBuf};

use chrono::Local;
use dialoguer::{MultiSelect, Select};
use serde_json::Value;
use tokio::fs;

use crate::{config_reader::BakData, errors::AppError};

mod config_reader;
mod errors;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = run().await {
        eprintln!("错误：{}", e);
        std::process::exit(1);
    }
    Ok(())
}

async fn run() -> Result<(), AppError> {
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
        .item("删除备份")
        .item("取消")
        .default(0)
        .interact()
        .unwrap();

    let mut data_dir = env::home_dir().ok_or(AppError::EnvNotFound)?;
    data_dir.push("AppData\\Roaming\\PCLCE");
    if !data_dir.exists() {
        tokio::fs::create_dir(&data_dir).await?;
    }

    let mut config_file = data_dir.clone();
    config_file.push("config.v1.json");

    match choice {
        0 => {
            if !config_file.exists() {
                println!("没有找到配置文件，退出。");
            } else {
                println!("配置文件路径：{:?}", config_file);
                let content = tokio::fs::read_to_string(&config_file).await?;
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
                tokio::fs::write(&bak_file_location, save_bak_content).await?;
                print!("数据备份文件已保存到 {:?}", bak_file_location);
            }
        }
        1 => {
            println!("请选择需要使用的文件");

            let show_files: Vec<&str> = bak_files.iter().map(|x| x.to_str().unwrap()).collect();
            let choice = Select::new()
                .with_prompt("选择文件")
                .item("取消")
                .items(&show_files)
                .default(0)
                .interact()
                .unwrap();

            if choice != 0 && choice <= bak_files.len() {
                let selected_file = &bak_files[choice - 1];
                println!("读取数据：{:?}", selected_file);
                let bak_content = tokio::fs::read_to_string(selected_file).await?;

                let bak_content = serde_json::from_str::<BakData>(&bak_content)?;

                let cfg_data = tokio::fs::read_to_string(&config_file).await?;
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

                tokio::fs::write(config_file, serde_json::to_string(&cfg_data)?).await?;
                println!("备份已应用");
            }
        }
        2 => {
            let show_files: Vec<&str> = bak_files.iter().map(|x| x.to_str().unwrap()).collect();

            let choice = MultiSelect::new()
                .with_prompt("选择文件")
                .items(&show_files)
                .with_prompt("使用上下键移动光标，空格选择文件，回车确认选择")
                .interact()
                .unwrap();

            for i in choice.iter().rev() {
                fs::remove_file(&bak_files[i.to_owned()]).await?;
                bak_files.remove(i.to_owned());
            }

            println!("已移除 {0} 个文件", choice.len());
        }
        3 => {}
        _ => {
            println!("操作无效");
        }
    }

    Ok(())
}
