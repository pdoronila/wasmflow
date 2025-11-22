# Gantt Chart Visualization

The Time-Partitioned Scheduler now includes a **real-time Gantt chart** in its footer that visualizes task execution history!

## What You'll See

When you select a scheduler node (click on it), the footer displays:

### 1. Statistics Dashboard

A compact dashboard showing:
- **Iterations**: Total number of scheduler cycles
- **Context Switches**: Number of times scheduler switched between tasks
- **CPU Utilization**: Percentage of CPU time used (color-coded)
  - 🟢 **Green**: Healthy (< 80%)
  - 🟡 **Yellow**: High (80-95%)
  - 🔴 **Red**: Near saturation (> 95%)
- **Violations**: Total budget overruns + deadline misses (color-coded)
  - 🟢 **Green**: No violations
  - 🔴 **Red**: Violations detected

### 2. Execution Timeline (Gantt Chart)

A visual timeline showing the **last 20 task executions**:

```
Task 1: ████████░░░░░░     (5ms - Success)
Task 2:         ██████░░   (3ms - Success)
Task 3:               ████ (7ms - Budget Exceeded!)
```

#### Visual Encoding

**Colors indicate execution status:**
- 🟢 **Green**: Task completed successfully within budget
- 🟠 **Orange**: Task exceeded its time budget (budget violation)
- 🔴 **Red**: Task missed its deadline (deadline violation)

**Timeline features:**
- Task names on the left (truncated if long)
- Horizontal bars showing when each task ran
- Bar width represents execution duration
- Timeline automatically scales to fit recent executions
- Hover over any bar for detailed information

### 3. Interactive Tooltips

Hover your mouse over any colored bar to see:
- Task name
- Start time (milliseconds since scheduler started)
- Execution duration (milliseconds)
- Warning indicators for budget/deadline violations

## Example: What You'll See

### Healthy Execution (Priority-Based)

```
Schedule Visualization
─────────────────────────────────────────────
Iterations:         142
Context Switches:   95
CPU Utilization:    68.3%  [Green]
Violations:         0      [Green]

Execution Timeline (Last 20 Iterations)
Legend: █ Success  █ Budget Exceeded  █ Deadline Missed

High Priority Add       ████░░░░░░░░░░░░░░░░  (4ms)
High Priority Add       ░░░░████░░░░░░░░░░░░  (4ms)
Medium Priority Mult    ░░░░░░░░████░░░░░░░░  (6ms)
High Priority Add       ░░░░░░░░░░░░████░░░░  (4ms)
Low Priority Divide     ░░░░░░░░░░░░░░░░████  (5ms)
```

Notice: High-priority task runs most frequently!

### Stressed System (Budget Violations)

```
Schedule Visualization
─────────────────────────────────────────────
Iterations:         523
Context Switches:   312
CPU Utilization:    96.7%  [Red - Near Saturation!]
Violations:         12     [Red - Issues Detected!]

Execution Timeline (Last 20 Iterations)
Legend: █ Success  █ Budget Exceeded  █ Deadline Missed

Fast Sensor (10Hz)      ████░░░░░░░░░░░░░░░░  (18ms - OK)
Actuator (2Hz)          ░░░░██████████░░░░░░  (45ms - BUDGET EXCEEDED!)
Fast Sensor (10Hz)      ░░░░░░░░░░░░██░░░░░░  (20ms - OK)
Status Logger           ░░░░░░░░░░░░░░██████  (98ms - OK)
```

Notice: Actuator took 45ms but only had 40ms budget - shown in orange!

### Real-Time System (EDF)

```
Schedule Visualization
─────────────────────────────────────────────
Iterations:         1024
Context Switches:   867
CPU Utilization:    82.4%  [Yellow - High]
Violations:         0      [Green]

Execution Timeline (Last 20 Iterations)
Legend: █ Success  █ Budget Exceeded  █ Deadline Missed

Fast Sensor (10Hz)      ███░░░░░░░░░░░░░░░░░  (15ms - 100ms period)
Fast Sensor (10Hz)      ░░░███░░░░░░░░░░░░░░  (15ms - 100ms period)
Actuator Update (2Hz)   ░░░░░░████████░░░░░░  (35ms - 500ms period)
Fast Sensor (10Hz)      ░░░░░░░░░░███░░░░░░░  (15ms - 100ms period)
Status Logger (0.2Hz)   ░░░░░░░░░░░░████████  (95ms - 5000ms period)
```

