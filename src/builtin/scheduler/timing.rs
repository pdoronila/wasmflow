//! Timing utilities for precise time measurement and budget enforcement

use std::time::{Duration, Instant};

/// Timer for measuring and enforcing execution budgets
pub struct BudgetTimer {
    start_time: Instant,
    budget: Duration,
}

impl BudgetTimer {
    /// Create a new budget timer
    pub fn new(budget_ms: u32) -> Self {
        Self {
            start_time: Instant::now(),
            budget: Duration::from_millis(budget_ms as u64),
        }
    }

    /// Check if budget has been exceeded
    pub fn is_exceeded(&self) -> bool {
        self.start_time.elapsed() > self.budget
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u32 {
        self.start_time.elapsed().as_millis() as u32
    }

    /// Get remaining budget in milliseconds
    pub fn remaining_ms(&self) -> i32 {
        let elapsed = self.elapsed_ms();
        let budget = self.budget.as_millis() as u32;
        (budget as i32) - (elapsed as i32)
    }

    /// Get budget utilization (0.0 to 1.0+, can exceed 1.0 on overrun)
    pub fn utilization(&self) -> f32 {
        let elapsed = self.elapsed_ms() as f32;
        let budget = self.budget.as_millis() as f32;
        elapsed / budget
    }
}

/// Deadline tracker for periodic tasks
pub struct DeadlineTracker {
    activation_time: Instant,
    deadline: Duration,
}

impl DeadlineTracker {
    /// Create a new deadline tracker
    pub fn new(activation_time: Instant, deadline_ms: u32) -> Self {
        Self {
            activation_time,
            deadline: Duration::from_millis(deadline_ms as u64),
        }
    }

    /// Get absolute deadline instant
    pub fn deadline_instant(&self) -> Instant {
        self.activation_time + self.deadline
    }

    /// Check if deadline has been missed
    pub fn is_missed(&self) -> bool {
        Instant::now() > self.deadline_instant()
    }

    /// Get time until deadline (negative if missed)
    pub fn laxity_ms(&self) -> i32 {
        let now = Instant::now();
        let deadline = self.deadline_instant();
        if now > deadline {
            -(now.duration_since(deadline).as_millis() as i32)
        } else {
            deadline.duration_since(now).as_millis() as i32
        }
    }

    /// Get time since activation in milliseconds
    pub fn elapsed_ms(&self) -> u32 {
        self.activation_time.elapsed().as_millis() as u32
    }
}

/// Periodic timer for task activation
pub struct PeriodicTimer {
    period: Duration,
    next_activation: Instant,
}

impl PeriodicTimer {
    /// Create a new periodic timer
    pub fn new(period_ms: u32, start_offset_ms: u32) -> Self {
        let period = Duration::from_millis(period_ms as u64);
        let start_offset = Duration::from_millis(start_offset_ms as u64);
        Self {
            period,
            next_activation: Instant::now() + start_offset,
        }
    }

    /// Check if it's time for next activation
    pub fn is_ready(&self) -> bool {
        Instant::now() >= self.next_activation
    }

    /// Advance to next period
    pub fn advance(&mut self) {
        self.next_activation += self.period;
    }

    /// Reset timer to current time + period
    pub fn reset(&mut self) {
        self.next_activation = Instant::now() + self.period;
    }

    /// Get time until next activation in milliseconds
    pub fn time_until_next_ms(&self) -> u32 {
        let now = Instant::now();
        if now >= self.next_activation {
            0
        } else {
            (self.next_activation - now).as_millis() as u32
        }
    }
}

/// Calculate jitter (variation in execution timing)
pub fn calculate_jitter(execution_times_ms: &[u32]) -> f32 {
    if execution_times_ms.len() < 2 {
        return 0.0;
    }

    let mean: f32 = execution_times_ms.iter().sum::<u32>() as f32 / execution_times_ms.len() as f32;

    let variance: f32 = execution_times_ms
        .iter()
        .map(|&x| {
            let diff = x as f32 - mean;
            diff * diff
        })
        .sum::<f32>()
        / execution_times_ms.len() as f32;

    variance.sqrt()
}

/// Compensate for scheduling overhead in next sleep duration
pub fn compensate_sleep_duration(
    target_interval_ms: u32,
    actual_elapsed_ms: u32,
) -> Duration {
    if actual_elapsed_ms >= target_interval_ms {
        // No sleep needed, we're already behind
        Duration::from_millis(0)
    } else {
        let remaining = target_interval_ms - actual_elapsed_ms;
        Duration::from_millis(remaining as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_budget_timer() {
        let timer = BudgetTimer::new(100);
        assert!(!timer.is_exceeded());
        assert!(timer.remaining_ms() > 0);

        thread::sleep(Duration::from_millis(50));
        assert!(!timer.is_exceeded());
        assert!(timer.elapsed_ms() >= 50);
    }

    #[test]
    fn test_deadline_tracker() {
        let now = Instant::now();
        let tracker = DeadlineTracker::new(now, 100);
        assert!(!tracker.is_missed());
        assert!(tracker.laxity_ms() > 0);

        thread::sleep(Duration::from_millis(50));
        assert!(!tracker.is_missed());
    }

    #[test]
    fn test_periodic_timer() {
        let mut timer = PeriodicTimer::new(100, 0);
        assert!(timer.is_ready());

        timer.reset();
        assert!(!timer.is_ready());
        assert!(timer.time_until_next_ms() > 0);
    }

    #[test]
    fn test_jitter_calculation() {
        let times = vec![100, 102, 98, 101, 99];
        let jitter = calculate_jitter(&times);
        assert!(jitter > 0.0);
        assert!(jitter < 5.0); // Should be small for these values
    }

    #[test]
    fn test_sleep_compensation() {
        let duration = compensate_sleep_duration(100, 30);
        assert_eq!(duration.as_millis(), 70);

        let duration = compensate_sleep_duration(100, 150);
        assert_eq!(duration.as_millis(), 0);
    }
}
