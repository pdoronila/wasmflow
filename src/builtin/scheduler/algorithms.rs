//! Scheduling algorithms implementation

use super::config::{ScheduleMode, SchedulerState, TaskExecutionState};
use std::time::Instant;

/// Select the next task to execute based on the current scheduling algorithm
pub fn select_next_task(state: &SchedulerState) -> Option<usize> {
    match state.schedule_mode {
        ScheduleMode::RoundRobin => select_round_robin(state),
        ScheduleMode::PriorityBased => select_priority_based(state),
        ScheduleMode::RateMonotonic => select_rate_monotonic(state),
        ScheduleMode::EarliestDeadlineFirst => select_earliest_deadline_first(state),
        ScheduleMode::LeastLaxityFirst => select_least_laxity_first(state),
    }
}

/// Round-robin: Select next ready task in circular order
fn select_round_robin(state: &SchedulerState) -> Option<usize> {
    if state.tasks.is_empty() {
        return None;
    }

    let start_index = state.current_task_index;
    let mut index = start_index;

    loop {
        // Check if task is ready
        if is_task_ready(state, index) {
            return Some(index);
        }

        // Move to next task
        index = (index + 1) % state.tasks.len();

        // If we've checked all tasks, return None
        if index == start_index {
            break;
        }
    }

    None
}

/// Priority-based: Select highest priority ready task
fn select_priority_based(state: &SchedulerState) -> Option<usize> {
    let mut best_index: Option<usize> = None;
    let mut best_priority: u8 = 0;

    for (index, task) in state.tasks.iter().enumerate() {
        if is_task_ready(state, index) {
            if best_index.is_none() || task.priority > best_priority {
                best_index = Some(index);
                best_priority = task.priority;
            }
        }
    }

    best_index
}

/// Rate Monotonic Scheduling: Shorter period = higher priority
fn select_rate_monotonic(state: &SchedulerState) -> Option<usize> {
    let mut best_index: Option<usize> = None;
    let mut best_period: u32 = u32::MAX;

    for (index, task) in state.tasks.iter().enumerate() {
        if is_task_ready(state, index) && task.is_periodic() {
            if best_index.is_none() || task.period_ms < best_period {
                best_index = Some(index);
                best_period = task.period_ms;
            }
        }
    }

    // If no periodic task is ready, fall back to aperiodic tasks
    if best_index.is_none() {
        for (index, task) in state.tasks.iter().enumerate() {
            if is_task_ready(state, index) && !task.is_periodic() {
                return Some(index);
            }
        }
    }

    best_index
}

/// Earliest Deadline First: Select task with earliest absolute deadline
fn select_earliest_deadline_first(state: &SchedulerState) -> Option<usize> {
    let now = Instant::now();
    let mut best_index: Option<usize> = None;
    let mut best_deadline: Option<Instant> = None;

    for (index, task) in state.tasks.iter().enumerate() {
        if is_task_ready(state, index) {
            // Calculate absolute deadline
            let task_state = &state.task_states[index];
            let deadline = if let Some(activation) = task_state.last_activation {
                activation
                    + std::time::Duration::from_millis(task.get_deadline() as u64)
            } else {
                // Not yet activated, deadline is now + deadline
                now + std::time::Duration::from_millis(task.get_deadline() as u64)
            };

            if best_index.is_none() || Some(deadline) < best_deadline {
                best_index = Some(index);
                best_deadline = Some(deadline);
            }
        }
    }

    best_index
}

/// Least Laxity First: Select task with minimum slack time
fn select_least_laxity_first(state: &SchedulerState) -> Option<usize> {
    let now = Instant::now();
    let mut best_index: Option<usize> = None;
    let mut best_laxity: Option<i64> = None;

    for (index, task) in state.tasks.iter().enumerate() {
        if is_task_ready(state, index) {
            let task_state = &state.task_states[index];

            // Calculate laxity = deadline - current_time - remaining_execution_time
            // For simplicity, use budget as remaining execution time estimate
            let deadline_instant = if let Some(activation) = task_state.last_activation {
                activation
                    + std::time::Duration::from_millis(task.get_deadline() as u64)
            } else {
                now + std::time::Duration::from_millis(task.get_deadline() as u64)
            };

            let time_to_deadline = if deadline_instant > now {
                deadline_instant
                    .duration_since(now)
                    .as_millis() as i64
            } else {
                -(now.duration_since(deadline_instant).as_millis() as i64)
            };

            let laxity = time_to_deadline - (task.budget_ms as i64);

            if best_index.is_none() || Some(laxity) < best_laxity {
                best_index = Some(index);
                best_laxity = Some(laxity);
            }
        }
    }

    best_index
}

