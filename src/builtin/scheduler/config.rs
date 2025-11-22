//! Configuration and state management for the scheduler

use crate::graph::node::NodeValue;
use std::collections::{BTreeMap, HashMap};
use std::time::{Duration, Instant};

/// Scheduling algorithm mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleMode {
    /// Round-robin with fixed time quantum
    RoundRobin,
    /// Priority-based preemptive scheduling
    PriorityBased,
    /// Rate Monotonic Scheduling (shorter period = higher priority)
    RateMonotonic,
    /// Earliest Deadline First
    EarliestDeadlineFirst,
    /// Least Laxity First
    LeastLaxityFirst,
}

impl ScheduleMode {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "round-robin" => Self::RoundRobin,
            "priority" | "priority-based" => Self::PriorityBased,
            "rms" | "rate-monotonic" => Self::RateMonotonic,
            "edf" | "earliest-deadline-first" => Self::EarliestDeadlineFirst,
            "llf" | "least-laxity-first" => Self::LeastLaxityFirst,
            _ => Self::RoundRobin, // Default
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::RoundRobin => "round-robin".to_string(),
            Self::PriorityBased => "priority-based".to_string(),
            Self::RateMonotonic => "rate-monotonic".to_string(),
            Self::EarliestDeadlineFirst => "earliest-deadline-first".to_string(),
            Self::LeastLaxityFirst => "least-laxity-first".to_string(),
        }
    }
}

/// Configuration for a scheduled task
#[derive(Debug, Clone)]
pub struct TaskConfig {
    /// Component ID to execute
    pub component_id: String,
    /// Task priority (higher = more important, 0-255)
    pub priority: u8,
    /// Maximum execution time budget in milliseconds
    pub budget_ms: u32,
    /// Period for periodic tasks (0 = aperiodic)
    pub period_ms: u32,
    /// Relative deadline (defaults to period if not set)
    pub deadline_ms: u32,
    /// Inputs to pass to the component
    pub inputs: HashMap<String, NodeValue>,
    /// Display name for visualization
    pub display_name: String,
}

impl TaskConfig {
    /// Create a new task configuration
    pub fn new(component_id: String) -> Self {
        Self {
            component_id: component_id.clone(),
            priority: 128, // Default mid-priority
            budget_ms: 100,
            period_ms: 0, // Aperiodic by default
            deadline_ms: 0,
            display_name: component_id,
            inputs: HashMap::new(),
        }
    }

    /// Check if this is a periodic task
    pub fn is_periodic(&self) -> bool {
        self.period_ms > 0
    }

    /// Get the deadline (uses period if deadline not explicitly set)
    pub fn get_deadline(&self) -> u32 {
        if self.deadline_ms > 0 {
            self.deadline_ms
        } else {
            self.period_ms
        }
    }

    /// Parse task configuration from NodeValue::Record
    pub fn from_record(record: &BTreeMap<String, NodeValue>) -> Result<Self, String> {
        let component_id = match record.get("component_id") {
            Some(NodeValue::String(id)) => id.clone(),
            _ => return Err("Missing or invalid 'component_id' field".to_string()),
        };

        let mut task = Self::new(component_id);

        // Parse optional fields
        if let Some(NodeValue::U32(priority)) = record.get("priority") {
            task.priority = (*priority).min(255) as u8;
        }

        if let Some(NodeValue::U32(budget)) = record.get("budget_ms") {
            task.budget_ms = *budget;
        }

        if let Some(NodeValue::U32(period)) = record.get("period_ms") {
            task.period_ms = *period;
        }

        if let Some(NodeValue::U32(deadline)) = record.get("deadline_ms") {
            task.deadline_ms = *deadline;
        }

        if let Some(NodeValue::String(name)) = record.get("display_name") {
            task.display_name = name.clone();
        }

        // Parse inputs if provided
        if let Some(NodeValue::Record(inputs_record)) = record.get("inputs") {
            // Convert BTreeMap to HashMap
            task.inputs = inputs_record.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        }

        Ok(task)
    }
}

/// Runtime state for a scheduled task
#[derive(Debug, Clone)]
pub struct TaskState {
    /// Task index in the schedule
    pub task_index: usize,
    /// Current execution state
    pub state: TaskExecutionState,
    /// Total number of activations
    pub activation_count: u64,
    /// Number of completed executions
    pub completion_count: u64,
    /// Number of budget overruns
    pub overrun_count: u64,
    /// Number of missed deadlines
    pub deadline_miss_count: u64,
    /// Minimum execution time observed (ms)
    pub min_execution_ms: u32,
    /// Maximum execution time observed (ms)
    pub max_execution_ms: u32,
    /// Average execution time (ms)
    pub avg_execution_ms: f32,
    /// Last execution time (ms)
    pub last_execution_ms: u32,
    /// Next activation time for periodic tasks
    pub next_activation: Option<Instant>,
    /// Last activation time
    pub last_activation: Option<Instant>,
    /// When current execution started (if running)
    pub execution_start: Option<Instant>,
}

