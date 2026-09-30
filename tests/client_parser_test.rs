use logmaster_batch_downloader::client::{parse_search_records, extract_filename, sanitize_filename};

#[test]
fn test_parse_search_records() {
    let raw = "5,18399,2024-10-06 00:35:19,8,2024-10-06 00:35:27,i,,0,高調,,;6,18400,2024-10-06 17:38:10,5,2024-10-06 17:38:15,i,,0,北 調,,;";
    let records = parse_search_records(raw);
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].channel, "5");
    assert_eq!(records[0].file_id, "18399");
    assert_eq!(records[0].start_time, "2024-10-06 00:35:19");
    assert_eq!(records[0].duration_sec, 8);
    assert_eq!(records[0].phone_name, "高調");
    assert_eq!(records[1].file_id, "18400");
}

#[test]
fn test_extract_filename_standard() {
    let header = "attachment; filename=\"Ch2_2024-01-31_09-02-14_22_2024-01-31_09-02-35_.mp3\"";
    assert_eq!(extract_filename(header, "16497"), "Ch2_2024-01-31_09-02-14_22_2024-01-31_09-02-35_.mp3");
}

#[test]
fn test_extract_filename_fallback_and_sanitization() {
    let empty_header = "";
    assert_eq!(extract_filename(empty_header, "999"), "rec_999.mp3");

    let dirty_name = "Ch1:test*file?.mp3";
    assert_eq!(sanitize_filename(dirty_name), "Ch1_test_file_.mp3");
}