Notice: EDF ensures fast sensor runs every ~100ms!

## Educational Value

The Gantt chart helps you **see** RTOS concepts in action:

### 1. Priority Scheduling

Watch high-priority tasks execute more frequently than low-priority ones. The timeline makes priority inversion immediately visible.

### 2. Budget Enforcement

Orange bars show when tasks violate their time budgets. This demonstrates temporal isolation - tasks cannot steal time from others.

### 3. Deadline Monitoring

Red bars show deadline misses in periodic tasks. This is critical for real-time system design.

### 4. Context Switching

See how often the scheduler switches between tasks. Higher context switches = more overhead.

### 5. CPU Utilization

The utilization percentage shows how busy your system is. Above 100% means the system is oversubscribed!

### 6. Execution Patterns

Different algorithms create different patterns:
- **Round-Robin**: Regular, alternating pattern
- **Priority-Based**: High-priority tasks cluster together
- **EDF**: Tasks execute just before deadlines
- **RMS**: Shorter-period tasks run more frequently

## How It Works

The scheduler tracks execution history in the `schedule_state` output:

```rust
{
  "execution_history": [
    {
      "task_name": "High Priority Add",
      "start_time_ms": 15234,
      "duration_ms": 4,
      "budget_exceeded": false,
      "deadline_missed": false
    },
    // ... up to 20 most recent executions
  ]
}
```

The footer view (`src/builtin/scheduler/views.rs`) parses this data and renders it as a Gantt chart using egui primitives.

## Performance

The visualization is lightweight:
- Renders in <10ms (imperceptible)
- Only stores last 20 executions (minimal memory)
- Updates every scheduler iteration (real-time feedback)
- Uses egui immediate mode rendering (no manual state sync needed)

## Limitations

- **History size**: Only last 20 executions shown (prevents clutter)
- **Time resolution**: Millisecond precision (adequate for teaching)
- **Static layout**: Timeline auto-scales but doesn't support zoom/pan
- **No export**: Currently view-only (future: export to PNG/SVG)

## Future Enhancements

Potential improvements:
- **Interactive controls**: Click to pause/resume, zoom timeline
- **Task filtering**: Hide/show specific tasks
- **Export**: Save Gantt chart as image
- **Metrics overlay**: Show budget/deadline lines on timeline
- **Color customization**: User-configurable color scheme
- **Timeline scrubbing**: Click to see state at specific time

## Tips for Teaching

### Exercise 1: Observe Priority Inversion

1. Load `scheduler_demo.json`
2. Start the scheduler
3. Watch the Gantt chart - high-priority task should dominate
4. Change all priorities to 100 (edit JSON constant)
5. Restart and observe round-robin pattern

### Exercise 2: Create Budget Violations

1. Load `scheduler_demo.json`
2. Edit task list JSON - set all `budget_ms` to `10`
3. Start scheduler
4. Watch orange bars appear - components can't finish in 10ms!
5. Observe `Violations` counter increase

### Exercise 3: Real-Time Scheduling

1. Load `scheduler_periodic_demo.json`
2. Start scheduler using EDF algorithm
3. Watch periodic pattern emerge
4. Fast sensor (10Hz) runs every ~100ms
5. Try increasing all budgets - watch for red bars (deadline misses)

### Exercise 4: CPU Saturation

1. Load `scheduler_demo.json`
2. Duplicate tasks to create 10+ tasks (edit JSON)
3. Watch CPU utilization climb toward 100%
4. Observe increased context switches
5. System becomes oversubscribed - violations occur

## Code Reference

**Footer View Implementation**: `src/builtin/scheduler/views.rs`
- `SchedulerFooterView` struct
- `render_footer()` - Main rendering logic
- `render_gantt_chart()` - Timeline visualization

**Data Generation**: `src/builtin/scheduler/executor.rs`
- Lines 450-507: Builds `execution_history` array
- Lines 470-490: Converts to `NodeValue::Record` for output

**Integration**: `src/builtin/scheduler/mod.rs`
- Exports `SchedulerFooterView`
- Registered in `register_scheduler()` with `.with_footer_view()`

## Conclusion

The Gantt chart transforms an abstract scheduler into a **visual teaching tool**. Students can:
- **See** how algorithms work in real-time
- **Understand** budget enforcement through color coding
- **Experiment** with parameters and observe results
- **Learn** RTOS concepts through interactive exploration

This makes GreenHills Integrity RTOS concepts tangible and observable!
