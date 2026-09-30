use logmaster_batch_downloader::client::LogMasterClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_client_login_search_and_download() {
    let mock_server = MockServer::start().await;

    // 1. Mock login
    Mock::given(method("POST"))
        .and(path("/wsgi/login"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;

    // 2. Mock search
    let search_body = "5,18399,2024-10-06 00:35:19,8,2024-10-06 00:35:27,i,,0,高調,,;";
    Mock::given(method("POST"))
        .and(path("/wsgi/search"))
        .respond_with(ResponseTemplate::new(200).set_body_string(search_body))
        .mount(&mock_server)
        .await;

    // 3. Mock download
    let audio_bytes = b"ID3fake_mp3_content";
    Mock::given(method("GET"))
        .and(path("/wsgi/downloadrec"))
        .and(query_param("device_id", "0"))
        .and(query_param("fileid", "18399"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-disposition", "attachment; filename=\"test_rec.mp3\"")
                .set_body_bytes(audio_bytes.to_vec()),
        )
        .mount(&mock_server)
        .await;

    let client = LogMasterClient::new(mock_server.uri()).expect("Failed to create client");

    // Test login
    client.login("admin", "123").await.expect("Login failed");

    // Test search
    let records = client.search_month(2024, 10).await.expect("Search failed");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].file_id, "18399");

    // Test download
    let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
    let downloaded_path = client
        .download_record("18399", temp_dir.path())
        .await
        .expect("Download failed");

    assert!(downloaded_path.exists());
    assert_eq!(downloaded_path.file_name().unwrap(), "test_rec.mp3");
    let content = std::fs::read(&downloaded_path).expect("Failed to read file");
    assert_eq!(content, audio_bytes);
}
