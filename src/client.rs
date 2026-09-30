use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Date error: {0}")]
    Date(#[from] crate::date_utils::DateError),
    #[error("Login failed: {0}")]
    LoginFailed(String),
    #[error("Search failed: {0}")]
    SearchFailed(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordInfo {
    pub channel: String,
    pub file_id: String,
    pub start_time: String,
    pub duration_sec: u32,
    pub end_time: String,
    pub phone_name: String,
}

pub fn parse_search_records(raw_response: &str) -> Vec<RecordInfo> {
    let mut records = Vec::new();
    for row in raw_response.split(';') {
        let trimmed = row.trim();
        if trimmed.is_empty() {
            continue;
        }
        let cols: Vec<&str> = trimmed.split(',').collect();
        if cols.len() >= 5 {
            let channel = cols[0].trim().to_string();
            let file_id = cols[1].trim().to_string();
            let start_time = cols[2].trim().to_string();
            let duration_sec = cols[3].trim().parse::<u32>().unwrap_or(0);
            let end_time = cols[4].trim().to_string();
            let phone_name = if cols.len() > 8 {
                cols[8].trim().to_string()
            } else {
                String::new()
            };
            if !file_id.is_empty() {
                records.push(RecordInfo {
                    channel,
                    file_id,
                    start_time,
                    duration_sec,
                    end_time,
                    phone_name,
                });
            }
        }
    }
    records
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

pub fn extract_filename_info(content_disposition: &str, fallback_file_id: &str) -> (String, bool) {
    if let Some(part) = content_disposition.split("filename=\"").nth(1) {
        if let Some(name) = part.split('"').next() {
            if !name.trim().is_empty() {
                return (sanitize_filename(name.trim()), false);
            }
        }
    } else if let Some(part) = content_disposition.split("filename=").nth(1) {
        let name = part.split(';').next().unwrap_or("").trim().trim_matches('"');
        if !name.is_empty() {
            return (sanitize_filename(name), false);
        }
    }
    (format!("rec_{}.mp3", fallback_file_id), true)
}

pub fn extract_filename(content_disposition: &str, fallback_file_id: &str) -> String {
    extract_filename_info(content_disposition, fallback_file_id).0
}

#[derive(Clone, Debug)]
pub struct LogMasterClient {
    base_url: String,
    client: reqwest::Client,
}

impl LogMasterClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, ClientError> {
        let mut base = base_url.into();
        while base.ends_with('/') {
            base.pop();
        }
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()?;
        Ok(Self {
            base_url: base,
            client,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<(), ClientError> {
        let mut form = HashMap::new();
        form.insert("username", username);
        form.insert("password", password);

        let url = format!("{}/wsgi/login", self.base_url);
        let resp = self.client.post(&url).form(&form).send().await?;

        if !resp.status().is_success() {
            return Err(ClientError::LoginFailed(format!(
                "HTTP Status: {}",
                resp.status()
            )));
        }

        Ok(())
    }

    pub async fn search_month(&self, year: u32, month: u32) -> Result<Vec<RecordInfo>, ClientError> {
        let last_day = crate::date_utils::last_day_of_month(year, month)?;
        let year_str = year.to_string();
        let month_str = month.to_string();
        let last_day_str = last_day.to_string();

        let mut form = HashMap::new();
        form.insert("server", "0");
        form.insert("ch_username", "*");
        form.insert("stime", "s-range");
        form.insert("year", &year_str);
        form.insert("month", &month_str);
        form.insert("day", "1");
        form.insert("hour", "00");
        form.insert("min", "00");
        form.insert("sec", "00");
        form.insert("yearend", &year_str);
        form.insert("monthend", &month_str);
        form.insert("dayend", &last_day_str);
        form.insert("hourend", "23");
        form.insert("minend", "59");
        form.insert("secend", "59");

        let url = format!("{}/wsgi/search", self.base_url);
        let resp = self.client.post(&url).form(&form).send().await?;

        if !resp.status().is_success() {
            return Err(ClientError::SearchFailed(format!(
                "HTTP Status: {}",
                resp.status()
            )));
        }

        let body = resp.text().await?;
        Ok(parse_search_records(&body))
    }

    pub async fn download_record(
        &self,
        file_id: &str,
        target_dir: &Path,
    ) -> Result<PathBuf, ClientError> {
        let url = format!("{}/wsgi/downloadrec?device_id=0&fileid={}", self.base_url, file_id);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            return Err(ClientError::DownloadFailed(format!(
                "HTTP Status: {}",
                resp.status()
            )));
        }

        let cd_header = resp
            .headers()
            .get("content-disposition")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        let (filename, is_fallback) = extract_filename_info(cd_header, file_id);
        if is_fallback {
            eprintln!("[警告] 查無 Content-Disposition 標頭，使用後備檔名: {filename}");
        }

        tokio::fs::create_dir_all(target_dir).await?;
        let file_path = target_dir.join(&filename);

        let bytes = resp.bytes().await?;
        tokio::fs::write(&file_path, bytes).await?;

        Ok(file_path)
    }
}
