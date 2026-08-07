pub fn current_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}
