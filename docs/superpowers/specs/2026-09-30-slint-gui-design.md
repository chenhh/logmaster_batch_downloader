# Design Specification: Slint GUI for LogMaster Batch Downloader

- **Date**: 2026-09-30
- **Status**: Approved
- **Topic**: Slint GUI Interface for LogMaster Batch Audio Downloader

---

## 1. Overview & Goals

`logmaster_batch_downloader` is a tool for downloading audio recordings from a LogMaster device (default host `http://192.168.1.100`). It logs in, searches recordings within a specified year and month, and batch-downloads the audio files.

This specification details the addition of a native GUI frontend using the [Slint](https://slint.dev/) toolkit (`slint` 1.x) while refactoring the existing HTTP and parsing logic into a shared, robust core client shared by both CLI and GUI interfaces.

### Key Goals
- **Native Desktop GUI**: Built using Slint, supporting responsive, modern UI styling.
- **Shared Core Module**: Extract network communication, session cookie handling, search query construction, response parsing, and file downloading into a reusable `client` module.
- **Asynchronous Execution & Cancellation**: Run all network operations on background Tokio tasks with thread-safe UI updates and mid-operation cancellation support.
- **Graceful Error Handling**: Individual recording download failures will not abort the batch; errors will be logged, and the job will continue with a final summary.
- **Cross-Platform Directory Picking**: Default to user's standard Downloads folder with a native directory chooser (`rfd`).

---

## 2. Architecture & Module Organization

The project is structured into modular components:

```
.
├── Cargo.toml
├── build.rs                        # Compiles ui/logmaster_ui.slint via slint_build
├── ui/
│   ├── icons/
│   │   └── tpc.ico                 # App icon
│   └── logmaster_ui.slint          # Slint UI layout & component definitions
├── src/
│   ├── lib.rs                      # Exposes client, date_utils, models
│   ├── client.rs                   # LogMasterClient (HTTP, login, search, download)
│   ├── date_utils.rs               # Calendar & leap year calculations
│   ├── gui.rs                      # Slint GUI controller, background workers & bindings
│   └── main.rs                     # Entry point (dispatches to CLI or GUI)
└── tests/
    └── client_test.rs              # Unit & integration tests for client & date logic
```

### Module Responsibilities

1. **`date_utils`**:
   - `is_leap_year(year: u32) -> bool`
   - `last_day_of_month(year: u32, month: u32) -> Result<u32, DateError>`

2. **`client`**:
   - `RecordInfo`: Represents a parsed search record.
     - `channel: String`
     - `file_id: String`
     - `start_time: String`
     - `duration_sec: u32`
     - `end_time: String`
     - `phone_name: String`
   - `LogMasterClient`:
     - Holds `reqwest::Client` configured with `cookie_store(true)` and default request timeouts.
     - `new(base_url: impl Into<String>) -> Result<Self, ClientError>`
     - `login(&self, username: &str, password: &str) -> Result<(), ClientError>`: Posts credentials to `/wsgi/login`.
     - `search_month(&self, year: u32, month: u32) -> Result<Vec<RecordInfo>, ClientError>`: Posts query to `/wsgi/search` from day 1 00:00:00 to last day 23:59:59.
     - `download_record(&self, file_id: &str, output_dir: &Path) -> Result<PathBuf, ClientError>`: Downloads `/wsgi/downloadrec?device_id=0&fileid={file_id}`, parses `Content-Disposition` header for filename (falls back to `rec_{file_id}.mp3` if header missing), creates parent directories, and streams response body to disk.
   - Helper function:
     - `extract_filename(content_disposition: &str, fallback_file_id: &str) -> String`

3. **`gui`** (compiled under `feature = "gui"`):
   - Initializes `LogMasterUI::new()`.
   - Populates initial values:
     - Host: `http://192.168.1.100`
     - Username: `admin`, Password: `123`
     - Years: List of years from `2020` to `2030` (or `current_year - 4` to `current_year + 2`), default current year.
     - Months: `01` through `12`, default current month.
     - Default output directory: OS standard download folder joined with `{year}/{month}`.
   - Attaches event callbacks:
     - `browse-folder-clicked`: Invokes native file dialog (`rfd::FileDialog::pick_folder()`), updates path on selection.
     - `test-connection-clicked`: Spawns Tokio task to verify login and updates connection status indicator.
     - `start-download-clicked`: Spawns batch download worker, manages cancellation tokens, updates progress and log output.
     - `cancel-download-clicked`: Triggers cancellation token to interrupt loop cleanly.
     - `clear-log-clicked`: Resets log display.

4. **`main`**:
   - Under `feature = "gui"`: calls `gui::run()`.
   - Under `feature = "cli"`: parses CLI arguments (`year`, `month`, optional `host`, `outdir`) and runs command-line download loop.

---

## 3. Dependencies & Feature Flags

### `Cargo.toml`
```toml
[package]
name = "logmaster_batch_downloader"
version = "0.1.0"
edition = "2021"

[features]
default = ["gui"]
cli = ["dep:clap"]
gui = ["dep:slint", "dep:rfd", "dep:dirs"]

[dependencies]
reqwest = { version = "0.12.8", features = ["cookies"] }
tokio = { version = "1.40.0", features = ["full"] }
chrono = "0.4.38"
thiserror = "1.0.64"
clap = { version = "4.5.19", features = ["derive"], optional = true }
slint = { version = "1.8", optional = true }
rfd = { version = "0.15", optional = true }
dirs = { version = "5.0", optional = true }

[build-dependencies]
slint-build = { version = "1.8", optional = true }
```

When building with `--features gui` (the default), `build.rs` compiles `ui/logmaster_ui.slint`.

---

## 4. Slint UI Specification (`ui/logmaster_ui.slint`)

### Components and Layout

- **Window Title**: `LogMaster 錄音批次下載器`
- **Default Dimensions**: `680px` width, `600px` height.
- **Top Section (Connection Settings GroupBox)**:
  - Host Address: `LineEdit` (default `http://192.168.1.100`)
  - Username: `LineEdit` (default `admin`)
  - Password: `LineEdit` (input-type: password, default `123`)
  - "測試連線" (Test Connection) `Button`
  - Connection Status: `Text` with dynamic color (Gray: "未連線", Green: "連線成功", Red: "連線失敗: ...")
- **Middle Section (Parameters GroupBox)**:
  - Year: `ComboBox`
  - Month: `ComboBox`
  - Target Path: `LineEdit` displaying selected directory
  - "瀏覽..." `Button` triggering native directory picker
- **Control & Progress Section**:
  - "開始下載" (Start Download) `Button`: Primary accent color, enabled when `!is_downloading`.
  - "停止下載" (Cancel) `Button`: Enabled when `is_downloading`.
  - Progress Bar: `ProgressIndicator` with `progress: float` (0.0 to 1.0)
  - Progress Text: `Text` e.g., `"進度: 24/100 (24%) - 成功 23 筆，失敗 1 筆"`
  - Current Status: `Text` e.g., `"正在下載: Ch1_2024-10-06_13-45-49.mp3 ..."`
- **Bottom Section (Log View GroupBox)**:
  - Header with title and "清除日誌" `Button`.
  - `ScrollView` with multi-line text or item list detailing chronological actions with timestamps.

### Slint Properties & Callbacks
```slint
export component LogMasterUI inherits Window {
    in-out property <string> host-url: "http://192.168.1.100";
    in-out property <string> username: "admin";
    in-out property <string> password: "123";
    in-out property <string> status-text: "未連線";
    in-out property <color> status-color: #888888;
    in-out property <string> current-year: "2024";
    in-out property <string> current-month: "10";
    in-out property <[string]> years: ["2021", "2022", "2023", "2024", "2025", "2026"];
    in-out property <[string]> months: ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12"];
    in-out property <string> output-path: "";
    in-out property <bool> is-downloading: false;
    in-out property <float> download-progress: 0.0;
    in-out property <string> progress-label: "0 / 0 (0%)";
    in-out property <string> current-file-label: "";
    in-out property <string> log-content: "";

    callback test-connection();
    callback browse-folder();
    callback start-download();
    callback cancel-download();
    callback clear-log();
}
```

---

## 5. Async Execution, State Machine & Cancellation

### State Machine
- **Idle**:
  - `is-downloading = false`
  - Input fields enabled.
  - "開始下載" enabled; "停止下載" disabled.
- **Downloading**:
  - `is-downloading = true`
  - Input fields disabled.
  - "開始下載" disabled; "停止下載" enabled.
- **Cancelling**:
  - User clicked "停止下載".
  - Cancellation token flagged.
  - Current in-flight request finishes or aborts; batch loop terminates.
  - State transitions back to **Idle** with message `"使用者已取消下載作業"`.

### Thread-Safe Communication
All background operations execute on Tokio runtime threads.
Updates to the UI invoke:
```rust
let ui_weak = ui.as_weak();
// inside async task
let _ = ui_weak.upgrade_in_event_loop(move |ui| {
    ui.set_download_progress(ratio);
    ui.set_progress_label(slint::SharedString::from(label));
    // append log
});
```

---

## 6. Error Handling & Edge Cases

1. **Network Unreachable / Bad Credentials**:
   - `client.login()` captures error:
     - Sets connection status to Red with concise failure message (e.g. `"連線超時"` or `"密碼錯誤"`).
     - Appends error details to log view.
2. **Empty Search Results**:
   - If `/wsgi/search` yields zero records:
     - Appends log: `"未搜尋到符合條件之錄音紀錄 (共 0 筆)"`.
     - Resets state to Idle without error crash.
3. **Missing `Content-Disposition` Header**:
   - Fallback filename: `format!("record_{}_{}.mp3", year_month, file_id)`.
   - Log warning and proceed with download.
4. **Individual File Download Failure**:
   - Catches error per item in the loop.
   - Logs: `"[X/N] 下載失敗 (ID: {}): {}"`.
   - Increments `fail_count`, continues immediately to next file.
5. **Special Characters in Filename / UTF-8**:
   - Decodes filename using proper URI / UTF-8 safe parser.
   - Replaces any invalid OS path characters (`:`, `*`, `?`, `"`, `<`, `>`, `|`) with `_`.

---

## 7. Testing Strategy

1. **Unit Tests (`tests/`)**:
   - `test_is_leap_year`: Verifies regular years (2021, 2022, 2023), leap years (2020, 2024), century non-leap (1900, 2100), century leap (2000).
   - `test_last_day_of_month`: Verifies 28, 29, 30, 31 day boundaries.
   - `test_parse_search_results`: Verifies splitting semicolon-separated rows, comma-separated columns, handling trailing empty rows.
   - `test_extract_filename`: Verifies extraction from standard Content-Disposition, quotes handling, and fallback behavior.
2. **Compilation & Syntax Verification**:
   - `cargo check --features gui`
   - `cargo test`
   - `cargo build --features gui`
