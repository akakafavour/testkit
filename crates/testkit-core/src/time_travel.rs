//! Deterministic clock control for time-dependent tests.
//!
//! Tests rarely wait thirty days for a trial to expire. [`VirtualClock`]
//! anchors a base instant and applies a caller-controlled offset so expiry,
//! cooling-off and scheduled-job logic can be exercised instantly.

use chrono::{DateTime, Duration, Utc};

/// Canonical jump sizes accepted by `--time-travel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeJump {
    OneHour,
    SevenDays,
    FourteenDays,
    ThirtyDays,
}

impl TimeJump {
    /// Length of the jump as a chrono duration.
    pub fn as_duration(&self) -> Duration {
        match self {
            Self::OneHour => Duration::hours(1),
            Self::SevenDays => Duration::days(7),
            Self::FourteenDays => Duration::days(14),
            Self::ThirtyDays => Duration::days(30),
        }
    }

    /// Wire format accepted by [`parse_jump_spec`].
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneHour => "1h",
            Self::SevenDays => "7d",
            Self::FourteenDays => "14d",
            Self::ThirtyDays => "30d",
        }
    }
}

impl std::fmt::Display for TimeJump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parses `"1h"`, `"7d"`, `"14d"` or `"30d"`, ignoring surrounding whitespace
/// and case. Returns `None` for anything else.
pub fn parse_jump_spec(s: &str) -> Option<TimeJump> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1h" => Some(TimeJump::OneHour),
        "7d" => Some(TimeJump::SevenDays),
        "14d" => Some(TimeJump::FourteenDays),
        "30d" => Some(TimeJump::ThirtyDays),
        _ => None,
    }
}

/// A wall clock frozen at `base` plus an accumulated offset.
///
/// `now()` never reads the operating system clock, so tests stay reproducible.
#[derive(Debug, Clone)]
pub struct VirtualClock {
    base: DateTime<Utc>,
    offset: Duration,
}

impl VirtualClock {
    /// Creates a clock whose `now()` equals `base`.
    pub fn new(base: DateTime<Utc>) -> Self {
        Self {
            base,
            offset: Duration::zero(),
        }
    }

    /// Current virtual instant: `base + offset`.
    pub fn now(&self) -> DateTime<Utc> {
        self.base + self.offset
    }

    /// Moves the clock by `d`. Negative durations rewind it.
    pub fn advance(&mut self, d: Duration) {
        self.offset = self.offset + d;
    }

    /// Applies a canonical jump size.
    pub fn jump(&mut self, j: TimeJump) {
        self.advance(j.as_duration());
    }

    /// Returns the clock to its base instant.
    pub fn reset(&mut self) {
        self.offset = Duration::zero();
    }

    /// Accumulated offset from the base instant.
    pub fn offset(&self) -> Duration {
        self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn base_instant() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap()
    }

    #[test]
    fn parses_canonical_specs_with_whitespace_and_case() {
        assert_eq!(parse_jump_spec("1h"), Some(TimeJump::OneHour));
        assert_eq!(parse_jump_spec(" 7D "), Some(TimeJump::SevenDays));
        assert_eq!(parse_jump_spec("14d"), Some(TimeJump::FourteenDays));
        assert_eq!(parse_jump_spec("\t30D\n"), Some(TimeJump::ThirtyDays));
    }

    #[test]
    fn rejects_unknown_specs() {
        for spec in ["", "1", "1d", "2h", "7 days", "thirty", "-7d"] {
            assert_eq!(parse_jump_spec(spec), None, "spec {spec:?} must be rejected");
        }
    }

    #[test]
    fn jump_durations_match_specs() {
        assert_eq!(TimeJump::OneHour.as_duration(), Duration::hours(1));
        assert_eq!(TimeJump::SevenDays.as_duration(), Duration::days(7));
        assert_eq!(TimeJump::FourteenDays.as_duration(), Duration::days(14));
        assert_eq!(TimeJump::ThirtyDays.as_duration(), Duration::days(30));
    }

    #[test]
    fn starts_at_base_with_zero_offset() {
        let clock = VirtualClock::new(base_instant());
        assert_eq!(clock.now(), base_instant());
        assert_eq!(clock.offset(), Duration::zero());
    }

    #[test]
    fn advance_moves_now_and_accumulates() {
        let mut clock = VirtualClock::new(base_instant());
        clock.advance(Duration::minutes(90));
        assert_eq!(clock.now(), base_instant() + Duration::minutes(90));
        clock.advance(Duration::minutes(-30));
        assert_eq!(clock.now(), base_instant() + Duration::hours(1));
        assert_eq!(clock.offset(), Duration::hours(1));
    }

    #[test]
    fn jump_and_reset_behave_as_expected() {
        let mut clock = VirtualClock::new(base_instant());
        clock.jump(TimeJump::ThirtyDays);
        assert_eq!(clock.now(), base_instant() + Duration::days(30));
        clock.jump(TimeJump::OneHour);
        assert_eq!(clock.offset(), Duration::days(30) + Duration::hours(1));
        clock.reset();
        assert_eq!(clock.now(), base_instant());
        assert_eq!(clock.offset(), Duration::zero());
    }
}
