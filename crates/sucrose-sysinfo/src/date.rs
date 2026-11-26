//! The Date group: the local time with the keys and value types of the legacy `Data.cs` `GetDateInfo`, which are those of
//! the .NET `DateTime.Now` it was built from: whole numbers for the parts, `State` true, `Kind` and `DayOfWeek` as the
//! text of their enum names (`$"{DateData.Kind}"`, `$"{DateData.DayOfWeek}"`).

use serde_json::{Value, json};

use crate::SysInfoError;

/// The parts of a local time, as the OS gives them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct LocalTime {
    pub year: u16,
    pub month: u16,
    /// 0 is Sunday, as for the Win32 `SYSTEMTIME` and the .NET `DayOfWeek`.
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub millisecond: u16,
}

/// The name .NET gives a `DayOfWeek` value.
fn day_name(day_of_week: u16) -> &'static str {
    match day_of_week {
        0 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        _ => "Saturday",
    }
}

/// The day of the year, 1 for January 1st, as .NET's `DateTime.DayOfYear`.
fn day_of_year(year: u16, month: u16, day: u16) -> u16 {
    const BEFORE_MONTH: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let index = usize::from(month.clamp(1, 12) - 1);
    let extra = u16::from(leap && month > 2);
    BEFORE_MONTH[index] + extra + day
}

/// The legacy object of `time`: the keys in the order of `Data.cs`.
pub(crate) fn snapshot(time: &LocalTime) -> Value {
    json!({
        "Day": time.day,
        "Hour": time.hour,
        "Year": time.year,
        "Month": time.month,
        "State": true,
        "Minute": time.minute,
        "Second": time.second,
        // `DateTime.Now` is a local time.
        "Kind": "Local",
        "DayOfYear": day_of_year(time.year, time.month, time.day),
        "Millisecond": time.millisecond,
        "DayOfWeek": day_name(time.day_of_week),
    })
}

#[cfg(windows)]
fn local_time() -> Result<LocalTime, SysInfoError> {
    use windows::Win32::System::SystemInformation::GetLocalTime;

    // SAFETY: GetLocalTime has no preconditions and returns the SYSTEMTIME by value.
    let time = unsafe { GetLocalTime() };
    Ok(LocalTime {
        year: time.wYear,
        month: time.wMonth,
        day_of_week: time.wDayOfWeek,
        day: time.wDay,
        hour: time.wHour,
        minute: time.wMinute,
        second: time.wSecond,
        millisecond: time.wMilliseconds,
    })
}

#[cfg(not(windows))]
fn local_time() -> Result<LocalTime, SysInfoError> {
    Err(SysInfoError::Unsupported("the local time"))
}

/// The Date group for the current local time.
///
/// # Errors
/// [`SysInfoError::Unsupported`] on an OS without a provider.
pub fn date_snapshot() -> Result<Value, SysInfoError> {
    Ok(snapshot(&local_time()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TIME: LocalTime = LocalTime {
        year: 2026,
        month: 11,
        day_of_week: 4,
        day: 26,
        hour: 4,
        minute: 7,
        second: 9,
        millisecond: 120,
    };

    #[test]
    fn the_keys_come_in_the_legacy_order() {
        let value = snapshot(&TIME);

        let keys: Vec<&str> = value
            .as_object()
            .expect("an object")
            .keys()
            .map(String::as_str)
            .collect();

        assert_eq!(
            keys,
            [
                "Day",
                "Hour",
                "Year",
                "Month",
                "State",
                "Minute",
                "Second",
                "Kind",
                "DayOfYear",
                "Millisecond",
                "DayOfWeek"
            ]
        );
    }

    #[test]
    fn the_values_have_the_legacy_types() {
        let value = snapshot(&TIME);

        for key in [
            "Day",
            "Hour",
            "Year",
            "Month",
            "Minute",
            "Second",
            "DayOfYear",
            "Millisecond",
        ] {
            assert!(value[key].is_u64(), "{key} is a whole number: {}", value[key]);
        }
        assert_eq!(value["State"], true);
        assert_eq!(value["Kind"], "Local");
        assert_eq!(value["DayOfWeek"], "Thursday");
        assert_eq!(value["Day"], 26);
        assert_eq!(value["Millisecond"], 120);
        assert_eq!(
            value["DayOfYear"], 330,
            "2026 is not a leap year: 304 days before November + 26"
        );
    }

    #[test]
    fn the_day_of_the_year_counts_the_leap_day() {
        assert_eq!(day_of_year(2024, 1, 1), 1);
        assert_eq!(day_of_year(2024, 3, 1), 61);
        assert_eq!(day_of_year(2023, 3, 1), 60);
        assert_eq!(day_of_year(2024, 12, 31), 366);
        assert_eq!(day_of_year(1900, 3, 1), 60, "1900 is not a leap year");
        assert_eq!(day_of_year(2000, 3, 1), 61, "2000 is");
    }

    #[test]
    fn the_day_names_are_the_dotnet_names() {
        let names: Vec<&str> = (0..7).map(day_name).collect();

        assert_eq!(
            names,
            [
                "Sunday",
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday"
            ]
        );
    }

    #[cfg(windows)]
    #[test]
    fn the_current_time_reads_on_windows() {
        let value = date_snapshot().expect("the local time reads");

        assert!(value["Year"].as_u64().is_some_and(|year| year >= 2026));
        assert_eq!(value["State"], true);
    }

    #[cfg(not(windows))]
    #[test]
    fn other_systems_report_the_provider_as_unsupported() {
        assert!(matches!(date_snapshot(), Err(SysInfoError::Unsupported(_))));
    }
}