/// Check if a task is ready to execute
fn is_task_ready(state: &SchedulerState, index: usize) -> bool {
    if index >= state.task_states.len() {
        return false;
    }

    let task = &state.tasks[index];
    let task_state = &state.task_states[index];

    // Task must be in Ready state
    if task_state.state != TaskExecutionState::Ready {
        return false;
    }

    // For periodic tasks, check if activation time has arrived
    if task.is_periodic() {
        if let Some(next_activation) = task_state.next_activation {
            return Instant::now() >= next_activation;
        } else {
            // First activation
            return true;
        }
    }

    // Aperiodic tasks are always ready when in Ready state
    true
}

/// Activate pending periodic tasks
pub fn activate_periodic_tasks(state: &mut SchedulerState) {
    let now = Instant::now();

    for (index, task) in state.tasks.iter().enumerate() {
        if task.is_periodic() {
            let task_state = &mut state.task_states[index];

            // Check if it's time for next activation
            if let Some(next_activation) = task_state.next_activation {
                if now >= next_activation {
                    task_state.activate(now);

                    // Schedule next activation
                    task_state.next_activation = Some(
                        next_activation
                            + std::time::Duration::from_millis(task.period_ms as u64),
                    );
                }
            } else {
                // First activation
                task_state.activate(now);
                task_state.next_activation =
                    Some(now + std::time::Duration::from_millis(task.period_ms as u64));
            }
        }
    }
}

/// Check for deadline misses across all tasks
pub fn check_deadline_misses(state: &mut SchedulerState) -> Vec<usize> {
    let now = Instant::now();
    let mut missed_tasks = Vec::new();

    for (index, task) in state.tasks.iter().enumerate() {
        if task.is_periodic() {
            let task_state = &mut state.task_states[index];

            if let Some(activation) = task_state.last_activation {
                let deadline =
                    activation + std::time::Duration::from_millis(task.get_deadline() as u64);

                if now > deadline && task_state.state != TaskExecutionState::Completed {
                    task_state.deadline_miss_count += 1;
                    missed_tasks.push(index);
                }
            }
        }
    }

    missed_tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task(component_id: &str, priority: u8, period_ms: u32) -> TaskConfig {
        let mut task = TaskConfig::new(component_id.to_string());
        task.priority = priority;
        task.period_ms = period_ms;
        task
    }

    #[test]
    fn test_round_robin_selection() {
        let mut state = SchedulerState::new();
        state.init_tasks(vec![
            create_test_task("task1", 100, 0),
            create_test_task("task2", 100, 0),
            create_test_task("task3", 100, 0),
        ]);

        // All tasks are ready, should select first
        let next = select_round_robin(&state);
        assert_eq!(next, Some(0));

        // Advance and select next
        state.current_task_index = 1;
        let next = select_round_robin(&state);
        assert_eq!(next, Some(1));

        // Wrap around
        state.current_task_index = 2;
        let next = select_round_robin(&state);
        assert_eq!(next, Some(2));
    }

    #[test]
    fn test_priority_based_selection() {
        let mut state = SchedulerState::new();
        state.init_tasks(vec![
            create_test_task("low", 50, 0),
            create_test_task("high", 200, 0),
            create_test_task("medium", 100, 0),
        ]);

        // Should select highest priority (task 1 with priority 200)
        let next = select_priority_based(&state);
        assert_eq!(next, Some(1));
    }

    #[test]
    fn test_rate_monotonic_selection() {
        let mut state = SchedulerState::new();
        state.init_tasks(vec![
            create_test_task("slow", 100, 1000), // Long period
            create_test_task("fast", 100, 100),  // Short period
            create_test_task("medium", 100, 500),
        ]);

        // Should select shortest period (task 1 with 100ms period)
        let next = select_rate_monotonic(&state);
        assert_eq!(next, Some(1));
    }

    #[test]
    fn test_is_task_ready() {
        let mut state = SchedulerState::new();
        state.init_tasks(vec![create_test_task("task1", 100, 0)]);

        // Task should be ready initially
        assert!(is_task_ready(&state, 0));

        // Mark as running
        state.task_states[0].state = TaskExecutionState::Running;
        assert!(!is_task_ready(&state, 0));

        // Mark as error
        state.task_states[0].state = TaskExecutionState::Error;
        assert!(!is_task_ready(&state, 0));
    }
}
