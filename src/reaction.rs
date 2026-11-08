//! Reaction game logic — pure, no I/O, fully unit-testable.
//!
//! A target letter sits on screen until the player presses the matching key.
//! Points are awarded by reaction speed plus a combo bonus for consecutive
//! hits; wrong keys cost points and reset the combo.

use std::time::{Duration, Instant};

/// One round lasts this long.
pub const GAME_DURATION: Duration = Duration::from_secs(60);
/// Points lost per wrong key press.
pub const MISS_PENALTY: i64 = 20;
/// Combo bonus per consecutive hit beyond the first.
const COMBO_STEP: i64 = 5;
/// The combo bonus never grows past this many steps.
const COMBO_CAP: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressResult {
    /// Correct key — the target is consumed and points were scored.
    Hit { points: i64, reaction: Duration },
    /// Printable key that didn't match the target.
    Wrong,
    /// Nothing to judge (no target on screen).
    Ignored,
}

#[derive(Debug, Clone, Copy)]
pub struct Target {
    pub ch: char,
    pub row: u16,
    pub col: u16,
    /// When the target appeared — reaction time is measured from here.
    pub spawned_at: Instant,
}

pub struct Session {
    pub score: i64,
    pub streak: u32,
    pub max_streak: u32,
    pub hits: u32,
    pub misses: u32,
    pub best_reaction: Option<Duration>,
    pub total_reaction: Duration,
    target: Option<Target>,
    started_at: Instant,
}

impl Session {
    pub fn new(now: Instant) -> Self {
        Self {
            score: 0,
            streak: 0,
            max_streak: 0,
            hits: 0,
            misses: 0,
            best_reaction: None,
            total_reaction: Duration::ZERO,
            target: None,
            started_at: now,
        }
    }

    pub fn target(&self) -> Option<Target> {
        self.target
    }

    /// Puts a new target on screen. Callers pick the character and position.
    pub fn spawn_target(&mut self, ch: char, row: u16, col: u16, now: Instant) {
        self.target = Some(Target { ch, row, col, spawned_at: now });
    }

    /// Judges one key press against the current target.
    pub fn press(&mut self, key: char, now: Instant) -> PressResult {
        let Some(t) = self.target else {
            return PressResult::Ignored;
        };
        if key.to_ascii_lowercase() != t.ch {
            self.misses += 1;
            self.score -= MISS_PENALTY;
            self.streak = 0;
            return PressResult::Wrong;
        }
        self.target = None;
        let reaction = now - t.spawned_at;
        self.hits += 1;
        self.streak += 1;
        self.max_streak = self.max_streak.max(self.streak);
        self.best_reaction = Some(match self.best_reaction {
            Some(best) if best < reaction => best,
            _ => reaction,
        });
        self.total_reaction += reaction;
        let points = points_for(reaction, self.streak);
        self.score += points;
        PressResult::Hit { points, reaction }
    }

    /// Time left before the round ends; clamps to zero.
    pub fn time_left(&self, now: Instant) -> Duration {
        GAME_DURATION.saturating_sub(now - self.started_at)
    }

    pub fn finished(&self, now: Instant) -> bool {
        now - self.started_at >= GAME_DURATION
    }

    /// Mean reaction time over all hits; `None` before the first hit.
    pub fn average_reaction(&self) -> Option<Duration> {
        (self.hits > 0).then(|| self.total_reaction / self.hits)
    }
}

