//! Time-partitioned scheduler executor implementation

use super::algorithms::{activate_periodic_tasks, check_deadline_misses, select_next_task};
use super::config::{
    HistoryEntry, ScheduleMode, SchedulerState, TaskConfig,
};
use super::timing::BudgetTimer;
use crate::graph::node::{ComponentSpec, DataType, NodeValue};
use crate::runtime::engine::NodeExecutor;
use crate::runtime::wasm_host::ComponentManager;
use crate::ComponentError;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// JSON representation of a task (for deserialization)
#[derive(serde::Deserialize)]
struct TaskConfigJson {
    component_id: String,
    #[serde(default = "default_priority")]
    priority: u8,
    #[serde(default = "default_budget")]
    budget_ms: u32,
    #[serde(default)]
    period_ms: u32,
    #[serde(default)]
    deadline_ms: u32,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    #[allow(dead_code)] // Reserved for future use when passing inputs to tasks
    inputs: HashMap<String, serde_json::Value>,
}

fn default_priority() -> u8 { 128 }
fn default_budget() -> u32 { 100 }

/// Time-Partitioned Scheduler Executor
///
/// Executes multiple WASM components with deterministic time budgets,
/// demonstrating space and time partitioning concepts from Integrity RTOS.
pub struct TimePartitionedSchedulerExecutor {
    /// Scheduler state (shared for continuous execution)
    state: Arc<Mutex<SchedulerState>>,
    /// Component manager for executing WASM components (optional for testing)
    component_manager: Option<Arc<Mutex<ComponentManager>>>,
}

