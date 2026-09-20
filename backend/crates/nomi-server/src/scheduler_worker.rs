use chrono::{DateTime, Datelike, Utc};

pub(crate) fn next_occurrence(
    from: DateTime<Utc>,
    recurrence: &str,
    weekday: Option<i16>,
    day_of_month: Option<i16>,
) -> DateTime<Utc> {
    match recurrence {
        "daily" => from + chrono::Duration::days(1),
        "weekly" => {
            // `weekday` is 0=Sunday..6=Saturday (matches the create_reminder tool schema's
            // description). chrono's Weekday::num_days_from_sunday() uses the same convention.
            let target = weekday.unwrap_or(0) as u32;
            let mut candidate = from + chrono::Duration::days(1);
            while candidate.weekday().num_days_from_sunday() != target {
                candidate += chrono::Duration::days(1);
            }
            candidate
        }
        "monthly" => {
            let day = day_of_month.unwrap_or(1) as u32;
            let (mut year, mut month) = (from.year(), from.month());
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            let last_day_of_month = chrono::NaiveDate::from_ymd_opt(year, month, 1)
                .unwrap()
                .checked_add_months(chrono::Months::new(1))
                .unwrap()
                .pred_opt()
                .unwrap()
                .day();
            let clamped_day = day.min(last_day_of_month);
            // Step through day=1 first: with_month()/with_year() reject a result that isn't a
            // real calendar date, and `from`'s own day-of-month (e.g. 31) may not exist in the
            // target month — day=1 always does, in every month.
            from.with_day(1).unwrap().with_year(year).unwrap().with_month(month).unwrap().with_day(clamped_day).unwrap()
        }
        _ => from,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn daily_advances_by_exactly_24_hours() {
        let from = Utc.with_ymd_and_hms(2026, 9, 20, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "daily", None, None);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 21, 11, 0, 0).unwrap());
    }

    #[test]
    fn weekly_advances_to_the_next_matching_weekday() {
        // 2026-09-20 is a Sunday (weekday 0). Target weekday 3 = Wednesday.
        let from = Utc.with_ymd_and_hms(2026, 9, 20, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "weekly", Some(3), None);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 23, 11, 0, 0).unwrap());
    }

    #[test]
    fn weekly_rolls_a_full_week_when_today_already_matches() {
        // 2026-09-20 is a Sunday (weekday 0). Target weekday 0 = Sunday.
        let from = Utc.with_ymd_and_hms(2026, 9, 20, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "weekly", Some(0), None);
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 27, 11, 0, 0).unwrap());
    }

    #[test]
    fn monthly_advances_to_the_next_month_same_day() {
        let from = Utc.with_ymd_and_hms(2026, 9, 15, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "monthly", None, Some(15));
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 10, 15, 11, 0, 0).unwrap());
    }

    #[test]
    fn monthly_clamps_to_the_target_months_last_day() {
        // day_of_month = 31 scheduled from January lands on February's last valid day (28, 2026
        // is not a leap year), not an invalid date and not rolling into March.
        let from = Utc.with_ymd_and_hms(2026, 1, 31, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "monthly", None, Some(31));
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 2, 28, 11, 0, 0).unwrap());
    }

    #[test]
    fn monthly_wraps_from_december_into_january_of_the_next_year() {
        let from = Utc.with_ymd_and_hms(2026, 12, 10, 11, 0, 0).unwrap();
        let next = next_occurrence(from, "monthly", None, Some(10));
        assert_eq!(next, Utc.with_ymd_and_hms(2027, 1, 10, 11, 0, 0).unwrap());
    }
}
