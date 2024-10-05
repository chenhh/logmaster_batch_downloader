use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::io::copy;

/// step 1: login (admin/123)
/// step 2: 選定搜尋日期範圍
/// step 3: 從日期與時間下載檔案，還要翻頁


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // step 1: login
    let url = "http://192.168.1.100/wsgi/login";
    let client = Client::new();

    // send POST　request
    let mut login_data = HashMap::new();
    login_data.insert("username", "admin");
    login_data.insert("password", "123");
    let response = client.post(url)
        .form(&login_data)
        .send()
        .await?;

    // step2 : frontpage -> search
    // note: 登入後，是否可以直接指定date interval後，直接搜尋結果，
    // 而不必先進到search.html後再搜尋?
    let response = client
        .get("http://192.168.1.100/search.html")
        .send()
        .await?;

    // step3: custom date interval
    let mut interval_data = HashMap::new();
    // form_inline
    //  stime: s-range
    // div-time-range
    // year, month, day, hour, min, sec
    // yearend, monthend, hourend, minend, hoursend, secend

    // parse, 要注意搜尋結果會有多頁
    // 主要分析audio channel no., channel name, start-end time interval
    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);
    println!("{html_content}");

    // download
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
