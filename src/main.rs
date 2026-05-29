use std::{env, path::PathBuf, rc::Rc, sync::Arc};

use slint::{ModelRc, StandardListViewItem, VecModel};
use tokio::runtime::Runtime;

use crate::backup_manager::BackupManager;

mod backup_manager;
mod bak_data;

slint::include_modules!();

fn make_backup_model(files: &[PathBuf]) -> ModelRc<StandardListViewItem> {
    let model = Rc::new(VecModel::<StandardListViewItem>::default());
    let items: Vec<StandardListViewItem> = files
        .iter()
        .map(|p| {
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            StandardListViewItem::from(name.as_str())
        })
        .collect();
    model.set_vec(items);
    model.clone().into()
}

fn refresh_ui(ui: &AppWindow, files: &[PathBuf], status: &str) {
    ui.set_backup_list(make_backup_model(files));
    ui.set_backup_count(files.len() as i32);
    ui.set_status_text(status.into());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;

    let base_dir = env::current_dir()?;
    let bak_dir = base_dir.join("baks");
    std::fs::create_dir_all(&bak_dir)?;

    let home_dir = env::home_dir().expect("无法获取用户主目录");
    let config_path = home_dir.join("AppData\\Roaming\\PCLCE\\config.v1.json");

    let manager = Arc::new(BackupManager::new(bak_dir, config_path));
    let rt = Arc::new(Runtime::new()?);

    let files = rt.block_on(async { manager.list_backups().await });
    refresh_ui(&ui, &files, "就绪");

    // ── 删除确认（弹窗内点击"确认删除"后执行）─────────────────
    let weak = ui.as_weak();
    let rt_cb = rt.clone();
    let mgr_cb = manager.clone();

    ui.on_delete_confirmed(move || {
        let weak = weak.clone();
        let rt = rt_cb.clone();
        let mgr = mgr_cb.clone();

        let selected: i32 = weak
            .upgrade()
            .map(|ui| ui.get_selected_index())
            .unwrap_or(-1);

        if selected < 0 {
            weak.upgrade_in_event_loop(move |ui| {
                ui.set_status_text("请选择一个备份文件".into());
            })
            .ok();
            return;
        }

        rt.spawn(async move {
            let files = mgr.list_backups().await;
            if selected as usize >= files.len() {
                weak.upgrade_in_event_loop(move |ui| {
                    ui.set_status_text("选中的文件已不存在".into());
                })
                .ok();
                return;
            }

            let path = files[selected as usize].clone();
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            match mgr.delete_backup(&path).await {
                Ok(updated) => {
                    weak.upgrade_in_event_loop(move |ui| {
                        refresh_ui(&ui, &updated, &format!("已删除：{}", file_name));
                    })
                    .ok();
                }
                Err(msg) => {
                    weak.upgrade_in_event_loop(move |ui| {
                        ui.set_status_text(msg.into());
                    })
                    .ok();
                }
            }
        });
    });

    // ── 显示删除确认弹窗 ──────────────────────────────────────
    let weak = ui.as_weak();

    ui.on_delete_backup(move || {
        let selected: i32 = weak
            .upgrade()
            .map(|ui| ui.get_selected_index())
            .unwrap_or(-1);

        if selected < 0 {
            weak.upgrade_in_event_loop(move |ui| {
                ui.set_status_text("请选择一个备份文件".into());
            })
            .ok();
            return;
        }

        // 通过 Slint 回调触发 PopupWindow.show()
        weak.upgrade_in_event_loop(move |ui| {
            ui.invoke_show_delete_popup();
        })
        .ok();
    });

    // ── 创建备份 ──────────────────────────────────────────────
    let weak = ui.as_weak();
    let rt_cb = rt.clone();
    let mgr_cb = manager.clone();

    ui.on_create_backup(move || {
        let weak = weak.clone();
        let rt = rt_cb.clone();
        let mgr = mgr_cb.clone();

        rt.spawn(async move {
            match mgr.create_backup().await {
                Ok((name, files)) => {
                    weak.upgrade_in_event_loop(move |ui| {
                        refresh_ui(&ui, &files, &format!("备份已保存：{}", name));
                    })
                    .ok();
                }
                Err(msg) => {
                    weak.upgrade_in_event_loop(move |ui| {
                        ui.set_status_text(msg.into());
                    })
                    .ok();
                }
            }
        });
    });

    // ── 应用备份 ──────────────────────────────────────────────
    let weak = ui.as_weak();
    let rt_cb = rt.clone();
    let mgr_cb = manager.clone();

    ui.on_apply_backup(move || {
        let weak = weak.clone();
        let rt = rt_cb.clone();
        let mgr = mgr_cb.clone();

        let selected: i32 = weak
            .upgrade()
            .map(|ui| ui.get_selected_index())
            .unwrap_or(-1);

        if selected < 0 {
            weak.upgrade_in_event_loop(move |ui| {
                ui.set_status_text("请选择一个备份文件".into());
            })
            .ok();
            return;
        }

        rt.spawn(async move {
            let files = mgr.list_backups().await;
            if selected as usize >= files.len() {
                weak.upgrade_in_event_loop(move |ui| {
                    ui.set_status_text("选中的文件已不存在".into());
                })
                .ok();
                return;
            }

            let path = files[selected as usize].clone();
            match mgr.apply_backup(&path).await {
                Ok(()) => {
                    weak.upgrade_in_event_loop(move |ui| {
                        ui.set_status_text("配置已恢复".into());
                    })
                    .ok();
                }
                Err(msg) => {
                    weak.upgrade_in_event_loop(move |ui| {
                        ui.set_status_text(msg.into());
                    })
                    .ok();
                }
            }
        });
    });

    ui.run()?;
    Ok(())
}
