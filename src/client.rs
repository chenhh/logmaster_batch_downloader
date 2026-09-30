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

pub fn extract_filename(content_disposition: &str, fallback_file_id: &str) -> String {
    if let Some(part) = content_disposition.split("filename=\"").nth(1) {
        if let Some(name) = part.split('"').next() {
            if !name.trim().is_empty() {
                return sanitize_filename(name.trim());
            }
        }
    } else if let Some(part) = content_disposition.split("filename=").nth(1) {
        let name = part.split(';').next().unwrap_or("").trim().trim_matches('"');
        if !name.is_empty() {
            return sanitize_filename(name);
        }
    }
    format!("rec_{}.mp3", fallback_file_id)
}
