use time::OffsetDateTime;

pub fn get_utc_timestamp_string() -> String {
    OffsetDateTime::now_utc().to_string()
}
