# LogMaster Batch Downloader

LogMaster 錄音系統音檔批次下載工具，支援 Slint 桌面圖形化介面（GUI）與命令列模式（CLI）。

## 功能特點

- **Slint 桌面圖形化介面**：現代化且響應迅速的 GUI 操作介面。
- **背景非同步下載**：透過 Tokio 非同步任務處理網路 I/O，介面流暢不卡頓。
- **即時進度與日誌**：提供百分比進度條、當前處理檔案標籤以及滾動式執行日誌。
- **隨時中止/取消**：下載中隨時可點擊「停止下載」優雅中止任務。
- **單檔容錯機制**：單一錄音檔下載超時或失敗時自動記錄錯誤並繼續下載其餘檔案，結束時統計成功與失敗數量。
- **本機資料夾選擇**：支援透過原生對話框（`rfd`）自由挑選儲存目錄，預設為系統下載資料夾。

## 使用方式

### 1. 執行 GUI 介面（預設）

直接使用 `cargo run` 或指定 `gui` feature 啟動圖形介面：

```bash
cargo run
# 或
cargo run --features gui
```

### 2. 執行 CLI 命令列模式

關閉預設功能並啟用 `cli` feature，需指定 `--year` 與 `--month`：

```bash
cargo run --no-default-features --features cli -- --year 2024 --month 10
```

可選參數：
- `--host <URL>`：設定 LogMaster 主機網址（預設 `http://192.168.1.100`）
- `-o, --outdir <PATH>`：自訂音檔儲存目錄（預設為系統下載資料夾）

### 3. 建置發行版本 (Release Build)

```bash
cargo build --release --features gui
```

產生的執行檔位於 `target/release/logmaster_batch_downloader`。

## 參考文件

- [Slint UI 官方文件](https://docs.slint.dev/1.8.0/docs/slint/)
- [Slint Pad 在線編輯器](https://slintpad.com/)