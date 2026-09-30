use crate::client::LogMasterClient;
use slint::ComponentHandle;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

slint::include_modules!();

fn append_log(ui: &LogMasterUI, message: &str) {
    let now = chrono::Local::now().format("%H:%M:%S");
    let current_log = ui.get_log_content();
    let new_line = format!("[{}] {}\n", now, message);
    let mut updated = current_log.to_string();
    updated.push_str(&new_line);
    ui.set_log_content(slint::SharedString::from(updated));
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let ui = LogMasterUI::new()?;

    // Populate Year and Month ComboBox models
    let current_date = chrono::Local::now();
    let cur_year = current_date.format("%Y").to_string();
    let cur_month = current_date.format("%m").to_string();

    let years: Vec<slint::SharedString> =
        (2020..=2030).map(|y| slint::SharedString::from(y.to_string())).collect();
    let months: Vec<slint::SharedString> =
        (1..=12).map(|m| slint::SharedString::from(format!("{:02}", m))).collect();

    ui.set_years(slint::ModelRc::new(slint::VecModel::from(years)));
    ui.set_months(slint::ModelRc::new(slint::VecModel::from(months)));
    ui.set_current_year(cur_year.into());
    ui.set_current_month(cur_month.into());

    let default_dir = dirs::download_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .to_string_lossy()
        .to_string();
    ui.set_output_path(default_dir.into());

    append_log(&ui, "LogMaster 批次下載器已就緒");

    // Shared cancellation flag
    let cancel_flag = Arc::new(AtomicBool::new(false));

    // Callback: browse-folder
    {
        let ui_weak = ui.as_weak();
        ui.on_browse_folder(move || {
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                let folder_str = folder.to_string_lossy().to_string();
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_output_path(folder_str.into());
                }
            }
        });
    }

    // Callback: clear-log
    {
        let ui_weak = ui.as_weak();
        ui.on_clear_log(move || {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_log_content("".into());
            }
        });
    }

    // Callback: test-connection
    {
        let ui_weak = ui.as_weak();
        ui.on_test_connection(move || {
            let ui = match ui_weak.upgrade() {
                Some(u) => u,
                None => return,
            };

            let host = ui.get_host_url().to_string();
            let username = ui.get_username().to_string();
            let password = ui.get_password().to_string();

            ui.set_status_text("測試連線中...".into());
            ui.set_status_color(slint::Color::from_rgb_u8(100, 100, 100));

            let ui_weak_async = ui_weak.clone();
            tokio::spawn(async move {
                match LogMasterClient::new(&host) {
                    Ok(client) => match client.login(&username, &password).await {
                        Ok(_) => {
                            let _ = ui_weak_async.upgrade_in_event_loop(|ui| {
                                ui.set_status_text("連線成功".into());
                                ui.set_status_color(slint::Color::from_rgb_u8(40, 167, 69));
                                append_log(&ui, "測試連線成功");
                            });
                        }
                        Err(e) => {
                            let err_msg = format!("連線失敗: {e}");
                            let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                                ui.set_status_text("連線失敗".into());
                                ui.set_status_color(slint::Color::from_rgb_u8(220, 53, 69));
                                append_log(&ui, &err_msg);
                            });
                        }
                    },
                    Err(e) => {
                        let err_msg = format!("主機網址格式錯誤: {e}");
                        let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                            ui.set_status_text("網址錯誤".into());
                            ui.set_status_color(slint::Color::from_rgb_u8(220, 53, 69));
                            append_log(&ui, &err_msg);
                        });
                    }
                }
            });
        });
    }

    // Callback: cancel-download
    {
        let cancel_flag = cancel_flag.clone();
        let ui_weak = ui.as_weak();
        ui.on_cancel_download(move || {
            cancel_flag.store(true, Ordering::SeqCst);
            if let Some(ui) = ui_weak.upgrade() {
                append_log(&ui, "收到取消請求，正在停止下載作業...");
                ui.set_current_file_label("正在取消中...".into());
            }
        });
    }

    // Callback: start-download
    {
        let cancel_flag = cancel_flag.clone();
        let ui_weak = ui.as_weak();
        ui.on_start_download(move || {
            let ui = match ui_weak.upgrade() {
                Some(u) => u,
                None => return,
            };

            let host = ui.get_host_url().to_string();
            let username = ui.get_username().to_string();
            let password = ui.get_password().to_string();
            let year_str = ui.get_current_year().to_string();
            let month_str = ui.get_current_month().to_string();
            let output_path_str = ui.get_output_path().to_string();

            cancel_flag.store(false, Ordering::SeqCst);
            let active_cancel = cancel_flag.clone();

            ui.set_is_downloading(true);
            ui.set_download_progress(0.0);
            ui.set_progress_label("準備中...".into());
            ui.set_current_file_label("正在連線中...".into());
            append_log(&ui, &format!("開始下載任務: {year_str} 年 {month_str} 月"));

            let ui_weak_async = ui_weak.clone();
            tokio::spawn(async move {
                let year: u32 = match year_str.parse() {
                    Ok(y) => y,
                    Err(_) => {
                        let _ = ui_weak_async.upgrade_in_event_loop(|ui| {
                            append_log(&ui, "無效的年份");
                            ui.set_is_downloading(false);
                        });
                        return;
                    }
                };
                let month: u32 = match month_str.parse() {
                    Ok(m) => m,
                    Err(_) => {
                        let _ = ui_weak_async.upgrade_in_event_loop(|ui| {
                            append_log(&ui, "無效的月份");
                            ui.set_is_downloading(false);
                        });
                        return;
                    }
                };

                let client = match LogMasterClient::new(&host) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                            append_log(&ui, &format!("建立連線失敗: {e}"));
                            ui.set_is_downloading(false);
                        });
                        return;
                    }
                };

                // 1. Login
                if let Err(e) = client.login(&username, &password).await {
                    let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                        append_log(&ui, &format!("登入失敗: {e}"));
                        ui.set_status_text("登入失敗".into());
                        ui.set_status_color(slint::Color::from_rgb_u8(220, 53, 69));
                        ui.set_is_downloading(false);
                    });
                    return;
                }

                let _ = ui_weak_async.upgrade_in_event_loop(|ui| {
                    ui.set_status_text("已連線".into());
                    ui.set_status_color(slint::Color::from_rgb_u8(40, 167, 69));
                    append_log(&ui, "登入成功，開始搜尋錄音清單...");
                });

                // 2. Search
                let records = match client.search_month(year, month).await {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                            append_log(&ui, &format!("搜尋失敗: {e}"));
                            ui.set_is_downloading(false);
                        });
                        return;
                    }
                };

                let total = records.len();
                if total == 0 {
                    let _ = ui_weak_async.upgrade_in_event_loop(|ui| {
                        append_log(&ui, "未搜尋到符合條件之錄音紀錄 (共 0 筆)");
                        ui.set_progress_label("0 / 0 (100%)".into());
                        ui.set_download_progress(1.0);
                        ui.set_current_file_label("完成 (無資料)".into());
                        ui.set_is_downloading(false);
                    });
                    return;
                }

                let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                    append_log(&ui, &format!("搜尋完成，共找到 {total} 筆錄音檔案"));
                });

                // Target directory: base / year / month
                let target_dir = PathBuf::from(&output_path_str)
                    .join(year.to_string())
                    .join(format!("{:02}", month));

                let mut success_count = 0;
                let mut fail_count = 0;

                for (idx, record) in records.iter().enumerate() {
                    if active_cancel.load(Ordering::SeqCst) {
                        let _ = ui_weak_async.upgrade_in_event_loop(|ui| {
                            append_log(&ui, "使用者已取消下載作業");
                            ui.set_current_file_label("已取消".into());
                        });
                        break;
                    }

                    let current_num = idx + 1;
                    let file_id = record.file_id.clone();
                    let prompt_text =
                        format!("正在下載 [{current_num}/{total}] (ID: {file_id}) ...");

                    let _ = ui_weak_async.upgrade_in_event_loop({
                        let prompt_text = prompt_text.clone();
                        move |ui| {
                            ui.set_current_file_label(prompt_text.into());
                        }
                    });

                    match client.download_record(&file_id, &target_dir).await {
                        Ok(saved_path) => {
                            success_count += 1;
                            let filename = saved_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&file_id)
                                .to_string();
                            let is_fallback = filename.starts_with("rec_");
                            let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                                if is_fallback {
                                    append_log(
                                        &ui,
                                        &format!("[警告] [{current_num}/{total}] 查無檔名標頭，使用預設檔名: {filename}"),
                                    );
                                }
                                append_log(&ui, &format!("[{current_num}/{total}] {filename} 下載完成"));
                            });
                        }
                        Err(e) => {
                            fail_count += 1;
                            let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                                append_log(
                                    &ui,
                                    &format!("[{current_num}/{total}] 下載失敗 (ID: {file_id}): {e}"),
                                );
                            });
                        }
                    }

                    let progress_ratio = current_num as f32 / total as f32;
                    let label = format!(
                        "{current_num} / {total} ({}%)",
                        (progress_ratio * 100.0) as u32
                    );
                    let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                        ui.set_download_progress(progress_ratio);
                        ui.set_progress_label(label.into());
                    });
                }

                let summary = format!(
                    "批次作業結束：共 {total} 筆，成功 {success_count} 筆，失敗 {fail_count} 筆"
                );
                let _ = ui_weak_async.upgrade_in_event_loop(move |ui| {
                    append_log(&ui, &summary);
                    ui.set_current_file_label("下載作業結束".into());
                    ui.set_is_downloading(false);
                });
            });
        });
    }

    ui.run()?;
    Ok(())
}
