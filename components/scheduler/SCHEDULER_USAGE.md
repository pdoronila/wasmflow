# Using the Time-Partitioned Scheduler

This guide explains how to configure and use the scheduler with your existing nodes.

## Overview

The scheduler executes multiple WASM components with deterministic time budgets, demonstrating space and time partitioning concepts from GreenHills Integrity RTOS.

## Quick Start: Creating a Task List

### Step 1: Create Individual Tasks with Task Builder

1. **Add Task Builder nodes** (one for each task you want to schedule)
   - Find "Task Builder" in the Scheduler category
   - Drag it onto the canvas

2. **Configure each task**:
   - Connect a **String Constant** to `component_id` (e.g., "user:math-adder")
   - Connect **U32 Constants** to configure:
     - `priority` (0-255, higher = more important) - default: 128
     - `budget_ms` (max execution time in ms) - default: 100
     - `period_ms` (for periodic tasks, 0 = aperiodic) - default: 0
     - `deadline_ms` (relative deadline in ms, 0 = use period) - default: 0
   - Optionally connect a **String Constant** to `display_name`

3. **Parse JSON to Record**:
   - Each Task Builder outputs `task_json` (a JSON string)
   - Connect this to a **JSON Parser** component
   - The JSON Parser will convert it to a Record

### Step 2: Collect Tasks into a List

You have two options:

#### Option A: Use JSON Array (Simpler)
1. Create a **String Constant** with a JSON array of all tasks:
```json
[
  {
    "component_id": "user:math-adder",
    "priority": 150,
    "budget_ms": 50,
    "period_ms": 0,
    "deadline_ms": 0,
    "display_name": "Add Task",
    "inputs": {}
  },
  {
    "component_id": "user:string-concat",
    "priority": 100,
    "budget_ms": 75,
    "period_ms": 1000,
    "deadline_ms": 900,
    "display_name": "String Task",
    "inputs": {}
  }
]
```
2. Connect to **JSON Parser** → outputs a List of Records

#### Option B: Use Multiple Task Builders (More Visual)
1. Create multiple Task Builder nodes (one per task)
2. Each outputs a Record (via JSON Parser)
3. Use a **List Builder** component (if available) or create a JSON array manually

### Step 3: Connect to Scheduler

1. **Add the Scheduler node**:
   - Find "Time-Partitioned Scheduler" in the Scheduler category
   - It's a continuous node (runs repeatedly)

2. **Connect the task list**:
   - Connect your List of Records to the `tasks` input port

3. **Configure scheduler settings** (optional):
   - `time_quantum_ms` (U32) - Time slice for round-robin (default: 100ms)
   - `schedule_mode` (String) - Algorithm to use:
     - `"round-robin"` - Equal time slices (default)
     - `"priority-based"` - Highest priority first
     - `"rate-monotonic"` - Shorter period = higher priority
     - `"earliest-deadline-first"` - Earliest deadline first
     - `"least-laxity-first"` - Minimum slack time first
   - `enable_preemption` (Bool) - Allow task preemption (default: false)

4. **Start continuous execution**:
   - Right-click on the scheduler node
   - Select "Start" to begin scheduling

### Step 4: Monitor Execution

The scheduler outputs:
- `status` (String) - "executed", "idle", or "no_tasks"
- `executed_task` (String) - Name of the last executed task
- `execution_time_ms` (U32) - Actual execution time
- `budget_exceeded` (Bool) - Whether the task exceeded its budget
- `execution_success` (Bool) - Whether execution succeeded
- `execution_error` (String) - Error message if execution failed
- `schedule_state` (Record) - Complete statistics and visualization data

## Example: Simple Round-Robin Schedule

```
Task 1: math-adder (priority: 100, budget: 50ms)
Task 2: string-concat (priority: 100, budget: 75ms)
Task 3: double-number (priority: 100, budget: 60ms)

Result: Tasks execute in order: 1 → 2 → 3 → 1 → 2 → 3 → ...
```

## Example: Priority-Based Schedule

```
Task 1: sensor-reader (priority: 200, budget: 50ms)  ← High priority
Task 2: data-processor (priority: 150, budget: 75ms) ← Medium priority
Task 3: logger (priority: 50, budget: 100ms)         ← Low priority

Result: Task 1 always runs first, then Task 2, then Task 3
```

## Example: Periodic Tasks

```
Task 1: sensor-reader (period: 100ms, deadline: 90ms, budget: 20ms)
Task 2: actuator-update (period: 500ms, deadline: 450ms, budget: 50ms)

Result: sensor-reader runs every 100ms, actuator-update runs every 500ms
The scheduler ensures deadlines are met and tracks violations.
```

## Key Concepts

### Space Partitioning
Each task runs in its own isolated WASM component with separate memory space. Tasks cannot interfere with each other's memory.

### Time Partitioning
Each task has a strict time budget. If a task exceeds its budget, it's terminated and marked as a budget violation.

### Scheduling Algorithms

**Round-Robin**: Fair time-slicing, all tasks get equal time
**Priority-Based**: Higher priority tasks run first
**Rate Monotonic**: Shorter period = higher priority (static)
**Earliest Deadline First**: Dynamic priority based on absolute deadlines
**Least Laxity First**: Minimize slack time (deadline - remaining time)

## Troubleshooting

**"Missing required input: component_id"**
- Make sure you connect a String to the Task Builder's `component_id` input

**"Component execution failed"**
- Check that the component_id is correct (e.g., "user:math-adder")
- Verify the component is installed in `components/bin/`

**"No ready task to execute"**
- Check that your tasks are properly configured
- For periodic tasks, make sure periods are set correctly

**Budget exceeded frequently**
- Increase the `budget_ms` value
- Or optimize the component being executed

## Next Steps

1. Try different scheduling algorithms to see how they behave
2. Create periodic tasks to simulate real-time systems
3. Experiment with priority inversions and deadline violations
4. Use the visualization data to create Gantt charts (coming soon!)
