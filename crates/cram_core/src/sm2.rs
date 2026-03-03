use chrono::{NaiveDate, TimeDelta};

use crate::card::ReviewState;

/// User's self-assessed quality of a review response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rating {
    /// Complete failure to recall.
    Again,
    /// Correct but with significant difficulty.
    Hard,
    /// Correct with moderate effort.
    Good,
    /// Correct with little effort.
    Easy,
}

impl Rating {
    /// SM-2 quality score (0-5). We map our four ratings to 1, 3, 4, 5.
    fn quality(self) -> u8 {
        match self {
            Self::Again => 1,
            Self::Hard => 3,
            Self::Good => 4,
            Self::Easy => 5,
        }
    }
}

const MIN_EASE_FACTOR: f64 = 1.3;

/// Apply the SM-2 algorithm to update a card's review state after a rating.
///
/// Returns the updated `ReviewState` with new interval, ease factor, and due date.
pub fn schedule(state: &ReviewState, rating: Rating, today: NaiveDate) -> ReviewState {
    let q = rating.quality();

    let (repetitions, interval, ease_factor) = if q < 3 {
        // Failed recall: reset to beginning
        (0, 1_u32, state.ease_factor)
    } else {
        // Successful recall: advance schedule
        let new_ef = (state.ease_factor
            + (0.1 - (5.0 - f64::from(q)) * (0.08 + (5.0 - f64::from(q)) * 0.02)))
            .max(MIN_EASE_FACTOR);

        let new_interval = match state.repetitions {
            0 => 1,
            1 => 6,
            _ => {
                #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let computed = (f64::from(state.interval) * new_ef).round() as u32;
                computed.max(1)
            }
        };

        (state.repetitions + 1, new_interval, new_ef)
    };

    let due_date = today
        .checked_add_signed(TimeDelta::days(i64::from(interval)))
        .unwrap_or(today);

    ReviewState {
        repetitions,
        interval,
        ease_factor,
        due_date: Some(due_date),
    }
}

/// Returns `true` if a card is due for review on the given date.
///
/// Cards that have never been reviewed are always due.
pub fn is_due(state: &ReviewState, today: NaiveDate) -> bool {
    match state.due_date {
        None => true,
        Some(due) => today >= due,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::ReviewState;

    fn default_state() -> ReviewState {
        ReviewState::default()
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
    }

    #[test]
    fn first_review_good_sets_interval_to_1() {
        let state = default_state();
        let today = date(2026, 3, 1);
        let next = schedule(&state, Rating::Good, today);
        assert_eq!(next.repetitions, 1);
        assert_eq!(next.interval, 1);
        assert_eq!(next.due_date, Some(date(2026, 3, 2)));
    }

    #[test]
    fn second_review_good_sets_interval_to_6() {
        let state = ReviewState {
            repetitions: 1,
            interval: 1,
            ease_factor: 2.5,
            due_date: Some(date(2026, 3, 2)),
        };
        let next = schedule(&state, Rating::Good, date(2026, 3, 2));
        assert_eq!(next.repetitions, 2);
        assert_eq!(next.interval, 6);
        assert_eq!(next.due_date, Some(date(2026, 3, 8)));
    }

    #[test]
    fn third_review_good_applies_ease_factor() {
        let state = ReviewState {
            repetitions: 2,
            interval: 6,
            ease_factor: 2.5,
            due_date: Some(date(2026, 3, 8)),
        };
        let next = schedule(&state, Rating::Good, date(2026, 3, 8));
        assert_eq!(next.repetitions, 3);
        assert_eq!(next.interval, 15);
    }

    #[test]
    fn again_resets_repetitions() {
        let state = ReviewState {
            repetitions: 5,
            interval: 30,
            ease_factor: 2.5,
            due_date: Some(date(2026, 3, 1)),
        };
        let next = schedule(&state, Rating::Again, date(2026, 3, 1));
        assert_eq!(next.repetitions, 0);
        assert_eq!(next.interval, 1);
        assert_eq!(next.due_date, Some(date(2026, 3, 2)));
    }

    #[test]
    fn ease_factor_decreases_on_hard() {
        let state = default_state();
        let next = schedule(&state, Rating::Hard, date(2026, 3, 1));
        assert!(next.ease_factor < 2.5);
    }

    #[test]
    fn ease_factor_increases_on_easy() {
        let state = default_state();
        let next = schedule(&state, Rating::Easy, date(2026, 3, 1));
        assert!(next.ease_factor > 2.5);
    }

    #[test]
    fn ease_factor_does_not_go_below_minimum() {
        let state = ReviewState {
            repetitions: 1,
            interval: 1,
            ease_factor: MIN_EASE_FACTOR,
            due_date: Some(date(2026, 3, 1)),
        };
        let next = schedule(&state, Rating::Hard, date(2026, 3, 1));
        assert!(next.ease_factor >= MIN_EASE_FACTOR);
    }

    #[test]
    fn new_card_is_due() {
        assert!(is_due(&default_state(), date(2026, 3, 1)));
    }

    #[test]
    fn card_is_due_on_due_date() {
        let state = ReviewState {
            due_date: Some(date(2026, 3, 5)),
            ..default_state()
        };
        assert!(is_due(&state, date(2026, 3, 5)));
    }

    #[test]
    fn card_is_due_after_due_date() {
        let state = ReviewState {
            due_date: Some(date(2026, 3, 5)),
            ..default_state()
        };
        assert!(is_due(&state, date(2026, 3, 10)));
    }

    #[test]
    fn card_is_not_due_before_due_date() {
        let state = ReviewState {
            due_date: Some(date(2026, 3, 5)),
            ..default_state()
        };
        assert!(!is_due(&state, date(2026, 3, 4)));
    }

    #[test]
    fn again_preserves_ease_factor() {
        let state = ReviewState {
            repetitions: 3,
            interval: 15,
            ease_factor: 2.2,
            due_date: Some(date(2026, 3, 1)),
        };
        let next = schedule(&state, Rating::Again, date(2026, 3, 1));
        assert!((next.ease_factor - 2.2).abs() < f64::EPSILON);
    }
}