/// Points for one hit: speed base plus combo bonus. Faster reactions score
/// more; the base floors at 10 and the combo bonus caps at `COMBO_CAP` steps.
pub fn points_for(reaction: Duration, streak: u32) -> i64 {
    let base = 100 - (reaction.as_millis().min(9_000) as i64 / 10);
    base.clamp(10, 100) + COMBO_STEP * streak.saturating_sub(1).min(COMBO_CAP) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_with_target(ch: char, t0: Instant) -> Session {
        let mut s = Session::new(t0);
        s.spawn_target(ch, 5, 10, t0);
        s
    }

    #[test]
    fn fast_reactions_score_more_than_slow() {
        let fast = points_for(Duration::from_millis(100), 1);
        let slow = points_for(Duration::from_millis(1_000), 1);
        assert!(fast > slow, "fast {fast} must beat slow {slow}");
        assert_eq!(points_for(Duration::from_secs(60), 1), 10, "base floors at 10");
        assert_eq!(points_for(Duration::ZERO, 1), 100, "base caps at 100");
    }

    #[test]
    fn combo_bonus_grows_and_caps() {
        // 500ms → base 50.
        assert_eq!(points_for(Duration::from_millis(500), 1), 50);
        assert_eq!(points_for(Duration::from_millis(500), 2), 55);
        assert_eq!(points_for(Duration::from_millis(500), 5), 70);
        assert_eq!(points_for(Duration::from_millis(500), 1_000), 100, "combo caps");
    }

    #[test]
    fn hit_scores_and_updates_stats() {
        let t0 = Instant::now();
        let mut s = session_with_target('a', t0);
        let expected = points_for(Duration::from_millis(200), 1);
        match s.press('a', t0 + Duration::from_millis(200)) {
            PressResult::Hit { points, reaction } => {
                assert_eq!(points, expected);
                assert_eq!(reaction, Duration::from_millis(200));
            }
            other => panic!("expected hit, got {other:?}"),
        }
        assert_eq!(s.hits, 1);
        assert_eq!(s.streak, 1);
        assert_eq!(s.score, expected);
        assert_eq!(s.best_reaction, Some(Duration::from_millis(200)));
        assert_eq!(s.average_reaction(), Some(Duration::from_millis(200)));
    }

    #[test]
    fn press_is_case_insensitive() {
        let t0 = Instant::now();
        let mut s = session_with_target('a', t0);
        assert!(matches!(s.press('A', t0), PressResult::Hit { .. }));
    }

    #[test]
    fn wrong_press_penalizes_and_resets_streak() {
        let t0 = Instant::now();
        let mut s = session_with_target('b', t0);
        match s.press('x', t0) {
            PressResult::Wrong => {}
            other => panic!("expected wrong, got {other:?}"),
        }
        assert_eq!(s.score, -MISS_PENALTY);
        assert_eq!(s.misses, 1);
        assert_eq!(s.streak, 0);
        assert_eq!(s.hits, 0);
    }

    #[test]
    fn wrong_after_hits_resets_combo() {
        let t0 = Instant::now();
        let mut s = session_with_target('a', t0);
        s.press('a', t0 + Duration::from_millis(100));
        s.spawn_target('b', 1, 1, t0 + Duration::from_millis(150));
        s.press('b', t0 + Duration::from_millis(250));
        assert_eq!(s.streak, 2);
        assert_eq!(s.max_streak, 2);
        let before = s.score;
        s.spawn_target('c', 2, 2, t0 + Duration::from_millis(300));
        s.press('z', t0 + Duration::from_millis(400));
        assert_eq!(s.score, before - MISS_PENALTY);
        assert_eq!(s.streak, 0, "combo must reset");
        assert_eq!(s.max_streak, 2, "max streak is remembered");
    }

    #[test]
    fn no_target_or_repeat_press_is_ignored() {
        let t0 = Instant::now();
        let mut s = Session::new(t0);
        assert_eq!(s.press('a', t0), PressResult::Ignored);
        s.spawn_target('a', 1, 1, t0);
        s.press('a', t0);
        assert_eq!(s.press('a', t0), PressResult::Ignored, "target consumed");
    }

    #[test]
    fn best_reaction_keeps_the_minimum() {
        let t0 = Instant::now();
        let mut s = session_with_target('a', t0);
        s.press('a', t0 + Duration::from_millis(500));
        s.spawn_target('b', 1, 1, t0 + Duration::from_millis(600));
        s.press('b', t0 + Duration::from_millis(700));
        assert_eq!(s.best_reaction, Some(Duration::from_millis(100)));
        assert_eq!(s.average_reaction(), Some(Duration::from_millis(300)));
    }

    #[test]
    fn session_ends_after_duration() {
        let t0 = Instant::now();
        let s = Session::new(t0);
        assert!(!s.finished(t0 + GAME_DURATION - Duration::from_millis(1)));
        assert!(s.finished(t0 + GAME_DURATION));
        assert_eq!(s.time_left(t0 + Duration::from_secs(30)), Duration::from_secs(30));
        assert_eq!(s.time_left(t0 + Duration::from_secs(70)), Duration::ZERO);
    }
}
