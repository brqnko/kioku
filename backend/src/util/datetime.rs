pub fn today_utc_midnight() -> Result<chrono::DateTime<chrono::Utc>, anyhow::Error> {
    Ok(chrono::Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| anyhow::anyhow!("failed to construct today utc midnight"))?
        .and_utc())
}
