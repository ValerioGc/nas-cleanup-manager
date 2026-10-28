//! Rule schedules. Daily, weekly on a weekday, monthly on a day or yearly on a date, always in the
//! local time of the NAS.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// The time uses the 24 hour format. `weekday` (1 Monday to 7 Sunday) applies to weekly rules,
/// `day` to monthly and yearly rules and `month` (1 to 12) to yearly rules. A day missing from a
/// shorter month runs on its last day.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuleSchedule {
    pub frequency: Frequency,
    pub time: String,
    #[serde(default)]
    pub weekday: Option<u8>,
    #[serde(default)]
    pub day: Option<u8>,
    #[serde(default)]
    pub month: Option<u8>,
}

/// Days in each month, with 29 for February.
const MONTH_DAYS: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Minute of the day of a valid 24 hour time, otherwise `None`.
pub fn minute_of(time: &str) -> Option<u16> {
    let (hours, minutes) = time.split_once(':')?;
    if hours.len() != 2 || minutes.len() != 2 {
        return None;
    }
    let hours: u16 = hours.parse().ok().filter(|hours| *hours < 24)?;
    let minutes: u16 = minutes.parse().ok().filter(|minutes| *minutes < 60)?;
    Some(hours * 60 + minutes)
}

impl RuleSchedule {
    /// Checks that the time is valid and that only the fields of the frequency are set.
    pub fn validate(&self) -> Result<(), &'static str> {
        minute_of(&self.time).ok_or("The time of the schedule is not HH:MM.")?;
        let weekday = self.weekday.map(|weekday| (1..=7).contains(&weekday));
        let month = self.month.map(|month| (1..=12).contains(&month));
        let day = self.day.map(|day| {
            let last = self
                .month
                .and_then(|month| MONTH_DAYS.get(usize::from(month).wrapping_sub(1)))
                .copied()
                .unwrap_or(31);
            (1..=last).contains(&day)
        });
        let valid = match self.frequency {
            Frequency::Daily => weekday.is_none() && day.is_none() && month.is_none(),
            Frequency::Weekly => weekday == Some(true) && day.is_none() && month.is_none(),
            Frequency::Monthly => weekday.is_none() && day == Some(true) && month.is_none(),
            Frequency::Yearly => weekday.is_none() && day == Some(true) && month == Some(true),
        };
        if valid {
            Ok(())
        } else {
            Err("The schedule does not match its frequency.")
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/axum/schedule.rs"]
mod tests;
