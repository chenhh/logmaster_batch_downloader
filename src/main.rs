use clap::Parser;
use reqwest::header::HeaderValue;
use std::collections::HashMap;

/// step 1: login (admin/123)
/// step 2: 選定搜尋日期範圍
/// step 3: 從日期與時間下載檔案，還要翻頁

/// Simple program to greet a person
///
#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[command(version="0.1", about = "備分調度電話記錄，必須指定年分--year與月份--month", long_about = None
)]
struct Args {
    #[arg(short, long, value_parser=clap::value_parser!(u32).range(2020..=2040))]
    year: u32,

    #[arg(short, long, value_parser=clap::value_parser!(u32).range(1..=12))]
    month: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "cli")]
    {
        let args = Args::parse();

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

        let year = args.year;
        let month = args.month;
        let last_day_str = last_day_of_month(year, month).to_string();
        let year_str = year.to_string();
        let month_str = month.to_string();
        println!(
            "抓取資料日期區間: {year}-{month}-01-00:00:00_{year}-{month}-{last_day_str}-23:59:59"
        );

        form.clear();
        form.insert("server", "0"); // 0, local machine
        form.insert("ch_username", "*"); // all channels
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
        // println!("{results}");

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

        for (rdx, row) in rows.iter().enumerate() {
            // println!("{row}");
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

            let empty_header = HeaderValue::from_str("").unwrap();
            // println!("response {:?}", response.headers());
            let header_filename = &response
                .headers()
                .get("content-disposition")
                .unwrap_or(&empty_header)
                .to_str()
                .unwrap();
            // println!("{}", header_filename);
            // 使用split找到filename開頭的位置

            if header_filename.len() == 0 {
                eprintln!(
                    "[{}/{}] 無法讀取資料表頭:{:?}",
                    rdx + 1,
                    rows.len(),
                    response.headers()
                );
                continue;
            }

            // 檔名中可能有utf8字串，用iter方式切子字串
            let mut filename = String::new();
            if let Some(part) = header_filename.split("filename=\"").nth(1) {
                if let Some(name) = part.split('"').next() {
                    filename = name.to_string();
                }
            }

            // 確保資料夾存在，若不存在則建立
            let folder_path: std::path::PathBuf = [
                "c:\\",
                "Users",
                "nanpu",
                "Downloads",
                &year.to_string(),
                &month.to_string(),
            ]
            .iter()
            .collect();
            // let folder_path = format!("C:\\Users\\nanpu\\Downloads\\{year}\\{month}");
            std::fs::create_dir_all(&folder_path)?;
            let path = std::path::Path::new(&folder_path).join(&filename);
            let mut file = std::fs::File::create(path)?;

            let mut content = std::io::Cursor::new(response.bytes().await?);
            std::io::copy(&mut content, &mut file)?;

            println!("[{}/{}]:{} 下載完成.", rdx + 1, rows.len(), &filename);
        }
        Ok(())
    }
    #[cfg(feature = "gui")]
    {
        println!("compile gui");
        Ok(())
    }
}

fn is_leap_year(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn last_day_of_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => panic!("invalid month: {month}"),
    }
}
