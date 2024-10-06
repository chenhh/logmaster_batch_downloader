use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::io::copy;

/// step 1: login (admin/123)
/// step 2: 選定搜尋日期範圍
/// step 3: 從日期與時間下載檔案，還要翻頁


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // step 1: login, 必須使用cookie才能維持登入狀態
    let client = reqwest::Client::builder().cookie_store(true).build()?;
    let mut login_data = HashMap::new();
    login_data.insert("username", "admin");
    login_data.insert("password", "123");
    let response = client.post("http://192.168.1.100/wsgi/login")
        .form(&login_data)
        .send()
        .await?;


    // step2: custom date interval, search.js
    // server=0&ch_username=*&stime=s-range&year=2024&month=9&day=06&hour=00&min=00&sec=00&yearend=2024&monthend=10&dayend=06&hourend=23&minend=59&secend=59
    // http://192.168.1.100/wsgi/search post
    let mut form = HashMap::new();
    form.insert("server", "0"); // 0, local machine
    form.insert("ch_username", "*");    // all channels
    form.insert("stime", "s-range");
    form.insert("year", "2024");
    form.insert("month","1");
    form.insert("day", "31");
    form.insert("hour", "00");
    form.insert("min", "00");
    form.insert("sec", "00");
    form.insert("yearend", "2024");
    form.insert("monthend","1");
    form.insert("dayend", "31");
    form.insert("hourend", "23");
    form.insert("minend", "59");
    form.insert("secend", "59");

    let response = client.post("http://192.168.1.100/wsgi/search")
    .form(&form)
    .send()
    .await?;

    // parse, 要注意搜尋結果會有多頁
    // 5,18399,2024-10-06 00:35:19,8,2024-10-06 00:35:27,i,,0,高調,,;
    // 6,18400,2024-10-06 17:38:10,5,2024-10-06 17:38:15,i,,0,北 調,,;
    // 2,16497,2024-01-31 09:02:14,22,2024-01-31 09:02:35,i,,0,64345,,;
    // 5,16498,2024-01-31 13:19:00,8,2024-01-31 13:19:08,i,,0, 高調,,;
    // 1,16499,2024-01-31 13:45:49,12,2024-01-31 13:46:01,i,27842,0,21042,,;
    // each row split by semicolon ;
    // each col split by comma ,
    // col: {0: channel, 1:fileid, 2:start datetime, 3:rec_length ,4L end datetime,, 5:call_direction, 
    // 6: call_phone_number, 7: 0, 8: phone_name, 9: ?, 10: asr_enable}
    // objPlayList(arrRecord[7], (i + 1), arrRecord[1], arrRecord[0], arrRecord[8], arrRecord[9], arrRecord[2], arrRecord[4], arrRecord[3], arrRecord[5], arrRecord[6], arrRecord[10]);
    let results = response.text().await?;
    println!("{results}");

    // download
    // http://192.168.1.100/wsgi/downloadrec?device_id=0&fileid=16499
    // http://192.168.1.100/audio_mnt/000/ch4/20240829/164503.mp3

    // 指定輸出資料夾


    // let mut file = File::create("logined.html")?;
    // copy(&mut html_content, &mut file)?;
    // let product_selector = Selector::parse("article.product_pod").unwrap();
    // let products = document.select(&product_selector);

    // download
    // let mut file = File::create("image.png")?;
    // copy(&mut response, &mut file)?;

    Ok(())
}