impl TimePartitionedSchedulerExecutor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SchedulerState::new())),
            component_manager: None,
        }
    }

    /// Create a new scheduler with a component manager for executing real WASM components
    pub fn with_component_manager(component_manager: Arc<Mutex<ComponentManager>>) -> Self {
        Self {
            state: Arc::new(Mutex::new(SchedulerState::new())),
            component_manager: Some(component_manager),
        }
    }

    /// Set the component manager (can be called after creation)
    pub fn set_component_manager(&mut self, component_manager: Arc<Mutex<ComponentManager>>) {
        self.component_manager = Some(component_manager);
    }

    /// Parse task list from JSON string
    fn parse_task_list_from_json(json: &str) -> Result<Vec<TaskConfig>, String> {
        let json_tasks: Vec<TaskConfigJson> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut tasks = Vec::new();
        for json_task in json_tasks {
            let display_name = if json_task.display_name.is_empty() {
                json_task.component_id.clone()
            } else {
                json_task.display_name
            };

            tasks.push(TaskConfig {
                component_id: json_task.component_id,
                priority: json_task.priority,
                budget_ms: json_task.budget_ms,
                period_ms: json_task.period_ms,
                deadline_ms: json_task.deadline_ms,
                display_name,
                inputs: HashMap::new(), // TODO: Convert serde_json::Value to NodeValue if needed
            });
        }

        Ok(tasks)
    }

    /// Parse task list from input value (Record-based, for future use)
    fn parse_task_list(value: &NodeValue) -> Result<Vec<TaskConfig>, String> {
        match value {
            NodeValue::List(items) => {
                let mut tasks = Vec::new();
                for item in items {
                    if let NodeValue::Record(record) = item {
                        tasks.push(TaskConfig::from_record(record)?);
                    } else {
                        return Err(format!("Expected Record in task list, got {:?}", item));
                    }
                }
                Ok(tasks)
            }
            _ => Err(format!("Expected List for tasks, got {:?}", value)),
        }
    }

    /// Create visualization output for the current schedule state
    fn create_visualization_output(state: &SchedulerState) -> NodeValue {
        let mut viz_data = BTreeMap::new();

        // Current task information
        if !state.tasks.is_empty() && state.current_task_index < state.tasks.len() {
            let current_task = &state.tasks[state.current_task_index];
            let current_state = &state.task_states[state.current_task_index];

            viz_data.insert(
                "current_task".to_string(),
                NodeValue::String(current_task.display_name.clone()),
            );
            viz_data.insert(
                "current_task_index".to_string(),
                NodeValue::U32(state.current_task_index as u32),
            );
            viz_data.insert(
                "current_task_state".to_string(),
                NodeValue::String(format!("{:?}", current_state.state)),
            );
        }

        // Overall statistics
        viz_data.insert(
            "iterations".to_string(),
            NodeValue::U32(state.iterations as u32),
        );
        viz_data.insert(
            "context_switches".to_string(),
            NodeValue::U32(state.context_switches as u32),
        );
        viz_data.insert(
            "cpu_utilization".to_string(),
            NodeValue::F32(state.get_cpu_utilization()),
        );
        viz_data.insert(
            "total_violations".to_string(),
            NodeValue::U32(state.get_total_violations() as u32),
        );

        // Task statistics (as list of records)
        let mut task_stats = Vec::new();
        for (index, task) in state.tasks.iter().enumerate() {
            if index < state.task_states.len() {
                let task_state = &state.task_states[index];
                let mut task_record = BTreeMap::new();

                task_record.insert(
                    "name".to_string(),
                    NodeValue::String(task.display_name.clone()),
                );
                task_record.insert(
                    "activations".to_string(),
                    NodeValue::U32(task_state.activation_count as u32),
                );
                task_record.insert(
                    "completions".to_string(),
                    NodeValue::U32(task_state.completion_count as u32),
                );
                task_record.insert(
                    "overruns".to_string(),
                    NodeValue::U32(task_state.overrun_count as u32),
                );
                task_record.insert(
                    "deadline_misses".to_string(),
                    NodeValue::U32(task_state.deadline_miss_count as u32),
                );
                task_record.insert(
                    "min_execution_ms".to_string(),
                    NodeValue::U32(if task_state.min_execution_ms == u32::MAX {
                        0
                    } else {
                        task_state.min_execution_ms
                    }),
                );
                task_record.insert(
                    "max_execution_ms".to_string(),
                    NodeValue::U32(task_state.max_execution_ms),
                );
                task_record.insert(
                    "avg_execution_ms".to_string(),
                    NodeValue::F32(task_state.avg_execution_ms),
                );
                task_record.insert(
                    "last_execution_ms".to_string(),
                    NodeValue::U32(task_state.last_execution_ms),
                );

                task_stats.push(NodeValue::Record(task_record));
            }
        }
        viz_data.insert("task_stats".to_string(), NodeValue::List(task_stats));

        // Execution history (last N entries for timeline visualization)
        let mut history_entries = Vec::new();
        // Calculate the earliest start time for relative timestamps
        let earliest_start = state.execution_history
            .iter()
            .map(|e| e.start_time)
            .min()
            .unwrap_or_else(Instant::now);

        for entry in state.execution_history.iter().rev().take(20).rev() {
            let mut history_record = BTreeMap::new();

            // Look up task name from task config
            let task_name = if entry.task_index < state.tasks.len() {
                state.tasks[entry.task_index].display_name.clone()
            } else {
                format!("Task {}", entry.task_index)
            };

            // Calculate relative start time in milliseconds
            let start_time_ms = entry.start_time
                .duration_since(earliest_start)
                .as_millis() as u64;

            history_record.insert(
                "task_name".to_string(),
                NodeValue::String(task_name),
            );
            history_record.insert(
                "start_time_ms".to_string(),
                NodeValue::U32(start_time_ms as u32),
            );
            history_record.insert(
                "duration_ms".to_string(),
                NodeValue::U32(entry.duration_ms),
            );
            history_record.insert(
                "budget_exceeded".to_string(),
                NodeValue::Bool(entry.overran_budget),
            );
            history_record.insert(
                "deadline_missed".to_string(),
                NodeValue::Bool(entry.missed_deadline),
            );
            history_entries.push(NodeValue::Record(history_record));
        }
        viz_data.insert("execution_history".to_string(), NodeValue::List(history_entries));

        NodeValue::Record(viz_data)
    }
}

impl Default for TimePartitionedSchedulerExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeExecutor for TimePartitionedSchedulerExecutor {
    fn execute(
        &self,
        inputs: &HashMap<String, NodeValue>,
    ) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let mut outputs = HashMap::new();

        // Lock scheduler state
        let mut state = self.state.lock().map_err(|e| {
            ComponentError::ExecutionError(format!("Failed to lock scheduler state: {}", e))
        })?;

        // Initialize or update configuration on first run or when tasks change
        // Try JSON input first, then fall back to Record-based input
        let new_tasks = if let Some(NodeValue::String(json)) = inputs.get("tasks_json") {
            // Parse from JSON string
            Self::parse_task_list_from_json(json).map_err(|e| {
                ComponentError::ExecutionError(format!("Failed to parse tasks_json: {}", e))
            })?
        } else if let Some(tasks_value) = inputs.get("tasks") {
            // Parse from List of Records (future functionality)
            Self::parse_task_list(tasks_value).map_err(|e| {
                ComponentError::ExecutionError(format!("Failed to parse tasks: {}", e))
            })?
        } else {
            // No tasks provided - keep existing tasks or use empty list
            if state.tasks.is_empty() {
                Vec::new()
            } else {
                // Keep existing tasks
                state.tasks.clone()
            }
        };