impl TaskState {
    pub fn new(task_index: usize) -> Self {
        Self {
            task_index,
            state: TaskExecutionState::Ready,
            activation_count: 0,
            completion_count: 0,
            overrun_count: 0,
            deadline_miss_count: 0,
            min_execution_ms: u32::MAX,
            max_execution_ms: 0,
            avg_execution_ms: 0.0,
            last_execution_ms: 0,
            next_activation: None,
            last_activation: None,
            execution_start: None,
        }
    }

    /// Update statistics after task execution
    pub fn record_execution(&mut self, duration_ms: u32, budget_ms: u32) {
        self.completion_count += 1;
        self.last_execution_ms = duration_ms;

        // Update min/max
        self.min_execution_ms = self.min_execution_ms.min(duration_ms);
        self.max_execution_ms = self.max_execution_ms.max(duration_ms);

        // Update rolling average
        let count = self.completion_count as f32;
        self.avg_execution_ms =
            ((self.avg_execution_ms * (count - 1.0)) + duration_ms as f32) / count;

        // Check for budget overrun
        if duration_ms > budget_ms {
            self.overrun_count += 1;
        }
    }

    /// Check and record deadline miss
    pub fn check_deadline(&mut self, deadline: Instant) -> bool {
        if Instant::now() > deadline {
            self.deadline_miss_count += 1;
            true
        } else {
            false
        }
    }

    /// Activate the task (mark as ready to run)
    pub fn activate(&mut self, now: Instant) {
        self.activation_count += 1;
        self.last_activation = Some(now);
        self.state = TaskExecutionState::Ready;
    }

    /// Start executing the task
    pub fn start_execution(&mut self, now: Instant) {
        self.execution_start = Some(now);
        self.state = TaskExecutionState::Running;
    }

    /// Complete task execution
    pub fn complete_execution(&mut self) {
        self.execution_start = None;
        self.state = TaskExecutionState::Ready;
    }
}

/// Execution state of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskExecutionState {
    /// Task is ready to run
    Ready,
    /// Task is currently executing
    Running,
    /// Task is blocked/waiting
    Blocked,
    /// Task completed (for one-shot tasks)
    Completed,
    /// Task encountered an error
    Error,
}

/// Complete scheduler state (not persisted, runtime only)
pub struct SchedulerState {
    /// Task configurations
    pub tasks: Vec<TaskConfig>,
    /// Runtime state for each task
    pub task_states: Vec<TaskState>,
    /// Current task index being executed (for round-robin)
    pub current_task_index: usize,
    /// Total context switches
    pub context_switches: u64,
    /// Scheduler start time
    pub started_at: Option<Instant>,
    /// Total scheduler iterations
    pub iterations: u64,
    /// Recent execution history (circular buffer, last N slots)
    pub execution_history: Vec<HistoryEntry>,
    /// Maximum history entries to keep
    pub max_history: usize,
    /// Current schedule mode
    pub schedule_mode: ScheduleMode,
    /// Time quantum for round-robin (ms)
    pub time_quantum_ms: u32,
    /// Whether preemption is enabled
    pub preemption_enabled: bool,
}

/// Entry in execution history for visualization
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// Which task executed
    pub task_index: usize,
    /// When it started
    pub start_time: Instant,
    /// How long it ran (ms)
    pub duration_ms: u32,
    /// Whether it completed successfully
    pub completed: bool,
    /// Whether it exceeded budget
    pub overran_budget: bool,
    /// Whether it missed deadline
    pub missed_deadline: bool,
}

impl SchedulerState {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            task_states: Vec::new(),
            current_task_index: 0,
            context_switches: 0,
            started_at: None,
            iterations: 0,
            execution_history: Vec::new(),
            max_history: 100,
            schedule_mode: ScheduleMode::RoundRobin,
            time_quantum_ms: 100,
            preemption_enabled: false,
        }
    }

    /// Initialize task states from task configurations
    pub fn init_tasks(&mut self, tasks: Vec<TaskConfig>) {
        self.tasks = tasks;
        self.task_states = (0..self.tasks.len())
            .map(|i| TaskState::new(i))
            .collect();
        self.current_task_index = 0;

        // For periodic tasks, set initial activation time
        let now = Instant::now();
        for (i, task) in self.tasks.iter().enumerate() {
            if task.is_periodic() {
                self.task_states[i].next_activation =
                    Some(now + Duration::from_millis(task.period_ms as u64));
            }
        }
    }

    /// Add entry to execution history
    pub fn add_history(&mut self, entry: HistoryEntry) {
        self.execution_history.push(entry);
        if self.execution_history.len() > self.max_history {
            self.execution_history.remove(0);
        }
    }

    /// Get CPU utilization (0.0 to 1.0)
    pub fn get_cpu_utilization(&self) -> f32 {
        if self.execution_history.is_empty() {
            return 0.0;
        }

        let total_time_ms: u32 = self.execution_history.iter().map(|e| e.duration_ms).sum();
        let elapsed_ms = if let Some(first) = self.execution_history.first() {
            first.start_time.elapsed().as_millis() as u32
        } else {
            1 // Avoid division by zero
        };

        (total_time_ms as f32 / elapsed_ms as f32).min(1.0)
    }

    /// Get total violations count
    pub fn get_total_violations(&self) -> u64 {
        self.task_states
            .iter()
            .map(|s| s.overrun_count + s.deadline_miss_count)
            .sum()
    }
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self::new()
    }
}
