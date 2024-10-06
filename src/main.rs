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
    let client = reqwest::Client::builder().cookie_store(true).build()?;

    // send POST　request
    let mut login_data = HashMap::new();
    login_data.insert("username", "admin");
    login_data.insert("password", "123");
    let response = client.post("http://192.168.1.100/wsgi/login")
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

    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);
    println!("{html_content}");

    // step3: custom date interval, search.js
    // server=0&ch_username=*&stime=s-today&year=2024&month=10&day=06&hour=00&min=00&sec=00&yearend=2024&monthend=10&dayend=06&hourend=23&minend=59&secend=59
    //  server=0&ch_username=*&stime=s-range&year=2024&month=9&day=06&hour=00&min=00&sec=00&yearend=2024&monthend=10&dayend=06&hourend=23&minend=59&secend=59
    // http://192.168.1.100/wsgi/search post
    let mut form = HashMap::new();
    form.insert("server", "0");
    form.insert("ch_username", "*");
    form.insert("stime", "s-range");
    form.insert("year", "2024");
    form.insert("month","9");
    form.insert("day", "06");
    form.insert("hour", "00");
    form.insert("min", "00");
    form.insert("sec", "00");
    form.insert("yearend", "2024");
    form.insert("monthend","10");
    form.insert("dayend", "06");
    form.insert("hourend", "23");
    form.insert("minend", "59");
    form.insert("secend", "59");

    let response = client.post("http://192.168.1.100/wsgi/search")
    .form(&form)
    .send()
    .await?;


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
