use reqwest::Client;
use scraper::{Html, Selector};
use std::io::copy;

/// step 1: login (admin/123)
/// step 2: 選定搜尋日期範圍
/// step 3: 從日期與時間下載檔案，還要翻頁
///  https://juejin.cn/post/7226177081197068346


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.dongs.xyz/";
    let client = Client::new();
    // send GET request
    let response = client
        .get(url)
        .send()
        .await?;

    // send POST　request
    let mut data = HashMap::new();

    data.insert("name", "admin");
    data.insert("password", "123");
    let response = client.post("https://localhost/post")
        .form(&data)
        .send()
        .await?;

    // parse
    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);
    println!("{html_content}");
    // let product_selector = Selector::parse("article.product_pod").unwrap();
    // let products = document.select(&product_selector);

    // download
    let mut file = File::create("image.png")?;
    // copy(&mut response, &mut file)?;


    Ok(())
}