        // Only reinitialize if tasks changed or not initialized
        if !new_tasks.is_empty() {
            let should_init = state.tasks.is_empty()
                || state.tasks.len() != new_tasks.len()
                || state.tasks.iter().zip(&new_tasks).any(|(a, b)| {
                    a.component_id != b.component_id || a.priority != b.priority
                });

            if should_init {
                log::info!("Initializing scheduler with {} tasks", new_tasks.len());
                state.init_tasks(new_tasks);
                state.started_at = Some(Instant::now());
            }
        }

        // Update configuration from inputs
        if let Some(NodeValue::U32(quantum)) = inputs.get("time_quantum_ms") {
            state.time_quantum_ms = *quantum;
        }

        if let Some(NodeValue::String(mode_str)) = inputs.get("schedule_mode") {
            state.schedule_mode = ScheduleMode::from_string(mode_str);
        }

        if let Some(NodeValue::Bool(preemption)) = inputs.get("enable_preemption") {
            state.preemption_enabled = *preemption;
        }

        // Check if we have any tasks to schedule
        if state.tasks.is_empty() {
            outputs.insert(
                "status".to_string(),
                NodeValue::String("no_tasks".to_string()),
            );
            outputs.insert(
                "schedule_state".to_string(),
                NodeValue::Record(BTreeMap::new()),
            );
            return Ok(outputs);
        }

        // Increment iteration counter
        state.iterations += 1;

        // Activate periodic tasks that are due
        activate_periodic_tasks(&mut state);

        // Check for deadline misses
        let missed_deadlines = check_deadline_misses(&mut state);
        if !missed_deadlines.is_empty() {
            log::warn!("Deadline misses detected for tasks: {:?}", missed_deadlines);
        }

        // Select next task to execute
        let next_task_index = select_next_task(&state);

