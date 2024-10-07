use chrono::{Datelike, Duration, Local, NaiveDate};
use std::collections::HashMap;

/// step 1: login (admin/123)
/// step 2: 選定搜尋日期範圍
/// step 3: 從日期與時間下載檔案，還要翻頁

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // step 1: login, 必須使用cookie才能維持登入狀態
    let client = reqwest::Client::builder().cookie_store(true).build()?;
    let mut form = HashMap::new();
    form.insert("username", "admin");
    form.insert("password", "123");

    // login後必須使用cookie保存已登入資訊
    let _ = client
        .post("http://192.168.1.100/wsgi/login")
        .form(&form)
        .send()
        .await?;

    // step2: custom date interval, search.js
    // server=0&ch_username=*&stime=s-range&
    // year=2024&month=9&day=06&hour=00&min=00&sec=00
    // &yearend=2024&monthend=10&dayend=06&hourend=23&minend=59&secend=59
    // http://192.168.1.100/wsgi/search post

    let today = Local::now();

    // let mut form = HashMap::new();
    form.clear();
    form.insert("server", "0"); // 0, local machine
    form.insert("ch_username", "*"); // all channels
    form.insert("stime", "s-range");
    form.insert("year", &today.year().to_string());
    form.insert("month", &today.month().to_string());
    form.insert("day", "31");
    form.insert("hour", "00");
    form.insert("min", "00");
    form.insert("sec", "00");
    form.insert("yearend", &today.year().to_string());
    form.insert("monthend", &today.month().to_string());
    form.insert("dayend", "31");
    form.insert("hourend", "23");
    form.insert("minend", "59");
    form.insert("secend", "59");

    let response = client
        .post("http://192.168.1.100/wsgi/search")
        .form(&form)
        .send()
        .await?;

    // parse
    // 5,18399,2024-10-06 00:35:19,8,2024-10-06 00:35:27,i,,0,高調,,;
    // 6,18400,2024-10-06 17:38:10,5,2024-10-06 17:38:15,i,,0,北 調,,;
    // 2,16497,2024-01-31 09:02:14,22,2024-01-31 09:02:35,i,,0,64345,,;
    // 5,16498,2024-01-31 13:19:00,8,2024-01-31 13:19:08,i,,0, 高調,,;
    // 1,16499,2024-01-31 13:45:49,12,2024-01-31 13:46:01,i,27842,0,21042,,;
    // 每一列以;結尾，每一欄以,分開。
    // col: {0: channel, 1:file_id, 2:start datetime, 3:rec_length ,4L end datetime,
    // 5:call_direction, 6: call_phone_number, 7: 0, 8: phone_name, 9: ?, 10: asr_enable}
    let results = response.text().await?;
    println!("{results}");

    // 使用wsgi下載只要知道fileid即可，檔名會自動給出
    // http://192.168.1.100/wsgi/downloadrec?device_id=0&fileid=16499
    // http://192.168.1.100/audio_mnt/000/ch4/20240829/164503.mp3
    let mut rows: Vec<&str> = results.split(';').collect();

    // 最後一筆資料為; 刪除
    if let Some(last) = rows.last() {
        if last.len() <= 1 {
            rows.pop();
        }
    }

    for row in rows {
        println!("{row}");
        let col: Vec<&str> = row.split(',').collect();

        let url = format!(
            "http://192.168.1.100/wsgi/downloadrec?device_id=0&fileid={}",
            col[1]
        );
        let response = client.get(url).send().await?;

        // 檔名在response的header中
        // response {"server": "nginx/1.12.1", "date": "Sun, 06 Oct 2024 14:18:33 GMT",
        // "content-type": "attachment", "content-length": "44878", "connection": "keep-alive",
        // "last-modified": "Wed, 31 Jan 2024 01:02:36 GMT", "accept-ranges": "bytes",
        // "content-disposition": "attachment; filename=\"Ch2_2024-01-31_09-02-14_22_2024-01-31_09-02-35_.mp3\""}
        // "attachment; filename=\"Ch1_2024-01-31_13-45-49_12_2024-01-31_13-46-01_.mp3\""

        // println!("response {:?}", response.headers());
        let header_filename = response
            .headers()
            .get("content-disposition")
            .unwrap()
            .to_str()
            .unwrap();
        println!("{}", header_filename);
        // 使用split找到filename開頭的位置

        // 檔名中可能有utf8字串，用iter方式切子字串
        let mut filename = "";
        if let Some(part) = header_filename.split("filename=\"").nth(1) {
            if let Some(name) = part.split('"').next() {
                filename = name;
            }
        }

        // 確保資料夾存在，若不存在則建立
        let folder_path = r"c:/mp3";
        std::fs::create_dir_all(folder_path)?;
        let path = std::path::Path::new(folder_path).join(filename);
        let mut file = std::fs::File::create(path)?;
        let mut content = std::io::Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;

        println!("file:{} download complete.", filename);
    }
    Ok(())
}

fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    // 取得該月的第一天
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    // 取得下一個月的第一天，然後往前一天就是本月的最後一天
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    let last_day = next_month.unwrap() - Duration::days(1);

    last_day
}
