use anyhow::anyhow;
use chrono::Duration;
use sqlx::postgres::types::PgInterval;

pub fn str_to_pginterval(interval_str: &str) -> anyhow::Result<Option<PgInterval>> {
    // Check if the input string is at least 2 characters long
    if interval_str.len() < 2 {
        return Ok(None);
    }

    // Split the string into the numeric part and the unit part
    let (value_str, unit) = interval_str.split_at(interval_str.len() - 1);

    // Parse the numeric part
    let value: i64 = value_str.parse()?;

    // Determine the duration based on the unit
    let duration = match unit {
        "s" => Duration::seconds(value),
        "m" => Duration::minutes(value),
        "h" => Duration::hours(value),
        "d" => Duration::days(value),
        _ => return Err(anyhow!("Invalid time unit")),
    };

    let pg_interval = PgInterval {
        months: 0,
        days: duration.num_days() as i32,
        microseconds: (duration.num_seconds() % 86400) * 1_000_000,
    };

    Ok(Some(pg_interval))
}

pub fn capitalize(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(index, char)| {
            if index == 0 {
                char.to_ascii_uppercase()
            } else {
                char.to_ascii_lowercase()
            }
        })
        .collect::<String>()
}
