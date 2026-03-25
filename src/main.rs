use std::{env, path::PathBuf};

use chrono::Local;
use dialoguer::{MultiSelect, Select};
use tokio::fs;

use crate::{bak_data::BakData, errors::AppError};

mod bak_data;
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
    println!("PCL CE 数据迁移工具 v1.0 by PCL Community");

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

                let bak_data = BakData::from_config_content(config_file).await?;
                println!("已完成数据读取");

                let bak_file_name = format!(
                    "ce-config-{0}.bak",
                    Local::now().format("%Y-%m-%d %H-%M-%S")
                );
                let mut bak_file_location = bak_dir.clone();
                bak_file_location.push(bak_file_name);

                bak_data.save_to(&bak_file_location).await?;
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
                let bak_content = BakData::create_from(selected_file).await?;
                bak_content.apply_config_content(&config_file).await?;

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
