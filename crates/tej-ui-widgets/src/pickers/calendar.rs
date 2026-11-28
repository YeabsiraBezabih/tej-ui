//! Calendar date representation, leap-year calculations, and date range picker state machine.

use std::fmt::Debug;

/// Gregorian calendar date (Year, Month 1..=12, Day 1..=31).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct CalendarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl CalendarDate {
    #[must_use]
    pub const fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Check if `year` is a leap year.
    #[must_use]
    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Days count in specified month.
    #[must_use]
    pub const fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            _ => 30,
        }
    }

    /// Calculate day of week (0 = Sunday, 1 = Monday, ..., 6 = Saturday) using Sakamoto's algorithm.
    #[must_use]
    pub const fn day_of_week(&self) -> u8 {
        let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = self.year;
        if self.month < 3 {
            y -= 1;
        }
        let m = self.month as usize;
        let day_idx = (y + y / 4 - y / 100 + y / 400 + t[m - 1] + self.day as i32) % 7;
        day_idx as u8
    }

    /// Format as ISO 8601 string `YYYY-MM-DD`.
    #[must_use]
    pub fn to_iso_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Continuous date interval with inclusive start and end dates.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DateRange {
    pub start: CalendarDate,
    pub end: CalendarDate,
}

impl DateRange {
    #[must_use]
    pub const fn new(start: CalendarDate, end: CalendarDate) -> Self {
        Self { start, end }
    }

    /// Check if `date` falls within this range (inclusive).
    #[must_use]
    pub fn contains(&self, date: CalendarDate) -> bool {
        date >= self.start && date <= self.end
    }
}

/// Interactive date range picker coordinator.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CalendarPickerState {
    pub view_year: i32,
    pub view_month: u8,
    pub range: Option<DateRange>,
    pub pending_start: Option<CalendarDate>,
}

impl CalendarPickerState {
    #[must_use]
    pub fn new(year: i32, month: u8) -> Self {
        Self {
            view_year: year,
            view_month: month.clamp(1, 12),
            range: None,
            pending_start: None,
        }
    }

    pub fn next_month(&mut self) {
        if self.view_month == 12 {
            self.view_year += 1;
            self.view_month = 1;
        } else {
            self.view_month += 1;
        }
    }

    pub fn prev_month(&mut self) {
        if self.view_month == 1 {
            self.view_year -= 1;
            self.view_month = 12;
        } else {
            self.view_month -= 1;
        }
    }

    /// Select or extend a date selection.
    pub fn select_date(&mut self, date: CalendarDate) {
        if let Some(start) = self.pending_start {
            if date >= start {
                self.range = Some(DateRange::new(start, date));
            } else {
                self.range = Some(DateRange::new(date, start));
            }
            self.pending_start = None;
        } else {
            self.pending_start = Some(date);
            self.range = Some(DateRange::new(date, date));
        }
    }
}