        if let Some(task_index) = next_task_index {
            // Extract task data we need (to avoid borrow conflicts)
            let task_display_name = state.tasks[task_index].display_name.clone();
            let task_budget_ms = state.tasks[task_index].budget_ms;
            let task_start = Instant::now();

            // Track context switch
            if state.current_task_index != task_index {
                state.context_switches += 1;
                state.current_task_index = task_index;
            }

            // Mark task as running
            state.task_states[task_index].start_execution(task_start);

            log::debug!(
                "Executing task {} ({}) with budget {}ms",
                task_index,
                task_display_name,
                task_budget_ms
            );

            // Create budget timer
            let budget_timer = BudgetTimer::new(task_budget_ms);

            // Extract component ID and inputs for execution
            let component_id = state.tasks[task_index].component_id.clone();
            let task_inputs = state.tasks[task_index].inputs.clone();

            // Release state lock before executing component (avoid deadlock)
            drop(state);

            // Execute component (real or simulated)
            let execution_result = if let Some(ref cm) = self.component_manager {
                // Real component execution via ComponentManager
                log::debug!("Executing component: {}", component_id);

                // Try to execute with timeout = budget
                match cm.lock() {
                    Ok(mut manager) => {
                        // Convert inputs from HashMap to the format ComponentManager expects
                        let mut component_inputs = HashMap::new();
                        for (k, v) in task_inputs.iter() {
                            component_inputs.insert(k.clone(), v.clone());
                        }

                        // Execute component asynchronously (ComponentManager is async)
                        // Execute with timeout = budget
                        let timeout_duration = std::time::Duration::from_millis(task_budget_ms as u64);
                        let exec_future = manager.execute_component(
                            &component_id,
                            &component_inputs,
                            Default::default()
                        );

                        // Try to use current runtime if available
                        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                            // We're in an async context - use block_in_place to avoid blocking worker threads
                            tokio::task::block_in_place(|| {
                                handle.block_on(tokio::time::timeout(timeout_duration, exec_future))
                            })
                        } else {
                            // No runtime available - create one (standalone mode)
                            match tokio::runtime::Runtime::new() {
                                Ok(rt) => rt.block_on(tokio::time::timeout(timeout_duration, exec_future)),
                                Err(e) => {
                                    log::error!("Failed to create tokio runtime: {}", e);
                                    return Err(ComponentError::ExecutionError(
                                        format!("Failed to create tokio runtime: {}", e)
                                    ));
                                }
                            }
                        };

                        match result {
                            Ok(Ok(_outputs)) => Ok(()),
                            Ok(Err(e)) => {
                                log::warn!("Task {} execution failed: {}", task_display_name, e);
                                Err(format!("Component execution failed: {}", e))
                            }
                            Err(_) => {
                                log::warn!("Task {} exceeded budget (timeout)", task_display_name);
                                Err(format!("Component execution timed out after {}ms", task_budget_ms))
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to lock ComponentManager: {}", e);
                        Err(format!("Failed to lock ComponentManager: {}", e))
                    }
                }
            } else {
                // Simulated execution (for testing without ComponentManager)
                log::trace!("Simulating component execution: {}", component_id);
                let simulated_execution_ms = (task_budget_ms as f32 * 0.7) as u32;
                std::thread::sleep(std::time::Duration::from_millis(simulated_execution_ms as u64));
                Ok(())
            };

            // Measure actual execution time
            let execution_ms = budget_timer.elapsed_ms();
            let exceeded_budget = budget_timer.is_exceeded();

            // Re-acquire state lock to update statistics
            let mut state = self.state.lock().map_err(|e| {
                ComponentError::ExecutionError(format!("Failed to lock scheduler state: {}", e))
            })?;

            // Update task statistics
            state.task_states[task_index].record_execution(execution_ms, task_budget_ms);
            state.task_states[task_index].complete_execution();

            // Add to execution history
            let history_entry = HistoryEntry {
                task_index,
                start_time: task_start,
                duration_ms: execution_ms,
                completed: true,
                overran_budget: exceeded_budget,
                missed_deadline: false, // Would be checked against deadline tracker
            };
            state.add_history(history_entry);

            // Create outputs
            outputs.insert(
                "status".to_string(),
                NodeValue::String("executed".to_string()),
            );
            outputs.insert(
                "executed_task".to_string(),
                NodeValue::String(task_display_name.clone()),
            );
            outputs.insert(
                "execution_time_ms".to_string(),
                NodeValue::U32(execution_ms),
            );
            outputs.insert(
                "budget_exceeded".to_string(),
                NodeValue::Bool(exceeded_budget),
            );
            outputs.insert(
                "schedule_mode".to_string(),
                NodeValue::String(state.schedule_mode.to_string()),
            );
            outputs.insert(
                "execution_success".to_string(),
                NodeValue::Bool(execution_result.is_ok()),
            );
            if let Err(ref err) = execution_result {
                outputs.insert(
                    "execution_error".to_string(),
                    NodeValue::String(err.clone()),
                );
            }

            log::debug!(
                "Task {} completed in {}ms (budget: {}ms, exceeded: {})",
                task_display_name,
                execution_ms,
                task_budget_ms,
                exceeded_budget
            );
        } else {
            // No ready task
            outputs.insert(
                "status".to_string(),
                NodeValue::String("idle".to_string()),
            );
            log::trace!("No ready task to execute");
        }

        // Always include visualization data (need to re-acquire state lock)
        let state = self.state.lock().map_err(|e| {
            ComponentError::ExecutionError(format!("Failed to lock scheduler state: {}", e))
        })?;
        outputs.insert(
            "schedule_state".to_string(),
            Self::create_visualization_output(&state),
        );

        Ok(outputs)
    }
}

