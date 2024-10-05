use reqwest::Client;
use scraper::{Html, Selector};

/// step 1: login (admin/123)
/// step 2: 選定搜尋日期範圍
/// step 3: 從日期與時間下載檔案，還要翻頁


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.dongs.xyz/";
    let client = Client::new();
    // send GET request
    let response = client
        .get(url)
        .send()
        .await?;

    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);
    println!("{html_content}");
    // let product_selector = Selector::parse("article.product_pod").unwrap();
    // let products = document.select(&product_selector);
    Ok(())
}
