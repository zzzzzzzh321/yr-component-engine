//! Shared signed frame timer; the SDK and engine use one numeric owner.

pub use vera_mod_api::timer::{CdTimer, PAUSED_START_FRAME};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn countdown_expires_at_exact_boundary() {
        let timer = CdTimer::started(100, 5);

        assert_eq!(timer.remaining(100), 5);
        assert_eq!(timer.remaining(104), 1);
        assert_eq!(timer.remaining(105), 0);
        assert!(timer.expired(105));
        assert_eq!(timer.remaining(106), 0);
    }

    #[test]
    fn signed_frame_subtraction_wraps() {
        let timer = CdTimer::started(i32::MAX - 1, 4);

        assert_eq!(timer.remaining(i32::MIN + 1), 1);
        assert_eq!(timer.remaining(i32::MIN + 2), 0);
    }

    #[test]
    fn signed_negative_elapsed_returns_the_wrapping_remainder() {
        let timer = CdTimer::started(0, 6);

        assert_eq!(timer.remaining(i32::MIN), i32::MIN.wrapping_add(6));
        assert!(!timer.expired(i32::MIN));
    }

    #[test]
    fn pause_freezes_and_resume_reanchors_remaining_time() {
        let mut timer = CdTimer::started(100, 10);

        timer.pause(103);
        assert!(timer.is_paused());
        assert_eq!(timer.duration(), 7);
        assert_eq!(timer.remaining(500), 7);

        timer.resume(500);
        assert!(!timer.is_paused());
        assert_eq!(timer.start_frame(), 500);
        assert_eq!(timer.remaining(506), 1);
        assert!(timer.expired(507));
    }

    #[test]
    fn pausing_an_expired_timer_stores_zero() {
        let mut timer = CdTimer::started(10, 3);

        timer.pause(13);

        assert_eq!(timer, CdTimer::from_raw(PAUSED_START_FRAME, 0));
        assert!(timer.expired(999));
    }

    #[test]
    fn a_running_timer_with_no_or_negative_time_has_expired() {
        assert!(CdTimer::started(100, 0).expired(100));
        let timer = CdTimer::started(100, -7);
        assert_eq!(timer.remaining(100), 0);
        assert!(timer.expired(100));
    }

    #[test]
    fn raw_paused_duration_is_not_implicitly_clamped() {
        let timer = CdTimer::from_raw(PAUSED_START_FRAME, -7);

        assert_eq!(timer.remaining(123), -7);
        assert!(!timer.expired(123));
    }
}