/// Register the scheduler node in the component registry
pub fn register_scheduler(registry: &mut crate::graph::node::ComponentRegistry) {
    use super::views::SchedulerFooterView;

    let spec = ComponentSpec::new_builtin(
        "builtin:continuous:scheduler".to_string(),
        "Time-Partitioned Scheduler".to_string(),
        "Executes multiple WASM components with deterministic time budgets, demonstrating \
         space and time partitioning concepts from GreenHills Integrity RTOS. \
         Each task runs in an isolated WASM component (space partitioning) with strict \
         time budgets (time partitioning). Provides real-time visualization of schedule execution."
            .to_string(),
        Some("Scheduler".to_string()),
    )
    // Configuration inputs
    .with_input(
        "tasks_json".to_string(),
        DataType::String,
        "Task list as JSON array. Example: [{\"component_id\":\"user:math-adder\",\"priority\":150,\"budget_ms\":50}]. \
         This is the easiest way to configure tasks - just paste JSON!"
            .to_string(),
    )
    .with_input(
        "tasks".to_string(),
        DataType::List(Box::new(DataType::Record(vec![]))),  // List of Records (future)
        "Alternative: List of task Records (not yet fully supported due to WIT limitations). \
         Use tasks_json instead."
            .to_string(),
    )
    .with_input(
        "time_quantum_ms".to_string(),
        DataType::U32,
        "Time quantum for round-robin scheduling in milliseconds (default: 100ms)"
            .to_string(),
    )
    .with_input(
        "schedule_mode".to_string(),
        DataType::String,
        "Scheduling algorithm: 'round-robin', 'priority-based', 'rate-monotonic', \
         'earliest-deadline-first', 'least-laxity-first' (default: round-robin)"
            .to_string(),
    )
    .with_input(
        "enable_preemption".to_string(),
        DataType::Bool,
        "Enable task preemption for priority-based scheduling (default: false)"
            .to_string(),
    )
    // Status outputs
    .with_output(
        "status".to_string(),
        DataType::String,
        "Scheduler status: 'executed', 'idle', 'no_tasks', or 'error'"
            .to_string(),
    )
    .with_output(
        "executed_task".to_string(),
        DataType::String,
        "Name of the task that was just executed"
            .to_string(),
    )
    .with_output(
        "execution_time_ms".to_string(),
        DataType::U32,
        "Actual execution time of the last task in milliseconds"
            .to_string(),
    )
    .with_output(
        "budget_exceeded".to_string(),
        DataType::Bool,
        "Whether the last task exceeded its time budget"
            .to_string(),
    )
    .with_output(
        "schedule_mode".to_string(),
        DataType::String,
        "Current scheduling algorithm in use"
            .to_string(),
    )
    .with_output(
        "execution_success".to_string(),
        DataType::Bool,
        "Whether the last task executed successfully"
            .to_string(),
    )
    .with_output(
        "execution_error".to_string(),
        DataType::String,
        "Error message if the last task execution failed (empty if successful)"
            .to_string(),
    )
    // Visualization output
    .with_output(
        "schedule_state".to_string(),
        DataType::Record(vec![]),  // Dynamic record structure
        "Complete schedule state for visualization including: current_task, iterations, \
         context_switches, cpu_utilization, total_violations, task_stats (per-task metrics), \
         execution_history (timeline data for Gantt chart)"
            .to_string(),
    )
    // Custom footer view with Gantt chart visualization
    .with_footer_view(SchedulerFooterView::new());

    registry.register_builtin(spec);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task_record(
        component_id: &str,
        priority: u32,
        budget_ms: u32,
    ) -> NodeValue {
        let mut record = HashMap::new();
        record.insert(
            "component_id".to_string(),
            NodeValue::String(component_id.to_string()),
        );
        record.insert("priority".to_string(), NodeValue::U32(priority));
        record.insert("budget_ms".to_string(), NodeValue::U32(budget_ms));
        record.insert("period_ms".to_string(), NodeValue::U32(0)); // Aperiodic
        record.insert("deadline_ms".to_string(), NodeValue::U32(budget_ms * 2));
        record.insert(
            "display_name".to_string(),
            NodeValue::String(component_id.to_string()),
        );
        NodeValue::Record(record)
    }

    #[test]
    fn test_parse_task_list() {
        let tasks = NodeValue::List(vec![
            create_test_task_record("task1", 100, 50),
            create_test_task_record("task2", 150, 75),
        ]);

        let parsed = TimePartitionedSchedulerExecutor::parse_task_list(&tasks).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].component_id, "task1");
        assert_eq!(parsed[0].priority, 100);
        assert_eq!(parsed[1].component_id, "task2");
        assert_eq!(parsed[1].priority, 150);
    }

    #[test]
    fn test_executor_initialization() {
        let executor = TimePartitionedSchedulerExecutor::new();
        let mut inputs = HashMap::new();

        inputs.insert(
            "tasks".to_string(),
            NodeValue::List(vec![
                create_test_task_record("task1", 100, 50),
                create_test_task_record("task2", 100, 75),
            ]),
        );
        inputs.insert("time_quantum_ms".to_string(), NodeValue::U32(100));
        inputs.insert(
            "schedule_mode".to_string(),
            NodeValue::String("round-robin".to_string()),
        );

        let result = executor.execute(&inputs);
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert!(outputs.contains_key("status"));
        assert!(outputs.contains_key("schedule_state"));
    }

    #[test]
    fn test_executor_empty_tasks() {
        let executor = TimePartitionedSchedulerExecutor::new();
        let mut inputs = HashMap::new();
        inputs.insert("tasks".to_string(), NodeValue::List(vec![]));

        let result = executor.execute(&inputs).unwrap();
        assert_eq!(
            result.get("status"),
            Some(&NodeValue::String("no_tasks".to_string()))
        );
    }
}
