#[cfg(feature = "cli")]
use clap::Parser;
use std::error::Error;

#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[command(
    version = "0.1",
    about = "備分調度電話記錄，必須指定年分--year與月份--month"
)]
struct Args {
    #[arg(short, long, value_parser=clap::value_parser!(u32).range(2020..=2040))]
    year: u32,

    #[arg(short, long, value_parser=clap::value_parser!(u32).range(1..=12))]
    month: u32,

    #[arg(long, default_value = "http://192.168.1.100")]
    host: String,

    #[arg(short, long)]
    outdir: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "gui")]
    {
        logmaster_batch_downloader::gui::run()?;
        Ok(())
    }

    #[cfg(all(feature = "cli", not(feature = "gui")))]
    {
        let args = Args::parse();
        let client = logmaster_batch_downloader::client::LogMasterClient::new(&args.host)?;
        client.login("admin", "123").await?;
        println!("登入成功，開始搜尋 {}-{} 錄音資料...", args.year, args.month);
        let records = client.search_month(args.year, args.month).await?;
        println!("搜尋完成，共 {} 筆資料", records.len());

        let base_dir = if let Some(custom) = args.outdir {
            std::path::PathBuf::from(custom)
        } else {
            dirs::download_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
        };
        let target_dir = base_dir
            .join(args.year.to_string())
            .join(format!("{:02}", args.month));

        for (idx, record) in records.iter().enumerate() {
            match client.download_record(&record.file_id, &target_dir).await {
                Ok(path) => {
                    println!("[{}/{}]: {} 下載完成", idx + 1, records.len(), path.display());
                }
                Err(e) => {
                    eprintln!(
                        "[{}/{}]: 下載失敗 (ID: {}): {}",
                        idx + 1,
                        records.len(),
                        record.file_id,
                        e
                    );
                }
            }
        }
        Ok(())
    }

    #[cfg(not(any(feature = "gui", feature = "cli")))]
    {
        eprintln!("請指定至少一個 feature: --features gui 或 --features cli");
        Ok(())
    }
}
