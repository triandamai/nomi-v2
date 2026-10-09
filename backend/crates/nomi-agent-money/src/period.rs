//! A person's money month: from their chosen start day (payday on the 25th, say) to the day
//! before it next month, in their timezone. A start day of 1 is the calendar month. A period is
//! named by the month it starts in ("2026-10" from 25 October to 24 November).

use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    /// The month it starts in.
    pub year: i32,
    pub month: u32,
    pub start_day: u32,
    /// First instant, and the first instant of the next period.
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Period {
    /// `YYYY-MM` of the month it starts in.
    pub fn key(&self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }

    pub fn first_day(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, self.start_day).unwrap_or_default()
    }

    /// The last day it covers.
    pub fn last_day(&self) -> NaiveDate {
        let (year, month) = shift_month(self.year, self.month, 1);
        NaiveDate::from_ymd_opt(year, month, self.start_day).unwrap_or_default().pred_opt().unwrap_or_default()
    }

    pub fn shifted(&self, tz: chrono_tz::Tz, by: i32) -> Period {
        let (year, month) = shift_month(self.year, self.month, by);
        Period::starting_in(tz, self.start_day, year, month)
    }

    /// The period that starts in `year`-`month`.
    pub fn starting_in(tz: chrono_tz::Tz, start_day: u32, year: i32, month: u32) -> Period {
        let start_day = start_day.clamp(1, 28);
        let (next_year, next_month) = shift_month(year, month, 1);
        Period { year, month, start_day, start: local_midnight(tz, year, month, start_day), end: local_midnight(tz, next_year, next_month, start_day) }
    }

    /// The period `at` falls in.
    pub fn containing(tz: chrono_tz::Tz, start_day: u32, at: DateTime<Utc>) -> Period {
        let local = at.with_timezone(&tz);
        let start_day = start_day.clamp(1, 28);
        let (year, month) = if local.day() >= start_day { (local.year(), local.month()) } else { shift_month(local.year(), local.month(), -1) };
        Period::starting_in(tz, start_day, year, month)
    }
}

pub fn shift_month(year: i32, month: u32, by: i32) -> (i32, u32) {
    let index = year * 12 + month as i32 - 1 + by;
    (index.div_euclid(12), (index.rem_euclid(12) + 1) as u32)
}

fn local_midnight(tz: chrono_tz::Tz, year: i32, month: u32, day: u32) -> DateTime<Utc> {
    let date = NaiveDate::from_ymd_opt(year, month, day).unwrap_or_default();
    tz.from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap_or_default()).earliest().map(|t| t.with_timezone(&Utc)).unwrap_or_default()
}

/// The person's timezone and money month start day.
pub async fn settings(conn: &mut PgConnection, user_id: Uuid) -> Result<(chrono_tz::Tz, u32), sqlx::Error> {
    let row: Option<(String, i16)> = sqlx::query_as("SELECT timezone, money_period_start_day FROM user_preferences WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(&mut *conn)
        .await?;
    Ok(match row {
        Some((timezone, day)) => (timezone.parse().unwrap_or(chrono_tz::UTC), day.clamp(1, 28) as u32),
        None => (chrono_tz::UTC, 1),
    })
}

/// Sets the day their money month starts (1–28).
pub async fn set_start_day(conn: &mut PgConnection, user_id: Uuid, day: i64) -> Result<(), String> {
    if !(1..=28).contains(&day) {
        return Err("the start day must be from 1 to 28".to_string());
    }
    sqlx::query(
        "INSERT INTO user_preferences (user_id, money_period_start_day) VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE SET money_period_start_day = $2, updated_at = now()",
    )
    .bind(user_id)
    .bind(day as i16)
    .execute(&mut *conn)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 12, 0, 0).unwrap()
    }

    #[test]
    fn a_payday_month_runs_from_the_25th_to_the_24th() {
        let p = Period::containing(chrono_tz::UTC, 25, at(2026, 11, 3));
        assert_eq!(p.key(), "2026-10");
        assert_eq!(p.first_day().to_string(), "2026-10-25");
        assert_eq!(p.last_day().to_string(), "2026-11-24");
        assert_eq!(Period::containing(chrono_tz::UTC, 25, at(2026, 11, 25)).key(), "2026-11");
    }

    #[test]
    fn day_one_is_the_calendar_month_and_wraps_years() {
        let p = Period::containing(chrono_tz::UTC, 1, at(2026, 12, 31));
        assert_eq!((p.first_day().to_string(), p.last_day().to_string()), ("2026-12-01".into(), "2026-12-31".into()));
        assert_eq!(p.shifted(chrono_tz::UTC, 1).key(), "2027-01");
        assert_eq!(Period::containing(chrono_tz::UTC, 25, at(2026, 1, 10)).key(), "2025-12");
    }

    #[test]
    fn periods_start_at_local_midnight() {
        let jakarta: chrono_tz::Tz = "Asia/Jakarta".parse().unwrap();
        let p = Period::starting_in(jakarta, 25, 2026, 10);
        assert_eq!(p.start.to_rfc3339(), "2026-10-24T17:00:00+00:00");
    }
}
